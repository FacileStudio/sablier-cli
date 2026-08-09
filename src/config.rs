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
        write_private(&path, contents.as_bytes())
            .with_context(|| format!("cannot write {}", path.display()))
    }

    /// clear forgets the token but keeps server_url, so signing back in does
    /// not also mean retyping which Sablier this is.
    ///
    /// Returns `Ok(false)` when there was no token to forget.
    pub fn clear() -> Result<bool> {
        let mut config = Self::load()?;
        if config.token.is_empty() {
            return Ok(false);
        }
        config.token.clear();
        config.save()?;
        Ok(true)
    }

    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        let contents = std::fs::read_to_string(&path).map_err(|_| {
            anyhow::anyhow!(
                "cannot read {} — run `sablier login` to sign in",
                path.display()
            )
        })?;
        let _ = ensure_secure_permissions(&path);
        let config: Self = serde_yaml::from_str(&contents)
            .with_context(|| format!("invalid config at {}", path.display()))?;
        Ok(config)
    }
}

/// Creates or replaces `path` with mode 0600 in one step. Writing first and
/// chmod-ing after leaves the token world-readable for the instant in between.
#[cfg(unix)]
fn write_private(path: &std::path::Path, contents: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    file.write_all(contents)?;
    file.sync_all()
}

#[cfg(not(unix))]
fn write_private(path: &std::path::Path, contents: &[u8]) -> std::io::Result<()> {
    std::fs::write(path, contents)
}

/// Tightens an existing config to 0600 when it grants group or other access,
/// so a token that leaked before this fix stops leaking on the next read.
///
/// Returns `Ok(true)` only when the mode was actually changed.
#[cfg(unix)]
fn ensure_secure_permissions(path: &std::path::Path) -> std::io::Result<bool> {
    use std::os::unix::fs::PermissionsExt;

    if std::fs::metadata(path)?.permissions().mode() & 0o077 == 0 {
        return Ok(false);
    }
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(true)
}

#[cfg(not(unix))]
fn ensure_secure_permissions(_path: &std::path::Path) -> std::io::Result<bool> {
    Ok(false)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn the_token_file_is_never_readable_by_anyone_else() {
        let mut path = std::env::temp_dir();
        path.push(format!("sablier-cli-{}-config.yml", std::process::id()));
        let _ = std::fs::remove_file(&path);

        write_private(&path, b"token: secret\n").unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "wrote {mode:o}");
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn an_already_leaked_config_is_tightened_on_read() {
        let mut path = std::env::temp_dir();
        path.push(format!("sablier-cli-{}-leaked.yml", std::process::id()));
        std::fs::write(&path, b"token: secret\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

        assert!(ensure_secure_permissions(&path).unwrap());
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "left {mode:o}");
        assert!(!ensure_secure_permissions(&path).unwrap());
        std::fs::remove_file(&path).unwrap();
    }
}
