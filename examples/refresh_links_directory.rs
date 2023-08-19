use std::io::ErrorKind;
use std::path::Path;

use sqlx::sqlite::SqlitePool;
use tokio::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    flexi_logger::Logger::try_with_env_or_str("warn")?.start()?;
    let pool = SqlitePool::connect("sqlite:chapter_tracker.db3").await?;
    let path = Path::new("episodes-links");
    fs::create_dir(path).await.or_else(|err| {
        if err.kind() == ErrorKind::AlreadyExists {
            Ok(())
        } else {
            Err(err)
        }
    })?;
    chapter_tracker::links_handling::refresh_links_directory(&mut pool.acquire().await?, &path)
        .await?;
    Ok(())
}
