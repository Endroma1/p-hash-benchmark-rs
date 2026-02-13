use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::{
    img_hash::HashingMethodID,
    matching::{
        error::Error,
        state::{Hash, Hashes},
    },
};

// Fetches hashes from source based on their hashing method used.
#[async_trait]
pub trait ResultsFetcher: Send + Sync {
    type Error;
    type Output;
    async fn fetch(&self, method_id: HashingMethodID) -> Result<Self::Output, Self::Error>;
}

pub struct SqliteFetcher {
    pool: SqlitePool,
}
impl SqliteFetcher {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}
impl ResultsFetcher for SqliteFetcher {
    type Output = Hashes;
    type Error = Error;
    fn fetch<'life0, 'async_trait>(
        &'life0 self,
        method_id: HashingMethodID,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<Self::Output, Self::Error>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let res: Result<Vec<Hash>, sqlx::Error> = sqlx::query_as(
                "
                SELECT id, hash FROM hashes WHERE hashing_method_id = ?;
                ",
            )
            .bind(method_id)
            .fetch_all(&self.pool)
            .await;
            res.map(Hashes::from).map_err(|e| Error::Sqlx { err: e })
        })
    }
}
