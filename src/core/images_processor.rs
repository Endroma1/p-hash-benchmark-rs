use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crossbeam::channel::unbounded;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use sqlx::SqlitePool;

use crate::{
    core::{
        error::Error,
        image_parser::{AppProcParser, ImageParser},
        state::{AppProcessResult, Hashes, Images},
    },
    image_hash::HashingMethods,
    image_modify::{Modifications, ModifiedImages},
    image_parse::Image,
};

/// Parses input images given by img_proc::Image data struct.
pub trait ImagesProcessor: Send + Sync {
    fn run(
        &self,
        images: Vec<Image>,
        modifications: &Modifications,
        hashing_methods: &HashingMethods,
    ) -> AppProcessResult;
}
pub struct RayonImagesProcessor {
    image_parser: Box<dyn ImageParser>,
}
impl ImagesProcessor for RayonImagesProcessor {
    fn run(
        &self,
        images: Vec<Image>,
        modifications: &Modifications,
        hashing_methods: &HashingMethods,
    ) -> AppProcessResult {
        let (s, r) = unbounded();
        let style = ProgressStyle::with_template(
            "[{elapsed_precise} | {eta_precise}] Processing images: {pos:>7}/{len:7} {percent}%",
        )
        .unwrap()
        .progress_chars("##-");

        images
            .par_iter()
            .progress_with(ProgressBar::new(images.len() as u64).with_style(style))
            .enumerate()
            .for_each(move |(id, image)| {
                let res = self
                    .image_parser
                    .run(image, id as u32, modifications, hashing_methods);

                let phash_res = match res {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::warn!(
                            "Could not parse image {:?}. Error: {}",
                            image.get_path(),
                            e
                        );
                        return;
                    }
                };

                if let Err(e) = s.send((id, phash_res)) {
                    tracing::warn!("failed to send result through channel, err: {}", e);
                }
            });

        let mut results = PHashResults::default();
        while let Ok((id, r)) = r.recv() {
            results.insert(id as u32, r);
        }

        AppProcessResult::new(Images::from(images), results)
    }
}
impl Default for RayonImagesProcessor {
    fn default() -> Self {
        let image_parser = Box::new(AppProcParser::default());
        Self { image_parser }
    }
}

#[derive(Default)]
pub struct PHashResults {
    results: HashMap<u32, PHashResult>,
}
impl Deref for PHashResults {
    type Target = HashMap<u32, PHashResult>;
    fn deref(&self) -> &Self::Target {
        &self.results
    }
}
impl DerefMut for PHashResults {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.results
    }
}
impl PHashResults {
    pub async fn send_to_db(&self, pool: &SqlitePool) -> Result<(), Error> {
        let mut tx = pool.begin().await?;
        for (id, res) in &self.results {
            for hash in res.hashes.into_iter() {
                let img = res.mod_imgs.get_img(hash.mod_img_id())?;

                tracing::debug!(
                    "Inserting mod_img with images_id: {}, modification_id: {}",
                    id,
                    img.get_mod_id()
                );

                let res: (i64,) = sqlx::query_as(
                    "
                INSERT INTO modified_images ( image_id, modification_id) VALUES (?,?) 
                ON CONFLICT (image_id, modification_id) 
                DO UPDATE SET modification_id = excluded.modification_id
                RETURNING id;
                ",
                )
                .bind(id)
                .bind(img.get_mod_id())
                .fetch_one(&mut *tx)
                .await?;

                let mod_img_id = res.0;

                sqlx::query(
                    "
                INSERT INTO hashes (hash, mod_image_id, hashing_method_id) VALUES (?,?,?) ON CONFLICT DO NOTHING;
                ",
                )
                .bind(hash.hash().hash().to_string())
                .bind(mod_img_id)
                .bind(hash.hash().hashing_method_id())
                .execute(&mut *tx)
                .await?;
            }
        }
        tx.commit().await?;
        Ok(())
    }
}
#[derive(Default)]
pub struct PHashResult {
    mod_imgs: ModifiedImages,
    hashes: Hashes,
}

impl PHashResult {
    pub fn mod_imgs(&self) -> &ModifiedImages {
        &self.mod_imgs
    }
    pub fn hashes(&self) -> &Hashes {
        &self.hashes
    }
    pub fn mod_imgs_mut(&mut self) -> &mut ModifiedImages {
        &mut self.mod_imgs
    }
    pub fn hashes_mut(&mut self) -> &mut Hashes {
        &mut self.hashes
    }
    pub fn new(mod_imgs: ModifiedImages, hashes: Hashes) -> Self {
        Self { mod_imgs, hashes }
    }
    pub fn set_mod_imgs(&mut self, mod_imgs: ModifiedImages) {
        self.mod_imgs = mod_imgs
    }
    pub fn set_hashes(&mut self, hashes: Hashes) {
        self.hashes = hashes
    }
}
