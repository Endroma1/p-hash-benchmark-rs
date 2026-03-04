use std::path::{Path, PathBuf};

use crate::{
    core::{
        app_builder::{AppBuilder, Missing},
        error::Error,
        images_processor::ImagesProcessor,
        result_parser::ResultParser,
    },
    image_hash::HashingMethods,
    image_modify::Modifications,
    image_parse::Images,
    matching::match_process::PipelineRunner,
};

pub struct App {
    imgs_path: PathBuf,

    images_processor: Box<dyn ImagesProcessor>,
    results_parser: Box<dyn ResultParser>,
    match_process: Box<dyn PipelineRunner>,

    modifications: Modifications,
    hashing_methods: HashingMethods,
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
        Self {
            imgs_path: path.to_path_buf(),
            images_processor,
            results_parser,
            match_process,
            modifications,
            hashing_methods,
        }
    }
    pub fn builder() -> AppBuilder<Missing, Missing, Missing, Missing, Missing> {
        AppBuilder::new()
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
        let res = self
            .images_processor
            .run(images, &self.modifications, &self.hashing_methods);

        tracing::info!("sending results to db");
        self.results_parser
            .parse(res, &self.modifications, &self.hashing_methods)
            .await?;
        self.match_process.run(&self.hashing_methods).await?;

        Ok(())
    }
}
