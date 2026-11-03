use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::{json, Value};

use super::settings::{is_ours, merge, strip};
use crate::ui;

/// Merges the heartbeat hooks into `~/.claude/settings.json`.
pub fn install() -> Result<()> {
    let path = settings_path()?;
    let mut settings = load(&path)?;
    let before = settings.clone();
    merge(&mut settings)?;
    if settings == before {
        ui::step(&format!("Claude Code hooks already in {}", path.display()));
        return Ok(());
    }
    save(&path, &settings)?;
    ui::success(&format!(
        "Claude Code hooks installed in {}",
        path.display()
    ));
    Ok(())
}

/// Removes the heartbeat hooks from `~/.claude/settings.json`.
pub fn uninstall() -> Result<()> {
    let path = settings_path()?;
    let mut settings = load(&path)?;
    let before = settings.clone();
    strip(&mut settings, is_ours);
    if settings == before {
        ui::step("No Sablier hooks to remove");
        return Ok(());
    }
    save(&path, &settings)?;
    ui::success(&format!(
        "Claude Code hooks removed from {}",
        path.display()
    ));
    Ok(())
}

fn settings_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("cannot determine home directory")?;
    Ok(home.join(".claude").join("settings.json"))
}

fn load(path: &Path) -> Result<Value> {
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(json!({})),
        Err(e) => return Err(e).with_context(|| format!("cannot read {}", path.display())),
    };
    if raw.trim().is_empty() {
        return Ok(json!({}));
    }
    serde_json::from_str(&raw).with_context(|| format!("{} is not valid JSON", path.display()))
}

fn save(path: &Path, settings: &Value) -> Result<()> {
    if path.exists() {
        std::fs::copy(path, path.with_extension("json.bak")).context("cannot back up settings")?;
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let body = serde_json::to_string_pretty(settings)? + "\n";
    std::fs::write(path, body).with_context(|| format!("cannot write {}", path.display()))
}
