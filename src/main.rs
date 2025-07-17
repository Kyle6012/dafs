mod peer;
mod crypto;
mod storage;
mod ai;
mod api;
mod models;

use storage::Storage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    println!("🚀 Starting DAFS node...");

    // Initialize storage
    let _storage = Storage::new("dafs_db")?;

    // Start P2P node
    peer::start_node().await?;

    // Placeholder: In the future, spawn API server here
    // tokio::spawn(async move { api::run().await });

    Ok(())
}
