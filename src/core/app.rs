use std::{
    env,
    path::{Path, PathBuf},
    str::FromStr,
    sync::OnceLock,
};

use crate::{
    core::{
        error::Error,
        images_processor::{ImagesProcessor, RayonImagesProcessor},
        result_parser::{ResultParser, SqliteResultParser},
    },
    img_hash::HashingMethods,
    img_mod::Modifications,
    img_proc::Images,
};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

#[derive(Default)]
pub struct AppConfig {
    imgs_path: Option<PathBuf>,
    images_processor: Option<Box<dyn ImagesProcessor>>,
    results_parser: Option<Box<dyn ResultParser>>,
}
impl AppConfig {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn images_processor(&mut self, processor: Box<dyn ImagesProcessor>) -> &mut Self {
        self.images_processor = Some(processor);
        self
    }
    pub fn results_parser(&mut self, parser: Box<dyn ResultParser>) -> &mut Self {
        self.results_parser = Some(parser);
        self
    }
    pub fn images_path(&mut self, path: &Path) -> &mut Self {
        self.imgs_path = Some(path.to_path_buf());
        self
    }
    pub async fn finish(self) -> Result<App, Error> {
        let path = match self.imgs_path {
            Some(p) => p,
            None => {
                let home_dir = env::home_dir().ok_or(Error::HomeDirNotFound)?;
                home_dir.join(".local/share/p-hash/images")
            }
        };
        let images_processor = match self.images_processor {
            Some(p) => p,
            None => Box::new(RayonImagesProcessor::default()),
        };
        let results_parser = match self.results_parser {
            Some(p) => p,
            None => {
                let pool = SqlitePool::connect_with(
                    SqliteConnectOptions::from_str("sqlite:data.db")?.create_if_missing(true),
                )
                .await?;
                Box::new(SqliteResultParser::new(pool))
            }
        };
        Ok(App::new(&path, results_parser, images_processor))
    }
}

pub struct App {
    imgs_path: PathBuf,

    images_processor: Box<dyn ImagesProcessor>,
    results_parser: Box<dyn ResultParser>,
}
impl App {
    pub fn new(
        path: &Path,
        results_parser: Box<dyn ResultParser>,
        images_processor: Box<dyn ImagesProcessor>,
    ) -> Self {
        Self {
            imgs_path: path.to_path_buf(),
            images_processor,
            results_parser,
        }
    }
    pub async fn try_default() -> Result<Self, Error> {
        AppConfig::default().finish().await
    }
    pub async fn run(&self) -> Result<(), Error> {
        let images = Images::from_path(self.imgs_path.to_path_buf());
        let images = images
            .filter_map(|r| {
                if let Err(e) = r.as_ref() {
                    log::warn!("An image failed to process: {}", e);
                }
                r.ok()
            })
            .collect();

        log::info!("starting image hashing");
        let res = self.images_processor.run(images);

        log::info!("sending results to db");
        self.results_parser.parse(res).await?;
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
