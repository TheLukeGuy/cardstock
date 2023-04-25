use crate::data::PersistentData;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub bind_addr: String,
    pub save: SaveConfig,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Serialize, Deserialize)]
pub struct SaveConfig {
    pub enabled: bool,
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
}

impl PersistentData for Config {
    const DESCRIPTION_LOWERCASE: &'static str = "config";
    const DEFAULT: &'static str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/config.toml"));
    const SAVE_DEFAULT: bool = true;
}
