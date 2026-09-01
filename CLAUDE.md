# sablier-cli

Terminal client for [Sablier](https://github.com/FacileStudio/sablier) time tracking. Provides both a full-screen TUI (ratatui) and a set of CLI subcommands for starting, stopping, pausing, and resuming timers against a Sablier server.

## Tech stack

- Language: Rust (edition 2021)
- Async runtime: Tokio
- HTTP client: reqwest (JSON feature)
- TUI framework: ratatui + crossterm, tui-big-text for the timer display
- CLI argument parsing: clap (derive)
- Config format: YAML (`~/.sablier.yml`) via serde_yaml
- Interactive selection (CLI mode): dialoguer (fuzzy-select)

## Key commands

```sh
cargo build                # debug build
cargo build --release      # optimized build (LTO + strip enabled)
cargo run                  # launch TUI
cargo run -- start         # start timer via CLI
cargo run -- status        # show running timer
cargo run -- stop          # stop running timer
cargo run -- pause         # pause running timer
cargo run -- resume        # resume paused timer
cargo run -- projects      # list projects
cargo run -- keys list     # list API keys
cargo run -- keys create   # create API key
cargo run -- keys revoke   # revoke API key
cargo run -- upgrade       # self-update from GitHub
```

## Installation

```sh
# one-liner install (requires cargo + git)
curl -fsSL https://raw.githubusercontent.com/FacileStudio/sablier-cli/main/install.sh | bash

# or manually
cargo install --git https://github.com/FacileStudio/sablier-cli.git --force
```

## Project structure

```
src/
  main.rs          CLI entry point, clap definition, subcommand handlers
  config.rs        Loads ~/.sablier.yml (server_url + token)
  api.rs           HTTP client for the Sablier REST API (reqwest, bearer auth)
  keys.rs          API key management command handlers (list, create, revoke)
  tui/
    mod.rs         TUI entry point, async event loop, background task orchestration
    app.rs         App state struct, screen enum, popup enum
    events.rs      Crossterm event polling (key events + tick)
    handlers.rs    Keyboard input dispatch (global, per-screen, popup, create-task)
    ui/
      mod.rs       Top-level render function, layout
      timer.rs     Timer screen (big-text clock, status)
      projects.rs  Projects list screen
      entries.rs   Time entries list screen
      sidebar.rs   Left sidebar / menu
      footer.rs    Bottom status bar, keybinding hints
      popup.rs     Modal popups (pick project, pick task, create task)
      theme.rs     Color/style constants
install.sh         One-liner installer script
```

## Configuration

The CLI reads `~/.sablier.yml`:

```yaml
server_url: https://your-instance.example.com
token: your-api-token
```

`sablier login` writes this file: it opens a loopback port, sends the browser to
`/api/auth/oidc?flow=cli&port=N`, and trades the one-time code the API redirects back with for a
token at `/api/auth/oidc/exchange`. That flow is porte's, shared with the rest of the suite.
Generating a token by hand in the dashboard under Profile > API Token still works.

`server_url` must include `/api`, as `api::url` appends paths to it verbatim. `login::api_base`
normalises whatever is passed to `--server`, which is why the bare host is accepted there and
nowhere else.

## Architecture notes

- The TUI uses a flag-based async task pattern: `App` fields like `needs_stop`, `needs_pause`, `needs_start`, etc. are set by keyboard handlers, and the main loop in `tui/mod.rs` spawns tokio tasks when it sees those flags. Results are polled each tick (500ms).
- The API client handles a quirk where the Go backend serializes some IDs as strings; `flexible_i64` is a custom serde deserializer that accepts both integer and string-encoded IDs.
- Release profile enables LTO and symbol stripping for a small binary.
- Test suite is executed with `cargo test` and style/architecture is verified with `filet check .`.

## Conventions

- No inline comments in code.
- Remove dead code.
