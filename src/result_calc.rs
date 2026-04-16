use std::{
    fmt::Display,
    iter::zip,
    ops::{Deref, DerefMut, Div},
    path::Path,
    sync::{Arc, Mutex},
};

use crossbeam::channel::{Receiver, Sender};
use futures::future::join_all;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use plotters::prelude::*;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use sqlx::SqlitePool;
use tokio_stream::StreamExt;

use crate::{
    image_hash::{HashingMethod, HashingMethods},
    image_modify::{ImageModification, Modifications},
    matching::state::Match,
};

#[derive(Default, Debug, Copy, Clone)]
pub struct ConfusionMatrix {
    true_positives: u32,
    true_negatives: u32,
    false_positives: u32,
    false_negatives: u32,
}
impl ConfusionMatrix {
    pub fn increment(&mut self, class: Classification) {
        match class {
            Classification::FalsePositive => self.false_positives += 1,
            Classification::TrueNegative => self.true_negatives += 1,
            Classification::TruePositive => self.true_positives += 1,
            Classification::FalseNegative => self.false_negatives += 1,
        }
    }
    pub fn inc(&mut self, class: Classification, delta: u32) {
        match class {
            Classification::FalsePositive => self.false_positives += delta,
            Classification::TrueNegative => self.true_negatives += delta,
            Classification::TruePositive => self.true_positives += delta,
            Classification::FalseNegative => self.false_negatives += delta,
        }
    }
    pub fn fp_rate(&self) -> f32 {
        (self.false_positives as f32).div((self.false_positives + self.true_negatives) as f32)
    }
    pub fn tp_rate(&self) -> f32 {
        (self.true_positives as f32).div((self.true_positives + self.false_negatives) as f32)
    }
    pub fn extend(&mut self, other: Self) {
        self.true_positives += other.true_positives;
        self.true_negatives += other.true_negatives;
        self.false_positives += other.false_positives;
        self.false_negatives += other.false_negatives;
    }
}

#[derive(Default, Debug, Clone)]
pub struct Roc {
    roc: Vec<ConfusionMatrix>,
}
impl Roc {
    pub fn merge(&mut self, other: Self) {
        for (i, entry) in other.into_iter().enumerate() {
            match self.get_mut(i) {
                Some(e) => e.extend(entry),
                None => self.push(entry),
            }
        }
    }
}
impl IntoIterator for Roc {
    type Item = ConfusionMatrix;
    type IntoIter = std::vec::IntoIter<ConfusionMatrix>;
    fn into_iter(self) -> Self::IntoIter {
        self.roc.into_iter()
    }
}
impl Deref for Roc {
    type Target = Vec<ConfusionMatrix>;
    fn deref(&self) -> &Self::Target {
        &self.roc
    }
}
impl DerefMut for Roc {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.roc
    }
}
pub struct RocProcess {
    thresholds: Vec<f32>,
    pool: SqlitePool,
    hashing_methods: HashingMethods,
    modifications: Modifications,
}
impl RocProcess {
    pub fn new(
        thresholds: Vec<f32>,
        pool: SqlitePool,
        modifications: Modifications,
        hashing_methods: HashingMethods,
    ) -> Self {
        Self {
            thresholds,
            pool,
            hashing_methods,
            modifications,
        }
    }
    /// Calculates the `ConfusionMatrix` for the given matches.
    pub async fn run(self) -> Result<Roc, Error> {
        println!("starting");
        let entries = self.modifications.iter().flat_map(|m| {
            self.hashing_methods
                .iter()
                .map(async |h| match_fetcher(h.as_ref(), m.as_ref(), self.pool.clone()).await)
        });
        let entries = join_all(entries).await;

        println!("starting roc calc");
        let (tx, rx) = tokio::sync::oneshot::channel();
        rayon::spawn(move || {
            let res = entries
                .into_par_iter()
                .map(|r| {
                    let mut roc = Roc::default();
                    while let Ok(Data { m, is_same_image }) = r.recv() {
                        for (i, threshold) in self.thresholds.iter().enumerate() {
                            let class = classify(
                                *threshold,
                                m.hamming_distance().relative(),
                                is_same_image,
                            );

                            // Get the entry that corresponds to the threshold if it exists.
                            match roc.get_mut(i) {
                                Some(p) => {
                                    p.increment(class);
                                }
                                None => {
                                    let mut matrix = ConfusionMatrix::default();
                                    matrix.increment(class);
                                    roc.push(matrix);
                                }
                            }
                        }
                    }
                    roc
                })
                .reduce(
                    || Roc::default(),
                    |mut acc, other| {
                        acc.merge(other);
                        acc
                    },
                );
            tx.send(res).unwrap();
        });
        let res = rx.await.unwrap();
        Ok(res)
    }
}
pub struct Data {
    pub m: Match,
    pub is_same_image: bool,
}

