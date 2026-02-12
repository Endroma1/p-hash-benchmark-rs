use p_hash::core::app::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    log::debug!("starting phash");

    let app = App::try_default().await?;
    app.run().await?;
    Ok(())
}
