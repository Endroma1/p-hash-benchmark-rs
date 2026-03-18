use std::{collections::HashMap, sync::mpsc::Sender, thread};

use async_trait::async_trait;
use crossbeam::channel::{Receiver, bounded};
use enum_iterator::all;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use sqlx::SqlitePool;

use crate::{
    image_hash::HashingMethods,
    matching::{
        error::Error,
        fetcher::{ResultsFetcher, SqliteFetcher},
        processor::{MatchProcessor, ThreadedUniquePairMatcher},
        result_parser::{MatchResultParser, RcSqliteResultParser},
        state::{Component, Message},
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
        let (state_handle, sub) = bounded(10000);
        indicatif_view(sub.clone());

        // Unwrap for now, will discuss if pipeline should be stopped if match state is not
        // running.
        state_handle
            .send(Message::Set {
                component: Component::Fetcher,
                total: hashing_methods.len() as u32,
            })
            .unwrap();

        for (id, _) in hashing_methods.iter().enumerate() {
            state_handle
                .send(Message::Update {
                    component: Component::Fetcher,
                    delta: 1,
                })
                .unwrap();

            let fetch_res = self.fetcher.fetch(id as u16, state_handle.clone()).await?;
            let processor_res = self.processor.process(fetch_res, state_handle.clone())?;
            self.parser
                .parse(processor_res, state_handle.clone())
                .await?;
        }
        Ok(())
    }
}

/// Spawn a view thread that uses indicatif as frontend to MatchState
fn indicatif_view(rx_state: Receiver<Message>) {
    let style = ProgressStyle::with_template(
        "[{elapsed_precise} | {eta_precise}] {msg}: {pos:>7}/{len:7} {percent}%",
    )
    .unwrap()
    .progress_chars("##-");

    thread::spawn(move || {
        let multi = MultiProgress::new();
        let mut progressbars = HashMap::new();
        while let Ok(m) = rx_state.recv() {
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
            let processor = Box::new(ThreadedUniquePairMatcher::default());
            let parser = Box::new(RcSqliteResultParser::from_pool(self.pool.clone()));

            let pipeline = MatchPipeline::new(fetcher, processor, parser);

            pipeline.execute(hashing_methods).await?;
            Ok(())
        })
    }
}
