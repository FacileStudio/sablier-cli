---
name: sablier
description: >
  Facile time tracking CLI. Use when the user asks to track time,
  start/stop timers, or view time entries.
---

# sablier — Facile time tracking

Binary: `sablier`
Config: `~/.sablier.yml`

## When to apply

Use when the user mentions time tracking, timers, time entries, or Sablier.
Triggers: "track time", "start timer", "stop timer", "pause timer", "time tracking", "timesheet", "sablier"

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

### Projects
```
sablier projects               List available projects
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
- Run `sablier -h` for exact syntax when unsure
