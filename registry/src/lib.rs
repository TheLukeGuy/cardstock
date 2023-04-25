use crate::data::config::Config;
use crate::data::store::DataStore;
use crate::data::PersistentData;
use anyhow::{Context, Result};
use log::debug;

pub mod data;

pub fn run() -> Result<()> {
    let config = Config::load_or_default("config.toml").context("failed to load the config")?;
    debug!("Using config: {config:?}");
    let data = DataStore::load_or_default("data.toml").context("failed to load the data store")?;
    debug!("Using data store: {data:?}");

    Ok(())
}
