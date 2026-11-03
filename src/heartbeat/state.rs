use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub const MIN_INTERVAL_SECS: i64 = 120;
const RETAIN_SECS: i64 = 86_400;

/// Locates `heartbeats.json` under `$XDG_STATE_HOME/sablier`, or
/// `~/.local/state/sablier` when that variable is unset or not absolute.
pub fn path() -> Result<PathBuf> {
    let root = state_root(std::env::var_os("XDG_STATE_HOME"), dirs::home_dir())
        .context("cannot determine the state directory")?;
    Ok(root.join("sablier").join("heartbeats.json"))
}

fn state_root(xdg: Option<OsString>, home: Option<PathBuf>) -> Option<PathBuf> {
    match xdg.map(PathBuf::from) {
        Some(dir) if dir.is_absolute() => Some(dir),
        _ => home.map(|h| h.join(".local").join("state")),
    }
}

/// Identifies the rate-limit bucket of one (source, entity) pair.
pub fn key(source: &str, entity: &str) -> String {
    format!("{source}\t{entity}")
}

/// Decides whether a heartbeat goes out: always when forced (a write or a
/// prompt), otherwise only when the last one for the pair is at least
/// `MIN_INTERVAL_SECS` old.
pub fn should_send(last_sent: Option<i64>, now: i64, force: bool) -> bool {
    force || last_sent.is_none_or(|t| now - t >= MIN_INTERVAL_SECS)
}

/// Reads the last-sent timestamps, treating a missing or corrupt file as empty.
pub fn load(path: &Path) -> HashMap<String, i64> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

/// Writes the timestamps back, dropping pairs older than a day so the file
/// stays small, through a rename so a concurrent reader never sees half a file.
pub fn save(path: &Path, mut seen: HashMap<String, i64>, now: i64) -> Result<()> {
    seen.retain(|_, t| now - *t < RETAIN_SECS);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension(format!("json.{}", std::process::id()));
    std::fs::write(&tmp, serde_json::to_vec(&seen)?)?;
    std::fs::rename(&tmp, path).with_context(|| format!("cannot write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pair_is_rate_limited_unless_forced() {
        assert!(should_send(None, 1_000, false));
        assert!(!should_send(Some(1_000), 1_119, false));
        assert!(should_send(Some(1_000), 1_120, false));
        assert!(should_send(Some(1_000), 1_001, true));
    }

    #[test]
    fn the_state_dir_follows_xdg_when_absolute() {
        let home = Some(PathBuf::from("/home/u"));
        assert_eq!(
            state_root(Some("/x/state".into()), home.clone()),
            Some(PathBuf::from("/x/state"))
        );
        assert_eq!(
            state_root(Some("relative".into()), home.clone()),
            Some(PathBuf::from("/home/u/.local/state"))
        );
        assert_eq!(
            state_root(None, home),
            Some(PathBuf::from("/home/u/.local/state"))
        );
    }

    #[test]
    fn saving_round_trips_and_forgets_stale_pairs() {
        let path = std::env::temp_dir()
            .join(format!("sablier-{}-state", std::process::id()))
            .join("heartbeats.json");
        let seen = HashMap::from([(key("git", "/a"), 10_000), (key("git", "/b"), 1)]);
        save(&path, seen, 90_000).unwrap();
        let back = load(&path);
        assert_eq!(back, HashMap::from([(key("git", "/a"), 10_000)]));
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
