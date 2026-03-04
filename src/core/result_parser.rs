use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::{
    core::{error::Error, state::AppProcessResult},
    db::DB,
    image_hash::HashingMethods,
    image_modify::Modifications,
};

#[async_trait]
pub trait ResultParser {
    async fn parse(
        &self,
        results: AppProcessResult,
        modifications: &Modifications,
        hashing_methods: &HashingMethods,
    ) -> Result<(), Error>;
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
    fn parse<'life0, 'life1, 'life2, 'async_trait>(
        &'life0 self,
        results: AppProcessResult,
        modifications: &'life1 Modifications,
        hashing_methods: &'life2 HashingMethods,
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
        'life2: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            DB::create_db().await?;
            send_hashes_to_db(&self.pool, hashing_methods).await?;
            send_modifications_to_db(&self.pool, &modifications).await?;
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

async fn send_modifications_to_db(
    pool: &SqlitePool,
    modifications: &Modifications,
) -> Result<(), Error> {
    let mut tx = pool.begin().await?;
    for (i, modification) in modifications.iter().enumerate() {
        let name = modification.name();
        sqlx::query(
            "
            INSERT INTO modifications (id, name) VALUES (?,?) ON CONFLICT(id) DO NOTHING;
            ",
        )
        .bind(i as u32)
        .bind(name)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

async fn send_hashes_to_db(
    pool: &SqlitePool,
    hashing_methods: &HashingMethods,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    for (id, obj) in hashing_methods.iter().enumerate() {
        sqlx::query(
            "
                INSERT INTO hashing_methods (id, name) VALUES (?,?) ON CONFLICT(id) DO NOTHING;
                ",
        )
        .bind(id as u32)
        .bind(obj.name())
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}
