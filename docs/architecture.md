# sablier-cli — Architecture

How the `sablier` binary is wired: the two modes it runs in, the async pattern the TUI uses,
and every Sablier endpoint it talks to.

## Topology

```
                     ~/.sablier.yml
                     (server_url + token)
                            │
                            ▼
  Terminal ──▶ sablier binary ──┬──▶ TUI mode      (no subcommand)
                                └──▶ command mode  (start/stop/pause/...)
                            │
                            │  reqwest, Authorization: Bearer <token>
                            ▼
                    Sablier Go API  (server_url, e.g. https://sablier.facile.studio/api)
                            │
                       PostgreSQL 16
```

There is no local database, no cache and no daemon. Every command opens a fresh
`reqwest::Client` with a 15-second timeout, makes its calls, and exits.

## Two modes, one binary

`src/main.rs` parses the command tree with clap. `Cli.command` is an `Option<Command>`:

- `None` — `tui::run()` takes over the terminal (raw mode, alternate screen).
- `Some(cmd)` — `run_command()` dispatches to a small `cmd_*` async function that prints one
  or two lines and returns.

Both paths start by loading `~/.sablier.yml`. Command mode goes through
`load_authed_config()`, which additionally fails with a setup hint when `token` is empty;
`tui::run()` performs the same check inline before entering raw mode.

## Command-mode lifecycle

1. `config::Config::load()` reads and deserializes `~/.sablier.yml`; a missing file produces
   an error naming the path and how to create it.
2. `api::ApiClient::new(server_url, token)` trims a trailing `/` from the base URL and builds
   the HTTP client.
3. One or more requests run. `ApiClient::check` inspects the status: on failure it tries to
   deserialize `{"error": {"message": ...}}` and reports `<status>: <message>`, otherwise it
   reports the raw body.
4. The result is printed to stdout. Errors bubble up through `anyhow` to `main`.

## TUI lifecycle

`src/tui/mod.rs` runs a single synchronous loop that never awaits inside itself:

```
loop {
  terminal.draw(ui::render)          render sidebar + screen + footer + popup
  events.next()                      500 ms poll: Key(..) or Tick
    Key   -> handlers::handle_key    mutates App, sets needs_* flags
    Tick  -> app.tick()              decrements the status-message TTL
  for each needs_* flag set:
      tokio::spawn(api call)         one JoinHandle per category
  for each JoinHandle:
      if handle.is_finished()        await it, fold the result into App
  if app.should_quit { break }
}
```

The flags live on `App` (`needs_initial_load`, `needs_timer_refresh`, `needs_stop`,
`needs_pause`, `needs_resume`, `needs_start`, `needs_tasks_load`, `needs_create_task`,
`needs_entries_load`). Keyboard handlers only set flags; they never perform I/O. Each
category holds at most one in-flight `JoinHandle`, so a held-down key cannot fan out into
concurrent requests.

The 500 ms tick doubles as the clock refresh: the timer screen recomputes elapsed time from
`started_at` locally, so the display advances without polling the server.

## Screens and state

`App.screen` is one of `Timer`, `Projects`, `Entries`; `App.popup` is `None` or one of
`PickProject`, `PickTask`, `CreateTask`. The layout is a 22-column sidebar, the active
screen, and a one-line footer that shows an error, a status message, or context-dependent
key hints. Popups render on top of everything.

Task names are cached in `App.task_names` (a `HashMap<i64, String>`) as task lists load, so
the entries screen can label entries whose payload omits `task_name`.

## Endpoints used

All paths are appended verbatim to `server_url`, which is why that value must already end in
`/api`.

| Method | Path | Used by |
|---|---|---|
| `GET` | `/users/me` | TUI initial load |
| `GET` | `/projects` | `projects`, `start`, `status`, TUI initial load |
| `GET` | `/projects/{id}/tasks` | `start`, TUI task picker |
| `POST` | `/projects/{id}/tasks` | TUI create-task popup |
| `GET` | `/time-entries/running` | `status`, TUI timer refresh |
| `GET` | `/time-entries?user_id={id}` | TUI entries screen |
| `POST` | `/time-entries/start` | `start`, TUI |
| `POST` | `/time-entries/stop` | `stop`, TUI |
| `POST` | `/time-entries/pause` | `pause`, TUI |
| `POST` | `/time-entries/resume` | `resume`, TUI |

Every request carries `Authorization: Bearer <token>`. There is no login endpoint and no
token refresh — the token is long-lived and comes from the dashboard.

## Data model

The client mirrors only what it renders:

- `User` — `id`, `name`, `email`, `color`
- `Project` — `id`, `name`, `description`
- `Task` — `id`, `name`, `status` (defaults to `to-do`), `project_id`
- `TimeEntry` — `id`, `project_id`, `task_id`, `task_name`, `user_id`, `started_at`,
  `stopped_at`, `paused_at`, `paused_duration_ms`

`Task.id`, `Task.project_id` and `User.id` go through a `flexible_i64` deserializer because
the Go backend serializes some IDs as JSON strings and others as numbers.

Elapsed time is computed client-side in `TimeEntry::elapsed_seconds`: the end bound is
`stopped_at`, else `paused_at`, else now; `paused_duration_ms / 1000` is subtracted and the
result clamped at zero. `status_label()` derives `Stopped` / `Paused` / `Running` from the
same three timestamps.

## Suite integration

The CLI is a plain REST consumer. It does not use `pool`, `enveloppe` or Journal, and it does
not participate in OIDC — authentication is the dashboard-issued API token only.
