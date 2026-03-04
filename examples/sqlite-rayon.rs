use std::{env::home_dir, str::FromStr};

use p_hash::{
    core::{app::App, images_processor::RayonImagesProcessor, result_parser::SqliteResultParser},
    image_hash::{self, HashingMethods},
    image_modify::{self, Modifications},
    matching::match_process::SqliteRunner,
};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger()?;

    tracing::debug!("starting phash");

    let processor = Box::new(RayonImagesProcessor::default());

    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::from_str("sqlite:data.db")?
            .create_if_missing(true)
            .pragma("cache_size", "200000"),
    )
    .await?;

    let parser = Box::new(SqliteResultParser::new(pool.clone()));
    let match_process = Box::new(SqliteRunner::new(pool.clone()));
    let mut home = home_dir().unwrap();
    home.push(".local/share/p-hash/images");

    let mut modifications = Modifications::new();
    modifications.push(image_modify::Angle::Rot180);

    let mut hashing_methods = HashingMethods::new();
    hashing_methods.push(image_hash::AverageHash::new());

    let app = App::builder()
        .images_path(&home)
        .images_processor(processor)
        .results_parser(parser)
        .match_process(match_process)
        .modifications(modifications)
        .hashing_methods(hashing_methods)
        .finish();

    app.run().await?;

    Ok(())
}

fn init_logger() -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    Ok(())
}
