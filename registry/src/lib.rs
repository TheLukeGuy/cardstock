use crate::store::DataStore;
use anyhow::{Context, Result};
use log::info;

pub mod store;

pub fn run() -> Result<()> {
    let data = DataStore::load_or_default("data.toml").context("failed to load the data store")?;
    info!("Using data store: {data:?}");
    Ok(())
}
