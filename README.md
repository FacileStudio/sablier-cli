# sablier-cli

Terminal client for Sablier, the self-hosted time
tracker. Ships a single `sablier` binary that is both a full-screen TUI and a set of
scriptable subcommands.

Run it with no arguments for the TUI, or with a subcommand to start, stop, pause and resume
timers straight from a shell.

## What it does

- Launches a ratatui TUI with Timer, Projects and Entries screens
- Starts a timer against a project and task, with an interactive fuzzy picker
- Stops, pauses and resumes the running timer from the shell or the TUI
- Prints the running timer with elapsed time as `HH:MM:SS` and its state
- Lists the projects the token can see
- Creates a task inline from the task picker, then starts a timer on it

## Stack

| Layer | Tech |
|---|---|
| CLI | Rust 2021, clap 4 (derive), tokio 1, anyhow 1 |
| TUI | ratatui 0.29, crossterm 0.28, tui-big-text 0.7, dialoguer 0.11 |
| Transport | reqwest 0.12 (JSON), bearer token auth, 15s request timeout |
| Storage | `~/.sablier.yml` via serde_yaml 0.9 |

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/FacileStudio/sablier-cli/main/install.sh | bash
```

Installs to `~/.local/bin` via [facile](https://github.com/FacileStudio/facile), the suite
installer. Pass `--bin-dir <dir>` to change that, `--source` to build from source, `--no-skill`
to skip AI agent skill registration.

Already have `facile`:

```sh
facile install sablier
```

## Usage

```sh
sablier            # TUI
sablier login      # sign in through the browser
sablier logout     # forget the token, keep the server URL
sablier start      # start a timer, interactive project then task picker
sablier status     # 00:42:17 Running — Facile Suite
sablier stop       # ✓ Stopped — total 00:42:31
sablier pause
sablier resume
sablier projects   # list projects
```

Full command reference, flags and TUI keybindings: [docs/usage.md](docs/usage.md).

## Configuration

All configuration lives in `~/.sablier.yml`. The CLI reads no environment variables and
takes no global flags.

```yaml
server_url: https://sablier.facile.studio/api
token: your-api-token
```

| Key | What it does |
|---|---|
| `server_url` | Base URL prefixed to every request. Must include the `/api` path |
| `token` | Sablier API token, sent as `Authorization: Bearer <token>` |

Run `sablier login --server https://sablier.example.com` and the file is written for you: the
browser opens, the identity provider signs you in, and the token lands in `~/.sablier.yml` with
mode `0600`. A token generated in the dashboard under Profile > API Token still works if you
prefer to paste one. Full reference: [docs/configuration.md](docs/configuration.md).

## Structure

```
src/
  main.rs      clap command tree and the non-interactive subcommand handlers
  config.rs    ~/.sablier.yml loader
  api.rs       Sablier REST client, response models, elapsed-time math
  tui/         event loop, app state, key handlers, ratatui screens
integrations/  SKILL.md, registered with Claude Code and Codex by install.sh
install.sh     one-liner installer
```

## Documentation

| Doc | What's in it |
|---|---|
| [Architecture](docs/architecture.md) | Topology, the flag-based async loop, endpoints used |
| [Configuration](docs/configuration.md) | The config file, every key, and how auth works |
| [Development](docs/development.md) | Building, running against a local Sablier, conventions |
| [Usage](docs/usage.md) | Every command, flag, keybinding and output shape |

---

Part of the [Facile Suite](https://facile.studio) — self-hosted tools for creative studios
and freelancers. One login, zero cloud dependency.
