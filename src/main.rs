fn main() -> anyhow::Result<()> {
    flexi_logger::Logger::try_with_env_or_str("warn")?.start()?;
    chapter_tracker::start_gui()?;
    Ok(())
}
