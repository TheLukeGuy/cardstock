use anyhow::{anyhow, bail, Result};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Plugins {
    plugins: HashMap<String, PluginInfo>,
    current: String, // Cardstock TODO: Make this an Option<String>
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
            debug!("Selecting plugin `{name}` by `{}`.", info.authors);
            self.plugins.insert(name.clone(), info);
        } else {
            debug!("Selecting plugin `{name}`.");
        }
        self.current = name;
        Ok(())
    }

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        let info = self.current_info_mut();
        match (&mut info.enabled, enabled) {
            (true, true) => bail!("the plugin is already enabled"),
            (false, false) => bail!("the plugin is already disabled"),
            (enabled, set) => {
                debug!("Setting plugin enabled to `{set}`.");
                *enabled = set
            }
        }
        Ok(())
    }

    pub fn register_cmd(&mut self, name: String, status: GlobalCommandStatus) {
        self.current_info_mut().cmds.insert(name, status);
    }

    pub fn current_authors(&self) -> &str {
        &self.current_info().authors
    }

    fn current_info(&self) -> &PluginInfo {
        self.plugins.get(&self.current).unwrap()
    }

    fn current_info_mut(&mut self) -> &mut PluginInfo {
        self.plugins.get_mut(&self.current).unwrap()
    }

    pub fn selected(&self) -> &str {
        &self.current
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct PluginInfo {
    pub authors: String,
    pub enabled: bool,
    pub cmds: HashMap<String, GlobalCommandStatus>,
}

impl PluginInfo {
    pub fn from_optional_authors(authors: Option<String>) -> Option<Self> {
        authors.map(|authors| PluginInfo {
            authors,
            enabled: false,
            cmds: HashMap::new(),
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Serialize, Deserialize)]
pub enum GlobalCommandStatus {
    #[default]
    Unregistered,
    Registered,
}
