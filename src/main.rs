fn main() -> anyhow::Result<()> {
    flexi_logger::Logger::with_env_or_str("warn").start()?;
    chapter_tracker::start_gui()?;
    // let system = actix::System::new("chapter-tracker");

    // system.run()?;
    Ok(())
}
