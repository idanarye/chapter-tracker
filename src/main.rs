fn main() -> anyhow::Result<()> {
    flexi_logger::Logger::try_with_env_or_str("warn")?.start()?;
    let exit_status = chapter_tracker::start_gui()?;
    if exit_status != 0 {
        std::process::exit(exit_status);
    }
    Ok(())
}
