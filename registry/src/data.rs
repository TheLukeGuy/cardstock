use anyhow::{Context, Result};
use log::warn;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::borrow::Cow;
use std::fs;
use std::path::Path;

pub mod config;
pub mod store;

pub trait PersistentData: DeserializeOwned {
    const DESCRIPTION_LOWERCASE: &'static str;
    const DEFAULT: &'static str;
    const SAVE_DEFAULT: bool;

    fn load_or_default(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let contents = match fs::read_to_string(path) {
            Ok(contents) => Cow::Owned(contents),
            Err(_) if !path.exists() => {
                warn!("Creating a new {}.", Self::DESCRIPTION_LOWERCASE);
                if Self::SAVE_DEFAULT {
                    fs::write(path, Self::DEFAULT).with_context(|| {
                        format!(
                            "failed to write the default {} to `{}`",
                            Self::DESCRIPTION_LOWERCASE,
                            path.display()
                        )
                    })?;
                }
                Cow::Borrowed(Self::DEFAULT)
            }
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "failed to read from the {} at `{}`",
                        Self::DESCRIPTION_LOWERCASE,
                        path.display()
                    )
                })
            }
        };

        toml::from_str(&contents)
            .with_context(|| format!("failed to deserialize the {}", Self::DESCRIPTION_LOWERCASE))
    }

    fn save(&self, path: impl AsRef<Path>) -> Result<()>
    where
        Self: Serialize,
    {
        let path = path.as_ref();
        let serialized = toml::to_string(self)
            .with_context(|| format!("failed to serialize the {}", Self::DESCRIPTION_LOWERCASE))?;
        fs::write(path, serialized).with_context(|| {
            format!(
                "failed to write the {} to `{}`",
                Self::DESCRIPTION_LOWERCASE,
                path.display()
            )
        })?;
        Ok(())
    }
}
