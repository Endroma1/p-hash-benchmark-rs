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

        // Current running program. References what run that is currently being processed and
        // should be used for matching.
        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS program (
            id INTEGER PRIMARY KEY CHECK (id = 0),
            run_id INTEGER NOT NULL,
            FOREIGN KEY (run_id) REFERENCES runs(id)
            );
            ",
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL
            );
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
            CREATE TABLE IF NOT EXISTS run_images(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            run_id INTEGER NOT NULL,
            image_id INTEGER NOT NULL,
            FOREIGN KEY (run_id) REFERENCES runs(id),
            FOREIGN KEY (image_id) REFERENCES images(id)
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
            UNIQUE (image_id, modification_id)
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
            UNIQUE (mod_image_id, hashing_method_id)
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

        sqlx::query(
            "
CREATE VIEW IF NOT EXISTS active_run_hashes AS
SELECT h.id AS hash_id,
       h.hash,
       h.hashing_method_id,
       mi.id AS modified_image_id,
       mi.image_id,
       mi.modification_id
FROM hashes h
JOIN modified_images mi ON mi.id = h.mod_image_id
JOIN images i ON i.id = mi.image_id
JOIN run_images ri ON ri.image_id = i.id
JOIN program p ON p.run_id = ri.run_id;
            ",
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "
CREATE VIEW IF NOT EXISTS active_run_modified_images AS
SELECT mi.id AS modified_image_id,
       mi.image_id,
       mi.modification_id
FROM modified_images mi
JOIN images i ON i.id = mi.image_id
JOIN run_images ri ON ri.image_id = i.id
JOIN program p ON p.run_id = ri.run_id;
            ",
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }
}
