use std::{
    env::home_dir,
    path::{Path, PathBuf},
    str::FromStr,
    sync::OnceLock,
};

use crate::{
    core::{
        app_builder::{AppBuilder, Missing},
        error::Error,
        images_processor::{ImagesProcessor, RayonImagesProcessor},
        result_parser::{ResultParser, SqliteResultParser},
    },
    img_hash::HashingMethods,
    img_mod::Modifications,
    img_proc::Images,
    matching::match_process::{PipelineRunner, SqliteRunner},
};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

pub struct App {
    imgs_path: PathBuf,

    images_processor: Box<dyn ImagesProcessor>,
    results_parser: Box<dyn ResultParser>,
    match_process: Box<dyn PipelineRunner>,
}
impl App {
    pub fn new(
        path: &Path,
        results_parser: Box<dyn ResultParser>,
        images_processor: Box<dyn ImagesProcessor>,
        match_process: Box<dyn PipelineRunner>,
    ) -> Self {
        Self {
            imgs_path: path.to_path_buf(),
            images_processor,
            results_parser,
            match_process,
        }
    }
    pub fn builder() -> AppBuilder<Missing, Missing, Missing> {
        AppBuilder::new()
    }
    /// Runs with rayon for processing images and uses sqlite db to store results
    pub async fn try_default() -> Result<Self, Error> {
        let processor = Box::new(RayonImagesProcessor::default());

        let pool = SqlitePool::connect_with(
            SqliteConnectOptions::from_str("sqlite:data.db")?
                .create_if_missing(true)
                .pragma("cache_size", "200000"),
        )
        .await?;
        let parser = Box::new(SqliteResultParser::new(pool.clone()));
        let match_process = Box::new(SqliteRunner::new(pool.clone()));
        let mut home = home_dir().unwrap();
        home.push(".local/share/p-hash/images");
        let app = AppBuilder::new()
            .images_path(&home)
            .images_processor(processor)
            .results_parser(parser)
            .match_process(match_process)
            .finish();

        Ok(app)
    }
    pub async fn run(&self) -> Result<(), Error> {
        let images = Images::from_path(self.imgs_path.to_path_buf());
        let images = images
            .filter_map(|r| {
                if let Err(e) = r.as_ref() {
                    tracing::warn!("An image failed to process: {}", e);
                }
                r.ok()
            })
            .collect();

        tracing::info!("starting image hashing");
        let res = self.images_processor.run(images);

        tracing::info!("sending results to db");
        self.results_parser.parse(res).await?;
        self.match_process.run().await?;

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
