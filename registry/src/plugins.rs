use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Plugins {
    plugins: HashMap<String, PluginInfo>,
    current: String,
}

impl Plugins {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn select(
        &mut self,
        name: String,
        create_info: impl FnOnce() -> Option<PluginInfo>,
    ) -> Result<()> {
        if !self.plugins.contains_key(&name) {
            let info = create_info().ok_or_else(|| anyhow!("the plugin info is `None`"))?;
            self.plugins.insert(name.clone(), info);
        }
        self.current = name;
        Ok(())
    }

    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> Result<()> {
        let info = self
            .plugins
            .get_mut(name)
            .ok_or_else(|| anyhow!("the plugin `{name}` doesn't exist"))?;
        match (info.enabled, enabled) {
            (true, true) => bail!("the plugin is already enabled"),
            (false, false) => bail!("the plugin is already disabled"),
            (ref mut enabled, set) => *enabled = set,
        }
        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Serialize, Deserialize)]
pub struct PluginInfo {
    pub authors: String,
    pub enabled: bool,
}

impl PluginInfo {
    pub fn from_optional_authors(authors: Option<String>) -> Option<Self> {
        authors.map(|authors| PluginInfo {
            authors,
            enabled: true,
        })
    }
}
