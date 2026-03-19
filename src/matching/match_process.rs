use std::{collections::HashMap, thread};

use async_trait::async_trait;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use sqlx::SqlitePool;

use crate::{
    image_hash::HashingMethods,
    matching::{
        error::Error,
        fetcher::{ResultsFetcher, SqliteFetcher},
        processor::{MatchProcessor, MultiThreadedUniquePairMatcher, ThreadedUniquePairMatcher},
        result_parser::{MatchResultParser, RcSqliteResultParser},
        state::{Component, MatchState, Message},
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
        let state = MatchState::new();

        indicatif_view(state.clone());

        state.set(Component::Fetcher, hashing_methods.len() as u32);

        for (id, _) in hashing_methods.iter().enumerate() {
            state.update(Component::Fetcher, 1);

            let fetch_res = self.fetcher.fetch(id as u16, state.clone()).await?;
            let processor_res = self.processor.process(fetch_res, state.clone())?;
            self.parser.parse(processor_res, state.clone()).await?;
        }
        Ok(())
    }
}

/// Spawn a view thread that uses indicatif as frontend to MatchState
fn indicatif_view(rx_state: MatchState) {
    let style = ProgressStyle::with_template(
        "[{elapsed_precise} | {eta_precise}] {msg}: {pos:>7}/{len:7} {percent}%",
    )
    .unwrap()
    .progress_chars("##-");

    thread::spawn(move || {
        let multi = MultiProgress::new();
        let mut progressbars = HashMap::new();
        while let Ok(m) = rx_state.get() {
            match m {
                Message::Set { component, total } => {
                    let pb = ProgressBar::new(total as u64)
                        .with_style(style.clone())
                        .with_message(component.to_string());
                    multi.add(pb.clone());
                    progressbars.insert(component, pb);
                }
                Message::Update { component, delta } => {
                    progressbars.get(&component).map(|pb| pb.inc(delta as u64));
                }
            }
        }
    });
}
#[derive(Debug)]
pub struct StateQuit;

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
            let processor = Box::new(MultiThreadedUniquePairMatcher::default());
            let parser = Box::new(RcSqliteResultParser::from_pool(self.pool.clone()));

            let pipeline = MatchPipeline::new(fetcher, processor, parser);

            pipeline.execute(hashing_methods).await?;
            Ok(())
        })
    }
}
