# sablier-cli — Development

Building the binary, running it against a Sablier instance, and the conventions this repo
holds itself to.

## Prerequisites

- Rust with edition 2021 support and `cargo` on `PATH` (install via [rustup](https://rustup.rs))
- A reachable Sablier instance and an API token, local or remote
- A terminal that handles the alternate screen and raw mode, for TUI work

There is no `mise.toml`, no `Makefile`, no `scripts/check.sh` and no `.githooks/` in this
repo. Cargo is the entire toolchain.

## Setup

```sh
git clone https://github.com/FacileStudio/sablier-cli.git
cd sablier-cli
cargo build
```

Point the config at whatever instance you are developing against. A local Sablier API
usually means:

```yaml
server_url: http://localhost:8080/api
token: your-api-token
```

The `/api` suffix is load-bearing — see [configuration.md](configuration.md).

## Running

```sh
cargo run                        # TUI
cargo run -- status              # a single command
cargo run -- start --project-id 4
cargo run -- --help
```

`cargo run` and the installed `sablier` binary read the same `~/.sablier.yml`; there is no
way to point a dev build at a different config file short of editing that one.

## Building a release binary

```sh
cargo build --release
```

The release profile sets `lto = true` and `strip = true`, so the binary is small but carries
no symbols — build in debug when you need a usable backtrace.

## Tests and lints

There is no test suite, no `tests/` directory, no linter configuration and no CI workflow.
The checks available are the ones cargo ships:

```sh
cargo check
cargo clippy
cargo fmt --check
```

Anything non-trivial you add should come with at least one runnable check — the API
response-parsing helpers (`flexible_i64`, `TimeEntry::elapsed_seconds`) are pure functions
and the obvious first candidates.

## Where things live

| Path | What it holds |
|---|---|
| `src/main.rs` | The clap command tree and one `cmd_*` handler per subcommand |
| `src/config.rs` | `Config::path()` and `Config::load()` for `~/.sablier.yml` |
| `src/api.rs` | `ApiClient`, response structs, `flexible_i64`, elapsed-time math |
| `src/tui/mod.rs` | Terminal setup and the render / event / spawn / collect loop |
| `src/tui/app.rs` | `App` state, `Screen`, `Popup`, the `needs_*` flags |
| `src/tui/events.rs` | Crossterm polling at the tick rate |
| `src/tui/handlers.rs` | Key dispatch: global, per-screen, popup, create-task input |
| `src/tui/ui/` | One module per screen plus `sidebar`, `footer`, `popup`, `theme` |
| `integrations/SKILL.md` | The AI agent skill the installer registers |
| `install.sh` | Clone, `cargo install --path`, register the skill |

## Adding a command

1. Add a variant to `enum Command` in `src/main.rs` with a clap `about`.
2. Add its arm to `run_command`.
3. Write an async `cmd_*` function that calls `load_authed_config()` first.
4. Add the endpoint to `ApiClient` in `src/api.rs` if it does not exist yet.
5. Document it in [usage.md](usage.md) and, if an assistant should know about it, in
   `integrations/SKILL.md`.

## Adding a TUI action

Keyboard handlers must not perform I/O. Set a `needs_*` flag on `App` in
`src/tui/handlers.rs`, then handle that flag in the loop in `src/tui/mod.rs` by spawning a
tokio task and folding its result back into `App` once the handle finishes. Each category
keeps at most one in-flight handle, which is what stops a repeated keypress from firing
concurrent requests.

## Conventions

- No inline comments. Names and structure carry the meaning.
- Remove dead code. `src/api.rs` still opens with `#![allow(dead_code)]`, which is a debt to
  pay down, not a pattern to copy.
- Commit messages are plain imperative sentence case.
