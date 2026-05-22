# sablier

Terminal client for [Sablier](https://github.com/FacileStudio/sablier) time tracking.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/FacileStudio/sablier-cli/main/install.sh | bash
```

## Setup

Add your API token to `~/.sablier.yml`:

```yaml
server_url: https://your-instance.example.com
token: your-api-token
```

## Usage

```sh
sablier            # launch TUI
sablier start      # start timer (interactive picker)
sablier status     # show running timer
sablier stop       # stop timer
sablier pause      # pause timer
sablier resume     # resume timer
sablier upgrade    # update to latest version
```

### Keybindings

| Key | Action |
|-----|--------|
| `n` | New timer |
| `s/p/r` | Stop / Pause / Resume |
| `j/k` | Navigate |
| `g/G` | Top / Bottom |
| `Tab` | Switch screen |
| `q` | Quit |
