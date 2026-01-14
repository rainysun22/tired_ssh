use anyhow::Result;
mod app;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    app::run().await?;
    Ok(())
}