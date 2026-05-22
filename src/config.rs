use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server_url: String,
    #[serde(default)]
    pub token: String,
}

impl Config {
    pub fn path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("cannot determine home directory")?;
        Ok(home.join(".sablier.yml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("cannot read {}\nRun `sablier login` to get started.", path.display()))?;
        let config: Self = serde_yaml::from_str(&contents)
            .with_context(|| format!("invalid config at {}", path.display()))?;
        Ok(config)
    }

    pub fn load_or_default() -> Self {
        Self::load().unwrap_or(Self {
            server_url: String::new(),
            token: String::new(),
        })
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        let contents = serde_yaml::to_string(self)?;
        std::fs::write(&path, contents)?;
        Ok(())
    }
}
