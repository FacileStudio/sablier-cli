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

    /// save writes the config, readable only by its owner.
    ///
    /// The file holds a bearer token that never expires, so it is a
    /// credential at rest and the mode matters as much as the contents.
    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        let contents = serde_yaml::to_string(self).context("cannot serialise the config")?;
        std::fs::write(&path, contents)
            .with_context(|| format!("cannot write {}", path.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
                .with_context(|| format!("cannot restrict {}", path.display()))?;
        }
        Ok(())
    }

    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        let contents = std::fs::read_to_string(&path).map_err(|_| {
            anyhow::anyhow!(
                "cannot read {} — run `sablier login` to sign in",
                path.display()
            )
        })?;
        let config: Self = serde_yaml::from_str(&contents)
            .with_context(|| format!("invalid config at {}", path.display()))?;
        Ok(config)
    }
}
