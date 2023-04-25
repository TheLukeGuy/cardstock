use anyhow::{Context, Result};
use log::LevelFilter;
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};

fn main() -> Result<()> {
    init_logging().context("failed to initialize logging")?;
    cardstock_registry::run()
}

fn init_logging() -> Result<()> {
    TermLogger::init(
        LevelFilter::Debug,
        ConfigBuilder::new()
            .set_time_offset_to_local()
            .unwrap_or_else(|builder| builder)
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;
    Ok(())
}
