use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

use crate::{
    core::{
        app_builder::{AppBuilder, Missing},
        error::Error,
        images_processor::{ImagesProcessor, RayonImagesProcessor},
        result_parser::{ResultParser, SqliteResultParser},
        state::AppState,
    },
    hashing_methods,
    image_hash::{self, HashingMethods},
    image_modify::{self, Modifications},
    image_parse::Images,
    matching::match_process::{PipelineRunner, SqliteRunner},
    modified_images,
};

pub struct App {
    imgs_path: PathBuf,

    images_processor: Box<dyn ImagesProcessor>,
    results_parser: Box<dyn ResultParser>,
    match_process: Box<dyn PipelineRunner>,

    state: AppState,
}
impl App {
    pub fn new(
        path: &Path,
        results_parser: Box<dyn ResultParser>,
        images_processor: Box<dyn ImagesProcessor>,
        match_process: Box<dyn PipelineRunner>,
        modifications: Modifications,
        hashing_methods: HashingMethods,
    ) -> Self {
        let state = AppState::new(hashing_methods, modifications);
        Self {
            imgs_path: path.to_path_buf(),
            images_processor,
            results_parser,
            match_process,
            state,
        }
    }
    pub fn builder() -> AppBuilder<Missing, Missing, Missing, Missing, Missing> {
        AppBuilder::new()
    }
    pub fn state(&self) -> &AppState {
        &self.state
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
        let res = self.images_processor.run(
            images,
            &self.state.modifications(),
            &self.state.hashing_methods(),
        );

        tracing::info!("sending results to db");
        self.results_parser
            .parse(
                res,
                &self.state.modifications(),
                &self.state.hashing_methods(),
            )
            .await?;
        self.match_process
            .run(&self.state.hashing_methods())
            .await?;

        Ok(())
    }

    pub async fn try_default() -> Result<Self, Error> {
        // Choosing what method to process images with.
        let processor = Box::new(RayonImagesProcessor::default());

        let pool = SqlitePool::connect_with(
            SqliteConnectOptions::from_str("sqlite:data.db")?
                .create_if_missing(true)
                .pragma("cache_size", "200000"),
        )
        .await?;

        let parser = Box::new(SqliteResultParser::new(pool.clone()));
        let match_process = Box::new(SqliteRunner::new(pool.clone()));

        let example_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("images");

        tracing::info!("Using path {:?}", example_dir);

        // What modifications that should be used.
        let modifications =
            modified_images![image_modify::Angle::Rot90, image_modify::Blur::new(0.9),];

        // What hashing methods that should be used.
        let hashing_methods = hashing_methods![
            image_hash::AverageHash::new(8),
            image_hash::AverageHash::new(16),
            image_hash::AverageHash::new(64),
            image_hash::AverageHash::new(256),
            image_hash::VertGradient::new(8),
            image_hash::VertGradient::new(16),
            image_hash::VertGradient::new(64),
            image_hash::VertGradient::new(256)
        ];

        let app = App::builder()
            .images_path(&example_dir)
            .images_processor(processor)
            .results_parser(parser)
            .match_process(match_process)
            .modifications(modifications)
            .hashing_methods(hashing_methods)
            .finish();
        Ok(app)
    }
}
