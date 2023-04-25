use crate::data::PersistentData;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct DataStore {
    commands: HashMap<String, Arc<String>>,
}

impl DataStore {
    pub fn check(&self, name: &str) -> Option<Arc<String>> {
        self.commands.get(name).cloned()
    }

    pub fn register(&mut self, name: impl Into<String>, plugin: impl Into<String>) -> Result<()> {
        let name = name.into();
        let plugin = plugin.into();
        match self.commands.entry(name.clone()) {
            Entry::Occupied(_) => bail!("the command `{name}` is already registered to `{plugin}`"),
            Entry::Vacant(vacant) => {
                vacant.insert(Arc::new(plugin));
                Ok(())
            }
        }
    }
}

impl PersistentData for DataStore {
    const DESCRIPTION_LOWERCASE: &'static str = "data store";
    const DEFAULT: &'static str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/data.toml"));
    const SAVE_DEFAULT: bool = false;
}
