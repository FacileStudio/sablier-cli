use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{bail, Result};
use clap::Args;
use serde_json::Value;

use crate::api::{ApiClient, Heartbeat};
use crate::config::Config;

mod project;
mod state;

#[derive(Args, Debug, Clone)]
pub struct HeartbeatArgs {
    #[arg(
        long,
        help = "Who reports the activity: claude-code, editor, git, cli, ..."
    )]
    pub source: String,
    #[arg(
        long,
        help = "File path or app name being worked on (default: current directory)"
    )]
    pub entity: Option<String>,
    #[arg(
        long,
        help = "Project name hint (default: git repository or directory name)"
    )]
    pub project: Option<String>,
    #[arg(
        long,
        help = "The activity changed something; bypasses the 2 minute rate limit"
    )]
    pub write: bool,
    #[arg(long, help = "Read a Claude Code hook payload from stdin")]
    pub stdin_claude: bool,
}

#[derive(Debug, Default, PartialEq)]
struct ClaudeHook {
    cwd: Option<String>,
    file: Option<String>,
    prompt: bool,
}

/// Posts one heartbeat and never fails: hooks call this, and a hook that
/// errors or hangs would block the tool that fired it. Any problem becomes a
/// single line on stderr.
pub async fn run(args: HeartbeatArgs) {
    if let Err(e) = send(args).await {
        eprintln!("sablier heartbeat: {e:#}");
    }
}

async fn send(args: HeartbeatArgs) -> Result<()> {
    let hook = if args.stdin_claude {
        read_claude_stdin()
    } else {
        ClaudeHook::default()
    };
    let cwd = match hook.cwd {
        Some(dir) => PathBuf::from(dir),
        None => std::env::current_dir()?,
    };
    let entity = resolve_entity(args.entity.or(hook.file), &cwd);
    let now = chrono::Utc::now();
    let key = state::key(&args.source, &entity);
    let state_path = state::path()?;
    let mut seen = state::load(&state_path);
    if !state::should_send(
        seen.get(&key).copied(),
        now.timestamp(),
        args.write || hook.prompt,
    ) {
        return Ok(());
    }
    let cfg = Config::load()?;
    if cfg.token.is_empty() {
        bail!("not signed in, run `sablier login`");
    }
    let beat = Heartbeat {
        at: now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        project: args
            .project
            .or_else(|| project::detect(Path::new(&entity), &cwd)),
        source: args.source,
        entity,
        write: args.write,
    };
    let client = ApiClient::with_timeout(&cfg.server_url, &cfg.token, Duration::from_secs(3));
    client.heartbeats(&[beat]).await?;
    seen.insert(key, now.timestamp());
    state::save(&state_path, seen, now.timestamp())
}

fn resolve_entity(entity: Option<String>, cwd: &Path) -> String {
    match entity {
        Some(e) if Path::new(&e).is_relative() && cwd.join(&e).exists() => {
            cwd.join(e).display().to_string()
        }
        Some(e) => e,
        None => cwd.display().to_string(),
    }
}

fn read_claude_stdin() -> ClaudeHook {
    let mut raw = String::new();
    if std::io::stdin().read_to_string(&mut raw).is_err() {
        return ClaudeHook::default();
    }
    parse_claude(&raw)
}

fn parse_claude(raw: &str) -> ClaudeHook {
    let payload: Value = serde_json::from_str(raw).unwrap_or(Value::Null);
    let text = |pointer: &str| {
        payload
            .pointer(pointer)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(String::from)
    };
    ClaudeHook {
        cwd: text("/cwd"),
        file: text("/tool_input/file_path").or_else(|| text("/tool_input/notebook_path")),
        prompt: text("/hook_event_name").as_deref() == Some("UserPromptSubmit"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_claude_edit_payload_yields_its_file_and_cwd() {
        let raw = r#"{"hook_event_name":"PostToolUse","cwd":"/repo","tool_input":{"file_path":"/repo/a.rs"}}"#;
        assert_eq!(
            parse_claude(raw),
            ClaudeHook {
                cwd: Some("/repo".into()),
                file: Some("/repo/a.rs".into()),
                prompt: false,
            }
        );
    }

    #[test]
    fn a_prompt_payload_is_flagged_and_malformed_input_is_ignored() {
        assert!(parse_claude(r#"{"hook_event_name":"UserPromptSubmit","cwd":"/r"}"#).prompt);
        assert_eq!(parse_claude("not json {"), ClaudeHook::default());
        assert_eq!(
            parse_claude(r#"{"tool_input":{"file_path":42}}"#),
            ClaudeHook::default()
        );
    }

    #[test]
    fn the_entity_defaults_to_the_working_directory() {
        let cwd = Path::new("/nowhere/repo");
        assert_eq!(resolve_entity(None, cwd), "/nowhere/repo");
        assert_eq!(resolve_entity(Some("Firefox".into()), cwd), "Firefox");
        assert_eq!(resolve_entity(Some("/a/b.rs".into()), cwd), "/a/b.rs");
    }
}
