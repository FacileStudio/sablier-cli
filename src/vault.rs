/// The keyring entry the CLI and the Sablier desktop app share.
pub const SERVICE: &str = "studio.facile.sablier";
pub const ACCOUNT: &str = "session";

/// Where the bearer token lives when it is not in `~/.sablier.yml`.
///
/// Every error is a reason the store cannot be used, carried as text so the
/// caller can name it in the fallback notice.
pub trait Vault {
    fn get(&self) -> Result<Option<String>, String>;
    fn set(&self, token: &str) -> Result<(), String>;
    fn delete(&self) -> Result<bool, String>;
}

/// The OS keyring: Keychain on macOS, Credential Manager on Windows, the
/// Secret Service on Linux.
pub struct Keyring;

fn entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(SERVICE, ACCOUNT).map_err(|err| err.to_string())
}

impl Vault for Keyring {
    fn get(&self) -> Result<Option<String>, String> {
        match entry()?.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(err) => Err(err.to_string()),
        }
    }

    fn set(&self, token: &str) -> Result<(), String> {
        entry()?.set_password(token).map_err(|err| err.to_string())
    }

    fn delete(&self) -> Result<bool, String> {
        match entry()?.delete_credential() {
            Ok(()) => Ok(true),
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(err) => Err(err.to_string()),
        }
    }
}

/// Writes `token` and reads it back, so a store that accepts a write and keeps
/// nothing never costs the copy on disk.
pub fn store(vault: &dyn Vault, token: &str) -> Result<(), String> {
    vault.set(token)?;
    match vault.get()? {
        Some(stored) if stored == token => Ok(()),
        _ => Err("the keyring did not keep the token".into()),
    }
}
