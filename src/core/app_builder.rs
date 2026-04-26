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
pub struct AppBuilder<I, P, R, M, MO, HA> {
    imgs_path: Option<PathBuf>,
    images_processor: Option<Box<dyn images_processor::ImagesProcessor>>,
    results_parser: Option<Box<dyn result_parser::ResultParser>>,
    match_process: Option<Box<dyn PipelineRunner>>,

    modifications: Option<Modifications>,
    hashing_methods: Option<HashingMethods>,

    _imgs_path: PhantomData<I>,
    _images_processor: PhantomData<P>,
    _results_parser: PhantomData<R>,
    _match_process: PhantomData<M>,

    _modifications: PhantomData<MO>,
    _hashing_methods: PhantomData<HA>,
}
pub struct Missing;
pub struct ImagesPath;
pub struct ImagesProcessor;
pub struct ResultParser;
pub struct MatchProcess;
pub struct ModificationsPresent;
pub struct HashingMethodsPresent;
impl AppBuilder<Missing, Missing, Missing, Missing, Missing, Missing> {
    pub fn new() -> Self {
        let mut home = home_dir().unwrap();
        home.push(".local/share/p-hash/images");
        Self {
            imgs_path: None,
            images_processor: None,
            results_parser: None,
            match_process: None,
            hashing_methods: None,
            modifications: None,

            _imgs_path: PhantomData,
            _images_processor: PhantomData,
            _results_parser: PhantomData,
            _match_process: PhantomData,
            _hashing_methods: PhantomData,
            _modifications: PhantomData,
        }
    }
}
impl< P, R, M, MO, HA> AppBuilder<Missing, P,R, M, MO, HA> {
    pub fn imgs_path(
        self,
        imgs_path: impl Into<PathBuf>
    ) -> AppBuilder<ImagesPath, P,R, M, MO, HA> {
        AppBuilder {
            imgs_path: Some(imgs_path.into()),
            images_processor: self.images_processor,
            results_parser: self.results_parser,
            match_process: self.match_process,
            hashing_methods: self.hashing_methods,
            modifications: self.modifications,

            _imgs_path: PhantomData,
            _images_processor: PhantomData,
            _results_parser: PhantomData,
            _match_process: PhantomData,
            _hashing_methods: PhantomData,
            _modifications: PhantomData,
        }

    }
}

impl<I,R, M, MO, HA> AppBuilder<I, Missing, R, M, MO, HA> {
    pub fn images_processor(
        self,
        processor: Box<dyn images_processor::ImagesProcessor>,
    ) -> AppBuilder<I, ImagesProcessor, R, M, MO, HA> {
        AppBuilder {
            imgs_path: self.imgs_path,
            images_processor: Some(processor),
            results_parser: self.results_parser,
            match_process: self.match_process,
            hashing_methods: self.hashing_methods,
            modifications: self.modifications,

            _imgs_path: PhantomData,
            _images_processor: PhantomData,
            _results_parser: PhantomData,
            _match_process: PhantomData,
            _hashing_methods: PhantomData,
            _modifications: PhantomData,
        }
    }
}
impl<I, P, M, MO, HA> AppBuilder<I, P, Missing, M, MO, HA> {
    pub fn results_parser(
        self,
        parser: Box<dyn result_parser::ResultParser>,
    ) -> AppBuilder<I, P, ResultParser, M, MO, HA> {
        AppBuilder {
            imgs_path: self.imgs_path,
            images_processor: self.images_processor,
            results_parser: Some(parser),
            match_process: self.match_process,
            hashing_methods: self.hashing_methods,
            modifications: self.modifications,

            _imgs_path: PhantomData,
            _images_processor: PhantomData,
            _results_parser: PhantomData,
            _match_process: PhantomData,
            _hashing_methods: PhantomData,
            _modifications: PhantomData,
        }
    }
}
impl<I,R, P, MO, HA> AppBuilder<I, R, P, Missing, MO, HA> {
    pub fn match_process(
        self,
        matcher: Box<dyn PipelineRunner>,
    ) -> AppBuilder<I, R, P, MatchProcess, MO, HA> {
        AppBuilder {
            imgs_path: self.imgs_path,
            images_processor: self.images_processor,
            results_parser: self.results_parser,
            match_process: Some(matcher),
            hashing_methods: self.hashing_methods,
            modifications: self.modifications,

            _imgs_path: PhantomData,
            _images_processor: PhantomData,
            _results_parser: PhantomData,
            _match_process: PhantomData,
            _hashing_methods: PhantomData,
            _modifications: PhantomData,
        }
    }
}
impl<I, R, P, M, MO> AppBuilder<I,R, P, M, MO, Missing> {
    pub fn hashing_methods(
        self,
        hashing_methods: HashingMethods,
    ) -> AppBuilder<I, R, P, M, MO, HashingMethodsPresent> {
        AppBuilder {
            imgs_path: self.imgs_path,
            images_processor: self.images_processor,
            results_parser: self.results_parser,
            match_process: self.match_process,
            modifications: self.modifications,
            hashing_methods: Some(hashing_methods),

            _imgs_path: PhantomData,
            _images_processor: PhantomData,
            _results_parser: PhantomData,
            _match_process: PhantomData,
            _hashing_methods: PhantomData,
            _modifications: PhantomData,
        }
    }
}
impl<I, R, P, M, HA> AppBuilder<I, R, P, M, Missing, HA> {
    pub fn modifications(
        self,
        modifications: Modifications,
    ) -> AppBuilder<I, R, P, M, ModificationsPresent, HA> {
        AppBuilder {
            imgs_path: self.imgs_path,
            images_processor: self.images_processor,
            results_parser: self.results_parser,
            match_process: self.match_process,
            hashing_methods: self.hashing_methods,
            modifications: Some(modifications),

            _imgs_path: PhantomData,
            _images_processor: PhantomData,
            _results_parser: PhantomData,
            _match_process: PhantomData,
            _hashing_methods: PhantomData,
            _modifications: PhantomData,
        }
    }
}
impl
    AppBuilder<
        ImagesPath,
        ImagesProcessor,
        ResultParser,
        MatchProcess,
        ModificationsPresent,
        HashingMethodsPresent,
    >
{
    pub fn finish(self) -> App {
        App::new(
            &self.imgs_path.unwrap(),
            self.results_parser.unwrap(),
            self.images_processor.unwrap(),
            self.match_process.unwrap(),
            self.modifications.unwrap(),
            self.hashing_methods.unwrap(),
        )
    }
}
