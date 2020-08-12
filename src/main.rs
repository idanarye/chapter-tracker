use sqlx::sqlite::SqlitePool;

use chapter_tracker::migrate_manually;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    flexi_logger::Logger::with_env_or_str("warn").start()?;
    let pool = SqlitePool::builder().build("sqlite:chapter_tracker.db3").await?;
    migrate_manually(&pool).await?;
    Ok(())
}
