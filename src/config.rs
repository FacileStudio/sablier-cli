use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::vault::{self, Keyring, Vault};

mod file;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server_url: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub token: String,
}

impl Config {
    pub fn path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("cannot determine home directory")?;
        Ok(home.join(".sablier.yml"))
    }

    /// save puts the token in the OS keyring and the server in `~/.sablier.yml`.
    ///
    /// Without a usable keyring the token goes into the file too, readable only
    /// by its owner, and the user is told so.
    pub fn save(&self) -> Result<()> {
        self.save_to(&Self::path()?, &Keyring)
    }

    /// clear forgets the token in the keyring and in the file but keeps
    /// server_url, so signing back in does not also mean retyping which Sablier
    /// this is.
    ///
    /// Returns `Ok(false)` when there was no token to forget.
    pub fn clear() -> Result<bool> {
        Self::clear_at(&Self::path()?, &Keyring)
    }

    /// load reads `~/.sablier.yml` and the token from the keyring.
    ///
    /// A token still in the file moves to the keyring, and leaves the file only
    /// once the keyring has it back. Without a usable keyring it stays put.
    pub fn load() -> Result<Self> {
        Self::load_from(&Self::path()?, &Keyring)
    }

    fn save_to(&self, path: &Path, vault: &dyn Vault) -> Result<()> {
        let mut on_disk = self.clone();
        if !self.token.is_empty() {
            match vault::store(vault, &self.token) {
                Ok(()) => on_disk.token.clear(),
                Err(reason) => fallback_notice(path, &reason),
            }
        }
        file::write(&on_disk, path)
    }

    fn clear_at(path: &Path, vault: &dyn Vault) -> Result<bool> {
        let in_vault = vault.delete().unwrap_or_else(|reason| {
            eprintln!("sablier: cannot clear the OS keyring ({reason})");
            false
        });
        let Ok(mut config) = file::read(path) else {
            return Ok(in_vault);
        };
        if config.token.is_empty() {
            return Ok(in_vault);
        }
        config.token.clear();
        file::write(&config, path)?;
        Ok(true)
    }

    fn load_from(path: &Path, vault: &dyn Vault) -> Result<Self> {
        let mut config = file::read(path)?;
        if config.token.is_empty() {
            config.token = vault.get().ok().flatten().unwrap_or_default();
            return Ok(config);
        }
        match vault::store(vault, &config.token) {
            Ok(()) => {
                let mut on_disk = config.clone();
                on_disk.token.clear();
                if let Err(err) = file::write(&on_disk, path) {
                    eprintln!(
                        "sablier: the token is in the OS keyring but is still in the file: {err:#}"
                    );
                }
            }
            Err(reason) => fallback_notice(path, &reason),
        }
        Ok(config)
    }
}

/// Tells the user the token stays on disk, and why.
fn fallback_notice(path: &Path, reason: &str) {
    eprintln!(
        "sablier: no usable OS keyring ({reason}); the token stays in {} (mode 0600)",
        path.display()
    );
}

#[cfg(all(test, unix))]
mod tests;
