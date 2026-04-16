use std::{env::args, path::Path, str::FromStr};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use p_hash::{
    hashing_methods,
    image_hash::{AverageHash, HashingMethods},
    image_modify::{Blur, Modifications},
    modifications,
    result_calc::{RocProcess, plot_roc},
};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = args().collect::<Vec<String>>();
    init_logger()?;
    let thresholds = (1..100).map(|i| (i as f32) / 100.).collect();

    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::from_str("sqlite:data.db")?
            .create_if_missing(false)
            .pragma("cache_size", "200000"),
    )
    .await?;
    let modifications = modifications![Blur::new(0.9)];
    let hashing_methods = hashing_methods![AverageHash::new(8)];
    let res = RocProcess::new(thresholds, pool, modifications, hashing_methods)
        .run()
        .await?;

    plot_roc(res, Path::new(&args[1]))?;
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
