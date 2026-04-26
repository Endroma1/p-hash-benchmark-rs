use async_trait::async_trait;
use chrono::Utc;
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use sqlx::SqlitePool;

use crate::{
    core::{error::Error, state::AppProcessResult},
    db::DB,
    image_hash::{HashingMethods, SelectedHashingMethods},
    image_modify::{Modifications, SelectedModifications},
};

#[async_trait]
pub trait ResultParser: Send + Sync {
    async fn parse(
        &self,
        results: AppProcessResult,
        modifications: &SelectedModifications,
        hashing_methods: &SelectedHashingMethods,
    ) -> Result<(), Error>;
}

pub struct SqliteResultParser {
    pool: SqlitePool,
}
impl SqliteResultParser {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool: pool }
    }
}
impl ResultParser for SqliteResultParser {
    fn parse<'life0, 'life1, 'life2, 'async_trait>(
        &'life0 self,
        results: AppProcessResult,
        modifications: &'life1 SelectedModifications,
        hashing_methods: &'life2 SelectedHashingMethods,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<(), Error>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            DB::create_db().await?;
            send_hashes_to_db(&self.pool, hashing_methods).await?;
            send_modifications_to_db(&self.pool, modifications).await?;

            let run_id = create_run(&self.pool).await?;
            create_program(&self.pool, run_id).await?;

            let style = ProgressStyle::with_template(
                "[{elapsed_precise} | {eta_precise}] Sending results to DB: {pos:>7}/{len:7} {percent}%",
            )
            .unwrap()
            .progress_chars("##-");

            let mut tx = self.pool.begin().await?;
            for (id, img) in results
                .images()
                .iter()
                .enumerate()
                .progress_with_style(style)
            {
                let path_str = img.get_path().to_string_lossy().to_string();
                sqlx::query(
                    "
                INSERT INTO images (id, path, user) VALUES (?, ?, ?) ON CONFLICT(id) DO NOTHING;
                    ",
                )
                .bind(id as u32)
                .bind(path_str)
                .bind(img.get_user())
                .execute(&mut *tx)
                .await?;

                sqlx::query(
                    "
                    INSERT INTO run_images (run_id, image_id) VALUES (?, ?);
                    ",
                )
                .bind(run_id)
                .bind(id as u32)
                .execute(&mut *tx)
                .await?;
            }
            tx.commit().await?;
            results.phash_results().send_to_db(&self.pool).await?;
            Ok(())
        })
    }
}
async fn create_program(pool: &SqlitePool, run_id: i64) -> Result<(), Error> {
    sqlx::query(
        "
        INSERT OR REPLACE INTO program (id, run_id) VALUES (0, ?);
        ",
    )
    .bind(run_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn create_run(pool: &SqlitePool) -> Result<i64, Error> {
    let now = Utc::now();
    let res = sqlx::query(
        "
        INSERT INTO runs (timestamp) VALUES (?);
        ",
    )
    .bind(now.timestamp_millis())
    .execute(pool)
    .await?;
    Ok(res.last_insert_rowid())
}

async fn send_modifications_to_db<'a>(
    pool: &SqlitePool,
    modifications: &SelectedModifications<'a>,
) -> Result<(), Error> {
    let mut tx = pool.begin().await?;
    for (i, modification) in modifications.iter().enumerate() {
        let name = modification.name();
        sqlx::query(
            "
            INSERT INTO modifications (id, name) VALUES (?,?) ON CONFLICT(id) DO NOTHING;
            ",
        )
        .bind(i as u32)
        .bind(name)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

async fn send_hashes_to_db<'a>(
    pool: &SqlitePool,
    hashing_methods: &SelectedHashingMethods<'a>,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    for (id, obj) in hashing_methods.iter().enumerate() {
        sqlx::query(
            "
                INSERT INTO hashing_methods (id, name) VALUES (?,?) ON CONFLICT(id) DO NOTHING;
                ",
        )
        .bind(id as u32)
        .bind(obj.name())
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}
