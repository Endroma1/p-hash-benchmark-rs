use std::sync::mpsc::Receiver;

use async_trait::async_trait;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use tokio::task::{self};

use crate::matching::{
    error::Error,
    state::{Component, Match, MatchState, Matches},
};

#[async_trait]
pub trait MatchResultParser: Sync + Send {
    type Result;
    type Error;
    async fn parse(&self, results: Self::Result, state: MatchState) -> Result<(), Self::Error>;
}

// Takes a Receiver<Hash> and sends each entry to DB. Use SqliteResultParser to pars Matches
// instead.
pub struct RcSqliteResultParser {
    pool: SqlitePool,
}
impl RcSqliteResultParser {
    pub fn from_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }
}
impl MatchResultParser for RcSqliteResultParser {
    type Error = Error;
    type Result = Receiver<Match>;
    fn parse<'life0, 'async_trait>(
        &'life0 self,
        results: Self::Result,
        _: MatchState,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<(), Self::Error>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            tracing::debug!("starting parser");
            let mut stop = false;
            let batch_size = 8000; // reaching limit for sqlite
            loop {
                let mut batch = Vec::with_capacity(10_000);
                while let Ok(m) = results.recv() {
                    batch.push(m);
                    if batch.len() >= batch_size {
                        break;
                    }
                }
                if batch.len() < batch_size {
                    stop = true;
                }

                let pool = self.pool.clone();
                let mut query: QueryBuilder<Sqlite> = QueryBuilder::new(
                    "INSERT INTO matches (hamming_distance, hash_len, hash1_id, hash2_id) ",
                );
                query.push_values(batch.iter(), |mut b, m| {
                    b.push_bind(m.hamming_distance().distance())
                        .push_bind(m.hamming_distance().entry_length())
                        .push_bind(m.hash_id1())
                        .push_bind(m.hash_id2());
                });
                query.build().execute(&pool).await?;

                if stop {
                    tracing::debug!("match result parser exiting");
                    break;
                }
            }

            Ok(())
        })
    }
}
pub struct SqliteResultParser {
    pool: SqlitePool,
}
impl SqliteResultParser {
    pub fn from_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }
}
impl MatchResultParser for SqliteResultParser {
    type Error = Error;
    type Result = Matches;
    fn parse<'life0, 'async_trait>(
        &'life0 self,
        mut results: Self::Result,
        s: MatchState,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<(), Self::Error>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            while !results.is_empty() {
                let chunk: Vec<Match> = results.drain(..10_000).collect();
                s.update(Component::Parser, chunk.len() as u32);

                let mut tx = self.pool.begin().await?;

                task::spawn(async move {
                    for result in chunk.iter() {
                        sqlx::query("
                    INSERT INTO matches (hamming_distance, hash_len, hash1_id, hash2_id) VALUES (?,?,?,?)
                    ")
                    .bind(result.hamming_distance().distance())
                    .bind(result.hamming_distance().entry_length())
                    .bind(result.hash_id1())
                    .bind( result.hash_id2())
                    .execute(&mut *tx)
                    .await.unwrap();
                    }
                    tx.commit().await.unwrap();
                });
            }
            Ok(())
        })
    }
}
