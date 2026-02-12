use std::{path::Path, str::FromStr, sync::OnceLock};

use crate::{
    core::{
        app_proc::AppProcess,
        error::Error,
        images_processor::{ImagesProcessor, RayonImagesProcessor},
        state::PHashResult,
    },
    db::DB,
    img_hash::{HashingMethodID, HashingMethods, hash_images},
    img_mod::{ModificationID, Modifications, modify_image, open_image},
    img_proc::{self, Image, ImageReadMessage, Images},
};
use sqlx::{
    SqlitePool,
    sqlite::{self, SqliteConnectOptions, SqlitePoolOptions},
};
pub struct App {
    images: Vec<img_proc::Image>,
    modifications: &'static Modifications,
    hashing_methods: &'static HashingMethods,

    images_processor: Box<dyn ImagesProcessor>,
}
impl App {
    pub fn new(path: &Path) -> Self {
        let images = Images::from_path(path.to_path_buf());
        let images = images
            .filter_map(|r| {
                if let Err(e) = r.as_ref() {
                    log::warn!("A image failed to process: {}", e);
                }
                r.ok()
            })
            .collect();

        let modifications = get_modifications();
        let hashing_methods = get_hashing_methods();
        let images_processor = Box::new(RayonImagesProcessor::default());
        Self {
            images,
            modifications,
            hashing_methods,
            images_processor,
        }
    }
    pub async fn run(&self) -> Result<(), Error> {
        let pool = SqlitePool::connect_with(
            SqliteConnectOptions::from_str("sqlite:data.db")?.create_if_missing(true),
        )
        .await?;

        log::info!("starting image hashing");
        let res = self.images_processor.run(&self.images);

        log::info!("sending results to db");
        self.send_to_db(res, &pool).await?;
        Ok(())
    }
    async fn send_to_db(&self, results: Vec<PHashResult>, pool: &SqlitePool) -> Result<(), Error> {
        log::debug!("sending modifications to db");
        self.modifications.send_to_db(pool).await?;
        log::debug!("sending hashing_methods to db");
        self.hashing_methods.send_to_db(pool).await?;

        log::debug!("results len : {}", results.len());
        log::debug!("sending results to db");
        for result in results {
            let image = self
                .images
                .get(result.img_id() as usize)
                .ok_or(Error::ImageNotFound {
                    id: result.img_id() as usize,
                })?;

            let path_str = image.get_path().to_string_lossy().to_string();
            log::debug!("inserting image with id {}", result.img_id());
            sqlx::query(
                "
            INSERT INTO images (id, path, user) VALUES (?, ?, ?) ON CONFLICT(id) DO NOTHING;
            ",
            )
            .bind(result.img_id())
            .bind(path_str)
            .bind(image.get_user())
            .execute(pool)
            .await?;

            result.send_to_db(pool).await?;
        }
        Ok(())
    }
}

pub fn get_modifications() -> &'static Modifications {
    static MODIFICATIONS: OnceLock<Modifications> = OnceLock::new();
    MODIFICATIONS.get_or_init(|| Modifications::new())
}
pub fn get_hashing_methods() -> &'static HashingMethods {
    static HASHING_METHODS: OnceLock<HashingMethods> = OnceLock::new();
    HASHING_METHODS.get_or_init(|| HashingMethods::new())
}
