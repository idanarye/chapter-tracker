use sqlx::sqlite::SqlitePool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    flexi_logger::Logger::with_env_or_str("warn").start()?;
    let pool = SqlitePool::connect("sqlite:chapter_tracker.db3").await?;
    let dangling_files = chapter_tracker::files_discovery::run_dangling_files_scan(&mut pool.acquire().await?).await?;
    println!("Dangling files be {:?}", dangling_files);
    Ok(())
}