/// Fetches matches for a given hashing_method and modification
async fn match_fetcher(
    hashing_method: &dyn HashingMethod,
    modification: &dyn ImageModification,
    pool: SqlitePool,
) -> Receiver<Data> {
    let hm_name = hashing_method.name();
    let m_name = modification.name().to_string();
    let (tx, rx) = crossbeam::channel::bounded(100);
    get_matches(&pool, &hm_name, &m_name, tx.clone()).await;
    rx
}
async fn get_matches(pool: &SqlitePool, hm: &str, m: &str, tx: Sender<Data>) {
    println!("finding count");
    let count: (u32,) = sqlx::query_as(
        "
SELECT count(m.id)
FROM matches m
JOIN hashes h1 ON m.hash1_id = h1.id
JOIN hashing_methods ha1 ON h1.hashing_method_id = ha1.id
JOIN modified_images mi1 ON h1.mod_image_id = mi1.id
JOIN modifications mod1 ON mi1.modification_id = mod1.id

JOIN hashes h2 ON m.hash2_id = h2.id
JOIN hashing_methods ha2 ON h2.hashing_method_id = ha2.id
JOIN modified_images mi2 ON h2.mod_image_id = mi2.id
JOIN modifications mod2 ON mi2.modification_id = mod2.id

WHERE mod1.name = ?
  AND mod2.name = ?
  AND ha1.name = ?
  AND ha2.name = ?;
                    ",
    )
    .bind(m)
    .bind(m)
    .bind(hm)
    .bind(hm)
    .fetch_one(pool)
    .await
    .unwrap();

    let mut streamer = sqlx::query_as(
        "
SELECT 
    m.hamming_distance, 
    m.hash_len, 
    m.hash1_id, 
    m.hash2_id
FROM matches m
JOIN hashes h1 ON m.hash1_id = h1.id
JOIN hashing_methods ha1 ON h1.hashing_method_id = ha1.id
JOIN modified_images mi1 ON h1.mod_image_id = mi1.id
JOIN modifications mod1 ON mi1.modification_id = mod1.id

JOIN hashes h2 ON m.hash2_id = h2.id
JOIN hashing_methods ha2 ON h2.hashing_method_id = ha2.id
JOIN modified_images mi2 ON h2.mod_image_id = mi2.id
JOIN modifications mod2 ON mi2.modification_id = mod2.id

WHERE mod1.name = ?
  AND mod2.name = ?
  AND ha1.name = ?
  AND ha2.name = ?;
                    ",
    )
    .bind(m)
    .bind(m)
    .bind(hm)
    .bind(hm)
    .fetch(pool);

    println!("len: {}", count.0);
    let style = ProgressStyle::with_template(
        "[{elapsed_precise} | {eta_precise}] Sending results to DB: {pos:>7}/{len:7} {percent}%",
    )
    .unwrap()
    .progress_chars("##-");
    let pb = ProgressBar::new(count.0 as u64).with_style(style);
    while let Some(m) = streamer.try_next().await.unwrap() {
        let m: Match = m; // Inferring type

        let is_same_image = match is_same_image(&pool, m.hash_id1(), m.hash_id2()).await {
            Ok(r) => r,
            Err(_) => break,
        };
        if tx.send(Data { m, is_same_image }).is_err() {
            break;
        };
        pb.inc(1);
    }
}

async fn is_same_image(pool: &SqlitePool, hash1_id: u32, hash2_id: u32) -> Result<bool, Error> {
    let is_same: (bool,) = sqlx::query_as(
        "
        SELECT (mi1.image_id = mi2.image_id) FROM
        hashes h1 
        JOIN modified_images mi1 ON h1.mod_image_id = mi1.id
        JOIN hashes h2
        JOIN modified_images mi2 ON h2.mod_image_id = mi2.id
        WHERE h1.id = ? AND h2.id = ?
    ",
    )
    .bind(hash1_id)
    .bind(hash2_id)
    .fetch_one(pool)
    .await?;

    Ok(is_same.0)
}

pub enum Classification {
    FalsePositive,
    FalseNegative,
    TruePositive,
    TrueNegative,
}

/// Classify an entry based a threshold and what its actual condition is.
fn classify(threshold: f32, entry: f32, actual_positive: bool) -> Classification {
    let predicted_same = entry < threshold;
    match (actual_positive, predicted_same) {
        (true, true) => Classification::TruePositive,
        (true, false) => Classification::FalseNegative,
        (false, true) => Classification::FalsePositive,
        (false, false) => Classification::TrueNegative,
    }
}

pub fn plot_roc(roc: Roc, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(path, (1024, 768)).into_drawing_area();

    root.fill(&WHITE)?;

    let sd = 0.13;
    let data: Vec<(f64, f64)> = roc
        .into_iter()
        .map(|c| (c.fp_rate() as f64, c.tp_rate() as f64))
        .collect();

    let areas = root.split_by_breakpoints([944], [80]);

    let mut x_hist_ctx = ChartBuilder::on(&areas[0])
        .y_label_area_size(40)
        .build_cartesian_2d((0.0..1.0).step(0.01).use_round().into_segmented(), 0..250)?;
    let mut y_hist_ctx = ChartBuilder::on(&areas[3])
        .x_label_area_size(40)
        .build_cartesian_2d(0..250, (0.0..1.0).step(0.01).use_round())?;
    let mut scatter_ctx = ChartBuilder::on(&areas[2])
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0f64..1f64, 0f64..1f64)?;
    scatter_ctx
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;
    scatter_ctx.draw_series(
        data.iter()
            .map(|(x, y)| Circle::new((*x, *y), 2, GREEN.filled())),
    )?;
    let x_hist = Histogram::vertical(&x_hist_ctx)
        .style(GREEN.filled())
        .margin(0)
        .data(data.iter().map(|(x, _)| (*x, 1)));
    let y_hist = Histogram::horizontal(&y_hist_ctx)
        .style(GREEN.filled())
        .margin(0)
        .data(data.iter().map(|(_, y)| (*y, 1)));
    x_hist_ctx.draw_series(x_hist)?;
    y_hist_ctx.draw_series(y_hist)?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", path.to_str().unwrap());

    Ok(())
}
#[derive(Debug)]
pub enum Error {
    Sqlx(sqlx::Error),
}
impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self::Sqlx(value)
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sqlx(e) => write!(f, "Sqlx error: {}", e),
        }
    }
}
impl std::error::Error for Error {}
