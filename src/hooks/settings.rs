use anyhow::{Context, Result};
use serde_json::{json, Map, Value};

const PROMPT_COMMAND: &str = "sablier heartbeat --source claude-code --stdin-claude";
const EDIT_COMMAND: &str = "sablier heartbeat --source claude-code --stdin-claude --write";
const EDIT_MATCHER: &str = "Edit|Write|MultiEdit|NotebookEdit";

/// Adds our two hook groups unless they are already there, after dropping
/// any older variant of our commands. Every other key and hook is kept.
pub fn merge(settings: &mut Value) -> Result<()> {
    strip(settings, |cmd| {
        is_ours(cmd) && cmd != PROMPT_COMMAND && cmd != EDIT_COMMAND
    });
    let hooks = settings
        .as_object_mut()
        .context("settings.json is not a JSON object")?
        .entry("hooks")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .context("\"hooks\" in settings.json is not an object")?;
    let prompt = json!({"hooks": [command(PROMPT_COMMAND)]});
    let edit = json!({"matcher": EDIT_MATCHER, "hooks": [command(EDIT_COMMAND)]});
    add(hooks, "UserPromptSubmit", PROMPT_COMMAND, prompt)?;
    add(hooks, "PostToolUse", EDIT_COMMAND, edit)
}

fn command(cmd: &str) -> Value {
    json!({"type": "command", "command": cmd})
}

fn add(hooks: &mut Map<String, Value>, event: &str, cmd: &str, group: Value) -> Result<()> {
    let groups = hooks
        .entry(event)
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .with_context(|| format!("hooks.{event} in settings.json is not a list"))?;
    if !groups.iter().any(|g| commands(g).any(|c| c == cmd)) {
        groups.push(group);
    }
    Ok(())
}

fn commands(group: &Value) -> impl Iterator<Item = &str> {
    group
        .get("hooks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|h| h.get("command").and_then(Value::as_str))
}

/// Recognises a hook command this CLI installed, in any version.
pub fn is_ours(cmd: &str) -> bool {
    cmd.starts_with("sablier heartbeat ") && cmd.contains("--source claude-code")
}

/// Removes every hook whose command matches `drop`, then any group or event
/// list that this emptied. Lists that were already empty stay as they were.
pub fn strip(settings: &mut Value, drop: impl Fn(&str) -> bool + Copy) {
    let Some(hooks) = settings.get_mut("hooks").and_then(Value::as_object_mut) else {
        return;
    };
    hooks.retain(|_, groups| match groups.as_array_mut() {
        Some(list) => retain_emptied(list, |g| strip_group(g, drop)),
        None => true,
    });
}

fn strip_group(group: &mut Value, drop: impl Fn(&str) -> bool + Copy) -> bool {
    match group.get_mut("hooks").and_then(Value::as_array_mut) {
        Some(list) => retain_emptied(list, |h| {
            !h.get("command").and_then(Value::as_str).is_some_and(drop)
        }),
        None => true,
    }
}

fn retain_emptied(list: &mut Vec<Value>, keep: impl FnMut(&mut Value) -> bool) -> bool {
    let len = list.len();
    list.retain_mut(keep);
    list.len() == len || !list.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn foreign() -> Value {
        json!({
            "model": "opus",
            "hooks": {
                "PostToolUse": [
                    {"matcher": "Bash", "hooks": [{"type": "command", "command": "mycelium log"}]}
                ],
                "Stop": [{"hooks": [{"type": "command", "command": "notify-send done"}]}]
            }
        })
    }

    #[test]
    fn install_adds_both_hooks_and_keeps_foreign_ones() {
        let mut settings = foreign();
        merge(&mut settings).unwrap();
        assert_eq!(settings["model"], "opus");
        assert_eq!(settings["hooks"]["Stop"], foreign()["hooks"]["Stop"]);
        let post = settings["hooks"]["PostToolUse"].as_array().unwrap();
        assert_eq!(post.len(), 2);
        assert_eq!(post[0], foreign()["hooks"]["PostToolUse"][0]);
        assert_eq!(post[1]["matcher"], EDIT_MATCHER);
        assert_eq!(post[1]["hooks"][0]["command"], EDIT_COMMAND);
        let prompt = &settings["hooks"]["UserPromptSubmit"][0]["hooks"][0];
        assert_eq!(prompt["type"], "command");
        assert_eq!(prompt["command"], PROMPT_COMMAND);
    }

    #[test]
    fn install_is_idempotent_and_replaces_older_variants() {
        let mut settings = foreign();
        merge(&mut settings).unwrap();
        let once = settings.clone();
        merge(&mut settings).unwrap();
        assert_eq!(settings, once);

        let mut stale = foreign();
        stale["hooks"]["PostToolUse"][0]["hooks"]
            .as_array_mut()
            .unwrap()
            .push(json!({"type": "command", "command": "sablier heartbeat --source claude-code"}));
        merge(&mut stale).unwrap();
        assert_eq!(stale, once);
    }

    #[test]
    fn uninstall_removes_only_our_entries() {
        let mut settings = foreign();
        merge(&mut settings).unwrap();
        strip(&mut settings, is_ours);
        assert_eq!(settings, foreign());

        let mut empty_lists = json!({"hooks": {"Stop": []}});
        strip(&mut empty_lists, is_ours);
        assert_eq!(empty_lists, json!({"hooks": {"Stop": []}}));
    }

    #[test]
    fn install_refuses_to_clobber_a_malformed_hooks_key() {
        assert!(merge(&mut json!({"hooks": []})).is_err());
        assert!(merge(&mut json!({"hooks": {"PostToolUse": {}}})).is_err());
        assert!(merge(&mut json!([])).is_err());
    }
}
