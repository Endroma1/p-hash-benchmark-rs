use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::{
    image_hash::HashingMethods,
    matching::{
        error::Error,
        fetcher::{ResultsFetcher, SqliteFetcher},
        processor::{MatchProcessor, ThreadedUniquePairMatcher},
        result_parser::{MatchResultParser, RcSqliteResultParser},
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
    pub async fn execute(&self, hashing_methods: &HashingMethods) -> Result<(), E> {
        for (id, _) in hashing_methods.iter().enumerate() {
            let fetch_res = self.fetcher.fetch(id as u16).await?;
            let processor_res = self.processor.process(fetch_res)?;
            self.parser.parse(processor_res).await?;
        }
        Ok(())
    }
}
/// Pipelinerunner should run for every hashing method. Matching across methods would be useless
#[async_trait]
pub trait PipelineRunner {
    async fn run(&self, hashing_methods: &HashingMethods) -> Result<(), Error>;
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
    fn run<'life0, 'life1, 'async_trait>(
        &'life0 self,
        hashing_methods: &'life1 HashingMethods,
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
        Self: 'async_trait,
    {
        Box::pin(async move {
            let fetcher = Box::new(SqliteFetcher::new(self.pool.clone()));
            let processor = Box::new(ThreadedUniquePairMatcher::default());
            let parser = Box::new(RcSqliteResultParser::from_pool(self.pool.clone()));

            let pipeline = MatchPipeline::new(fetcher, processor, parser);

            pipeline.execute(hashing_methods).await?;
            Ok(())
        })
    }
}
