use std::str::FromStr;

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

pub struct DB {}
impl DB {
    pub fn new() -> Self {
        Self {}
    }
    pub async fn create_db() -> Result<(), sqlx::Error> {
        let pool = SqlitePool::connect_with(
            SqliteConnectOptions::from_str("sqlite:data.db")?.create_if_missing(true),
        )
        .await?;

        let mut tx = pool.begin().await?;

        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL
            )
            ",
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS images (
            id INTEGER PRIMARY KEY,
            path TEXT NOT NULL,
            user TEXT NOT NULL
            );
            ",
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS modifications (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL
            );
            ",
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS hashing_methods (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            name TEXT NOT NULL
            );
            ",
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS modified_images (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            image_id INTEGER NOT NULL,
            modification_id INTEGER NOT NULL,
            FOREIGN KEY (image_id) REFERENCES images(id),
            FOREIGN KEY (modification_id) REFERENCES modifications(id)
            );
            ",
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS hashes (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            hash BLOB NOT NULL,
            mod_image_id INTEGER NOT NULL,
            hashing_method_id INTEGER NOT NULL,
            FOREIGN KEY (mod_image_id) REFERENCES modified_images(id),
            FOREIGN KEY (hashing_method_id) REFERENCES hashing_methods(id)
            );
            ",
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS matches (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                hamming_distance INTEGER,
                hash_len INTEGER,
                hash1_id INTEGER,
                hash2_id INTEGER,
                FOREIGN KEY (hash1_id) REFERENCES hashes(id),
                FOREIGN KEY (hash2_id) REFERENCES hashes(id)
                )",
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }
}
