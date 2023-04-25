use anyhow::{bail, Context, Error, Result};
use log::warn;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

const DEFAULT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/data.toml"));

#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct DataStore {
    commands: HashMap<String, String>,
}

impl DataStore {
    pub fn load_or_default(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let contents = match fs::read_to_string(path) {
            Ok(contents) => Cow::Owned(contents),
            Err(_) if !path.exists() => {
                warn!("Creating a new data store.");
                Cow::Borrowed(DEFAULT)
            }
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("failed to read from the data store at `{}`", path.display())
                })
            }
        };
        Self::from_str(&contents)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let serialized = toml::to_string(self).context("failed to serialize the data store")?;
        fs::write(path, serialized)
            .with_context(|| format!("failed to write the data store to `{}`", path.display()))?;
        Ok(())
    }

    pub fn register(&mut self, name: impl Into<String>, plugin: impl Into<String>) -> Result<()> {
        let name = name.into();
        let plugin = plugin.into();
        match self.commands.entry(name.clone()) {
            Entry::Occupied(_) => bail!("the command `{name}` is already registered to `{plugin}`"),
            Entry::Vacant(vacant) => {
                vacant.insert(plugin);
                Ok(())
            }
        }
    }
}

impl FromStr for DataStore {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        toml::from_str(s).context("failed to deserialize the data store")
    }
}
