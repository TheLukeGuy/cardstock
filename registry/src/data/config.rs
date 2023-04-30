use crate::data::PersistentData;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub save: SaveConfig,
    pub ads: AdsConfig,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind_addr: String,
    pub error_tolerance: i32,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Serialize, Deserialize)]
pub struct SaveConfig {
    pub enabled: bool,
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Serialize, Deserialize)]
pub struct AdsConfig {
    pub enabled: bool,
    pub one_in_x_chance: u32,
    pub list: Vec<String>,
}

impl PersistentData for Config {
    const DESCRIPTION_LOWERCASE: &'static str = "config";
    const DEFAULT: &'static str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/config.toml"));
    const SAVE_DEFAULT: bool = true;
}
