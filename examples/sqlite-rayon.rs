use std::{env::args, path::PathBuf, str::FromStr};

use p_hash::{
    core::{app::App, images_processor::RayonImagesProcessor, result_parser::SqliteResultParser},
    hashing_methods,
    image_hash::{self, HashingMethods},
    image_modify::{self, Modifications},
    matching::match_process::SqliteRunner,
    modified_images,
};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger()?;

    tracing::debug!("starting phash");

    // Choosing what method to process images with.
    let processor = Box::new(RayonImagesProcessor::default());

    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::from_str("sqlite:data.db")?
            .create_if_missing(true)
            .pragma("cache_size", "200000"),
    )
    .await?;

    let parser = Box::new(SqliteResultParser::new(pool.clone()));
    let match_process = Box::new(SqliteRunner::new(pool.clone()));

    let args: Vec<String> = args().collect();
    let example_dir = match args.get(1) {
        Some(s) => PathBuf::from(s),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("images"),
    };

    tracing::info!("Using path {:?}", example_dir);

    // What modifications that should be used.
    let modifications = modified_images![
        image_modify::Angle::Rot180,
        image_modify::Angle::Rot90,
        image_modify::Angle::Rot270,
        image_modify::Blur::default()
    ];

    // What hashing methods that should be used.
    let hashing_methods = hashing_methods![
        image_hash::AverageHash::new(),
        image_hash::VertGradient::new()
    ];

    let app = App::builder()
        .images_path(&example_dir)
        .images_processor(processor)
        .results_parser(parser)
        .match_process(match_process)
        .modifications(modifications)
        .hashing_methods(hashing_methods)
        .finish();

    if let Err(e) = app.run().await {
        tracing::error!("{e}");
    };

    Ok(())
}

fn init_logger() -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    Ok(())
}
