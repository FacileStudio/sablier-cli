---
name: sablier
description: >
  Facile time tracking CLI. Use when the user asks to track time,
  start/stop timers, view time entries, or manage API keys.
---

# sablier — Facile time tracking

Binary: `sablier`
Config: `~/.sablier.yml`

## When to apply

Use when the user mentions time tracking, timers, time entries, API keys, or Sablier.
Triggers: "track time", "start timer", "stop timer", "pause timer", "time tracking", "timesheet", "sablier", "api key", "sablier keys"

## Commands

### Timer
```
sablier start                  Start timer (interactive project/task picker)
  --project-id <id>           Skip picker
  --task-id <id>              Requires --project-id
sablier stop                   Stop running timer
sablier pause                  Pause running timer
sablier resume                 Resume paused timer
sablier status                 Show current timer
```

### Session
```
sablier login                  Sign in through the browser
  --server <url>              Sablier instance URL
sablier logout                 Forget the stored token, keep the server URL
```

### Projects
```
sablier projects               List available projects
```

### Keys
```
sablier keys create --app <name> [--public] [--origins <urls>] [--quota <n>]
sablier keys list [--app <name>]
sablier keys revoke <id> [--yes]
```

### TUI mode
```
sablier                        Launch interactive TUI (no args)
```

### Self-upgrade
```
sablier upgrade
```

## Rules
- Timer states: Running ↔ Paused → Stopped
- `start` without flags opens interactive fuzzy-search picker
- Status shows elapsed time as HH:MM:SS
- All keys commands support `--json` for machine-readable output
- Run `sablier -h` for exact syntax when unsure
