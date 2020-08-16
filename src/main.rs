fn main() -> anyhow::Result<()> {
    flexi_logger::Logger::with_env_or_str("warn").start()?;
    let system = actix::System::new("chapter-tracker");

    system.run()?;
    Ok(())
}
