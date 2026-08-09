# sablier-cli — Usage

The complete command reference: every subcommand, every flag, the TUI keybindings, and the
AI agent skill the installer registers.

## Synopsis

```sh
sablier [COMMAND]
```

`sablier` with no command launches the TUI. `sablier --help` and `sablier <command> --help`
print clap's generated help. There is no `--version` flag, no global flags, and no
`--config` override.

Every command except `upgrade` requires `~/.sablier.yml` with a non-empty `token` — see
[configuration.md](configuration.md).

## `sablier`

Launch the TUI. Loads the current user, the running timer and the project list, then enters
the alternate screen.

```sh
sablier
```

Fails before touching the terminal if the config file is missing or the token is empty.

## `sablier login`

Sign in through the browser and write the token to `~/.sablier.yml`.

| Flag | Type | Default | What it does |
|---|---|---|---|
| `--server <SERVER>` | `string` | the stored `server_url` | Sablier instance, e.g. `https://sablier.facile.studio` |

```sh
sablier login --server https://sablier.facile.studio
```

The CLI opens a loopback port, sends the browser to `/auth/oidc`, and exchanges the one-time
code the API redirects back with. The file is created mode `0600`.

## `sablier logout`

Forget the stored token. `server_url` is kept, so signing back in does not mean retyping
which Sablier this is.

```sh
sablier logout
```

```
✓ Signed out. Token removed from /Users/you/.sablier.yml
```

Prints `▸ Not signed in` when there was no token, and fails when there is no config file at
all.

## `sablier start`

Start a timer. With no flags it opens a fuzzy-search picker for the project, then one for
its tasks.

| Flag | Type | Default | What it does |
|---|---|---|---|
| `--project-id <PROJECT_ID>` | `i64` | — | Skip the project picker |
| `--task-id <TASK_ID>` | `i64` | — | Skip the task picker; requires `--project-id` |

```sh
sablier start
sablier start --project-id 4
sablier start --project-id 4 --task-id 137
```

Behavior by flag combination:

- neither flag — pick a project, then a task
- `--project-id` only — pick a task within that project
- both flags — start immediately, no prompt
- `--task-id` only — the task ID is **ignored** and the full project-then-task picker opens

On success it prints `Timer started — <project name>`. It bails with `No projects found` or
`No tasks found for project "<name>"` when a picker would be empty.

## `sablier status`

Print the running timer.

```sh
sablier status
```

```
00:42:17 Running — Facile Suite
```

The format is `<HH:MM:SS> <Running|Paused|Stopped> — <project name>`. With no timer it
prints `No timer running.` The project name falls back to `?` if the project list cannot be
fetched.

## `sablier stop`

Stop the running timer and print the total.

```sh
sablier stop
```

```
Stopped. Total: 00:42:31
```

## `sablier pause`

Pause the running timer. Prints `Timer paused.`

```sh
sablier pause
```

## `sablier resume`

Resume a paused timer. Prints `Timer resumed.`

```sh
sablier resume
```

## `sablier projects`

List every project the token can see, two spaces indented, with the description appended
after an em dash when the project has one.

```sh
sablier projects
```

```
  Facile Suite  — Internal tooling
  Client work
```

Prints `No projects.` when the list is empty.

## `sablier upgrade`

Reinstall the binary from GitHub. Shells out to
`cargo install --git https://github.com/FacileStudio/sablier-cli.git --force`, so `cargo`
must be on `PATH`. This is the only command that does not read the config file.

```sh
sablier upgrade
```

## TUI keybindings

Three screens — Timer, Projects, Entries — with a sidebar, a hint footer, and modal popups.

### Global

| Key | Action |
|---|---|
| `q` | Quit |
| `Ctrl-C` | Quit, from anywhere including popups |
| `Tab` | Next screen (Timer → Projects → Entries → Timer) |
| `Shift-Tab` | Previous screen |
| `1` / `2` / `3` | Jump to Timer / Projects / Entries |

Screen keys are inert while a popup is open.

### Timer screen

| Key | Action |
|---|---|
| `n` | Open the project picker to start a new timer |
| `s` | Stop the running timer |
| `p` | Pause the running timer |
| `r` | Resume when paused; refresh the timer when nothing is running |

The elapsed clock is recomputed locally every 500 ms tick, so it advances without polling.

### Projects and Entries screens

| Key | Action |
|---|---|
| `j` / `Down` | Move down |
| `k` / `Up` | Move up |
| `gg` | Jump to the top |
| `G` | Jump to the bottom |
| `r` | Reload the list (Entries only) |

Entries load on first visit and list the current user's time entries.

### Popups

| Key | Action |
|---|---|
| `j` / `k` / `Down` / `Up` | Move the selection |
| `gg` / `G` | Top / bottom |
| `Enter` | Confirm |
| `Esc` | Back — from the task picker to the project picker, otherwise close |

The task picker carries one extra row past the last task: **+ New Task**. Selecting it opens
a text input; type a name, press `Enter` to create the task and immediately start a timer on
it, or `Esc` to go back to the task list. `Backspace` edits the input.

## AI agent skill

`install.sh` registers `integrations/SKILL.md` with whichever assistants it finds on `PATH`:

- `claude` present — copies the file to `~/.claude/skills/sablier/SKILL.md` and injects its
  contents into `~/.claude/CLAUDE.md`
- `codex` present — injects the same contents into `~/.codex/AGENTS.md`

Injection is idempotent: the block is fenced by `<!-- sablier:start -->` and
`<!-- sablier:end -->` markers, and a rerun strips the old block before appending the new
one. Neither file is created unless the corresponding binary exists. To opt out, install with
`cargo install --git` instead of the script; to remove it later, delete the marked block and
the skill directory.
