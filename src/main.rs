mod cpe;
use cpe::{download_cpe, put_cpe_to_db};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    download_cpe().await?;
    put_cpe_to_db().await?;
    Ok(())
}
