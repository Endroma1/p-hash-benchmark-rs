use std::{
    env::home_dir,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use crate::{
    core::{
        app::App,
        images_processor,
        result_parser::{self},
    },
    image_hash::HashingMethods,
    image_modify::Modifications,
    matching::match_process::PipelineRunner,
};

#[derive(Default)]
pub struct AppBuilder<P, R, M, MO, HA> {
    imgs_path: PathBuf,
    images_processor: Option<Box<dyn images_processor::ImagesProcessor>>,
    results_parser: Option<Box<dyn result_parser::ResultParser>>,
    match_process: Option<Box<dyn PipelineRunner>>,

    modifications: Option<Modifications>,
    hashing_methods: Option<HashingMethods>,

    _imgs_path: PhantomData<P>,
    _results_parser: PhantomData<R>,
    _match_process: PhantomData<M>,

    _modifications: PhantomData<MO>,
    _hashing_methods: PhantomData<HA>,
}
pub struct Missing;
pub struct ImagesProcessor;
pub struct ResultParser;
pub struct MatchProcess;
pub struct ModificationsPresent;
pub struct HashingMethodsPresent;
impl AppBuilder<Missing, Missing, Missing, Missing, Missing> {
    pub fn new() -> Self {
        let mut home = home_dir().unwrap();
        home.push(".local/share/p-hash/images");
        Self {
            imgs_path: home,
            images_processor: None,
            results_parser: None,
            match_process: None,
            hashing_methods: None,
            modifications: None,

            _imgs_path: PhantomData,
            _results_parser: PhantomData,
            _match_process: PhantomData,
            _hashing_methods: PhantomData,
            _modifications: PhantomData,
        }
    }
}
impl<R, M, MO, HA> AppBuilder<Missing, R, M, MO, HA> {
    pub fn images_processor(
        self,
        processor: Box<dyn images_processor::ImagesProcessor>,
    ) -> AppBuilder<ImagesProcessor, R, M, MO, HA> {
        AppBuilder {
            imgs_path: self.imgs_path,
            images_processor: Some(processor),
            results_parser: self.results_parser,
            match_process: self.match_process,
            hashing_methods: self.hashing_methods,
            modifications: self.modifications,

            _imgs_path: PhantomData,
            _results_parser: PhantomData,
            _match_process: PhantomData,
            _hashing_methods: PhantomData,
            _modifications: PhantomData,
        }
    }
}
impl<P, M, MO, HA> AppBuilder<P, Missing, M, MO, HA> {
    pub fn results_parser(
        self,
        parser: Box<dyn result_parser::ResultParser>,
    ) -> AppBuilder<P, ResultParser, M, MO, HA> {
        AppBuilder {
            imgs_path: self.imgs_path,
            images_processor: self.images_processor,
            results_parser: Some(parser),
            match_process: self.match_process,
            hashing_methods: self.hashing_methods,
            modifications: self.modifications,

            _imgs_path: PhantomData,
            _results_parser: PhantomData,
            _match_process: PhantomData,
            _hashing_methods: PhantomData,
            _modifications: PhantomData,
        }
    }
}
impl<R, P, MO, HA> AppBuilder<R, P, Missing, MO, HA> {
    pub fn match_process(
        self,
        matcher: Box<dyn PipelineRunner>,
    ) -> AppBuilder<R, P, MatchProcess, MO, HA> {
        AppBuilder {
            imgs_path: self.imgs_path,
            images_processor: self.images_processor,
            results_parser: self.results_parser,
            match_process: Some(matcher),
            hashing_methods: self.hashing_methods,
            modifications: self.modifications,

            _imgs_path: PhantomData,
            _results_parser: PhantomData,
            _match_process: PhantomData,
            _hashing_methods: PhantomData,
            _modifications: PhantomData,
        }
    }
}
impl<R, P, M, MO> AppBuilder<R, P, M, MO, Missing> {
    pub fn hashing_methods(
        self,
        hashing_methods: HashingMethods,
    ) -> AppBuilder<R, P, M, MO, HashingMethodsPresent> {
        AppBuilder {
            imgs_path: self.imgs_path,
            images_processor: self.images_processor,
            results_parser: self.results_parser,
            match_process: self.match_process,
            modifications: self.modifications,
            hashing_methods: Some(hashing_methods),

            _imgs_path: PhantomData,
            _results_parser: PhantomData,
            _match_process: PhantomData,
            _hashing_methods: PhantomData,
            _modifications: PhantomData,
        }
    }
}
impl<R, P, M, HA> AppBuilder<R, P, M, Missing, HA> {
    pub fn modifications(
        self,
        modifications: Modifications,
    ) -> AppBuilder<R, P, M, ModificationsPresent, HA> {
        AppBuilder {
            imgs_path: self.imgs_path,
            images_processor: self.images_processor,
            results_parser: self.results_parser,
            match_process: self.match_process,
            hashing_methods: self.hashing_methods,
            modifications: Some(modifications),

            _imgs_path: PhantomData,
            _results_parser: PhantomData,
            _match_process: PhantomData,
            _hashing_methods: PhantomData,
            _modifications: PhantomData,
        }
    }
}
impl
    AppBuilder<
        ImagesProcessor,
        ResultParser,
        MatchProcess,
        ModificationsPresent,
        HashingMethodsPresent,
    >
{
    pub fn finish(self) -> App {
        App::new(
            &self.imgs_path,
            self.results_parser.unwrap(),
            self.images_processor.unwrap(),
            self.match_process.unwrap(),
            self.modifications.unwrap(),
            self.hashing_methods.unwrap(),
        )
    }
}
impl<R, P, M, MO, HA> AppBuilder<R, P, M, MO, HA> {
    pub fn images_path(mut self, path: &Path) -> Self {
        self.imgs_path = path.to_path_buf();
        self
    }
}
