use std::path::Path;

use p_hash::{core::app::App, db::DB};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    log::debug!("starting phash");
    let test_path = Path::new("/home/endro/.local/share/p-hash/images/");

    log::debug!("Creating db ...");
    DB::create_db().await?;

    let app = App::new(test_path);
    app.run().await?;
    Ok(())
}
