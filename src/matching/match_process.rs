use async_trait::async_trait;
use indicatif::ProgressIterator;
use sqlx::SqlitePool;

use crate::{
    core::app::get_hashing_methods,
    matching::{
        error::Error,
        fetcher::{ResultsFetcher, SqliteFetcher},
        processor::{MatchProcessor, UniquePairMatcher},
        result_parser::{MatchResultParser, SqliteResultParser},
    },
};

struct MatchPipeline<E, M, R> {
    fetcher: Box<dyn ResultsFetcher<Error = E, Output = R>>,
    processor: Box<dyn MatchProcessor<Input = R, Output = M, Error = E>>,
    parser: Box<dyn MatchResultParser<Result = M, Error = E>>,
}
impl<E, M, R> MatchPipeline<E, M, R> {
    pub fn new(
        fetcher: Box<dyn ResultsFetcher<Error = E, Output = R>>,
        processor: Box<dyn MatchProcessor<Input = R, Output = M, Error = E>>,
        parser: Box<dyn MatchResultParser<Result = M, Error = E>>,
    ) -> Self {
        Self {
            fetcher,
            processor,
            parser,
        }
    }
    pub async fn execute(&self) -> Result<(), E> {
        for id in get_hashing_methods().get_keys().iter().progress() {
            let fetch_res = self.fetcher.fetch(*id).await?;
            let processor_res = self.processor.process(fetch_res)?;
            self.parser.parse(processor_res).await?;
        }
        Ok(())
    }
}
#[async_trait]
pub trait PipelineRunner {
    async fn run(&self) -> Result<(), Error>;
}

pub struct SqliteRunner {
    pool: SqlitePool,
}
impl SqliteRunner {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}
impl PipelineRunner for SqliteRunner {
    fn run<'life0, 'async_trait>(
        &'life0 self,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<(), Error>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let fetcher = Box::new(SqliteFetcher::new(self.pool.clone()));
            let processor = Box::new(UniquePairMatcher::default());
            let parser = Box::new(SqliteResultParser::from_pool(self.pool.clone()));

            let pipeline = MatchPipeline::new(fetcher, processor, parser);

            pipeline.execute().await?;
            Ok(())
        })
    }
}
