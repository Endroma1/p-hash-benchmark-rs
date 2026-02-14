use p_hash::core::app::App;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger()?;

    tracing::debug!("starting phash");

    let app = App::try_default().await?;
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
