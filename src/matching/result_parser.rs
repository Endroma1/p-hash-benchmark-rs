use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::matching::{error::Error, state::Matches};

#[async_trait]
pub trait MatchResultParser: Sync + Send {
    type Result;
    type Error;
    async fn parse(&self, results: Self::Result) -> Result<(), Self::Error>;
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
        results: Self::Result,
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
            let mut tx = self.pool.begin().await?;
            for result in results.iter() {
                sqlx::query("
                    INSERT INTO matches (hamming_distance, hash_len, hash1_id, hash2_id) VALUES (?,?,?,?)
                    ")
                    .bind(result.hamming_distance().distance())
                    .bind(result.hamming_distance().entry_length())
                    .bind(result.hash_id1())
                    .bind( result.hash_id2())
                    .execute(&mut *tx)
                    .await?;
            }
            tx.commit().await?;
            Ok(())
        })
    }
}
