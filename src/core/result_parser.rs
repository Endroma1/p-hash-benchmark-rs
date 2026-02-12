use async_trait::async_trait;

use crate::core::{error::Error, state::PHashResult};

#[async_trait]
trait ResultParser {
    async fn parse(&self, results: Vec<PHashResult>) -> Result<(), Error>;
}

pub struct SqliteResultParser {}
