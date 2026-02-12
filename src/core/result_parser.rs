use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::{
    core::{
        app::{get_hashing_methods, get_modifications},
        error::Error,
        state::AppProcessResult,
    },
    db::DB,
};

#[async_trait]
pub trait ResultParser {
    async fn parse(&self, results: AppProcessResult) -> Result<(), Error>;
}

pub struct SqliteResultParser {
    pool: SqlitePool,
}
impl SqliteResultParser {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool: pool }
    }
}
impl ResultParser for SqliteResultParser {
    fn parse<'life0, 'async_trait>(
        &'life0 self,
        results: AppProcessResult,
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
            DB::create_db().await?;
            get_modifications().send_to_db(&self.pool).await?;
            get_hashing_methods().send_to_db(&self.pool).await?;
            let mut tx = self.pool.begin().await?;
            for (id, img) in results.images().iter().enumerate() {
                let path_str = img.get_path().to_string_lossy().to_string();
                sqlx::query(
                    "
            INSERT INTO images (id, path, user) VALUES (?, ?, ?) ON CONFLICT(id) DO NOTHING;
            ",
                )
                .bind(id as u32)
                .bind(path_str)
                .bind(img.get_user())
                .execute(&mut *tx)
                .await?;
            }
            tx.commit().await?;
            results.phash_results().send_to_db(&self.pool).await?;
            Ok(())
        })
    }
}
