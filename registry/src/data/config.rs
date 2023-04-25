use crate::data::PersistentData;
use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub bind_addr: String,
}

impl PersistentData for Config {
    const DESCRIPTION_LOWERCASE: &'static str = "config";
    const DEFAULT: &'static str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/config.toml"));
    const SAVE_DEFAULT: bool = true;
}
