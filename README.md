# sablier

Terminal client for [Sablier](https://github.com/FacileStudio/sablier) time tracking.

Full TUI with vim-style navigation, big timer display, and project/task management — or use it as a quick CLI.

## Install

```sh
cargo install --path .
```

## Setup

Create `~/.sablier.yml`:

```yaml
server_url: https://your-instance.example.com
token: your-api-token
```

Generate your token at **Profile > API Token** in the Sablier dashboard.

## Usage

### TUI

```sh
sablier
```

| Key | Action |
|-----|--------|
| `n` | New timer |
| `s` | Stop |
| `p` | Pause |
| `r` | Resume / Refresh |
| `j/k` | Navigate lists |
| `g/G` | Jump to top/bottom |
| `Tab` | Switch screens |
| `1/2/3` | Jump to Timer/Projects/Entries |
| `q` | Quit |

### CLI

```sh
sablier start                  # interactive project/task picker
sablier start --project-id 1 --task-id 2
sablier status
sablier stop
sablier pause
sablier resume
sablier projects
```

## License

MIT
