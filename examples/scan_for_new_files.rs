use sqlx::sqlite::SqlitePool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    flexi_logger::Logger::with_env_or_str("warn").start()?;
    let pool = SqlitePool::connect("sqlite:chapter_tracker.db3").await?;
    chapter_tracker::files_discovery::run_files_discovery(pool.acquire().await?).await?;
    Ok(())
}
