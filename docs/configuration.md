# sablier-cli — Configuration

Everything the CLI reads at runtime: one YAML file, the OS keyring for the token, and
`XDG_STATE_HOME` for the heartbeat state file.

## The config file

`~/.sablier.yml`, resolved as `dirs::home_dir().join(".sablier.yml")`. The path is fixed —
there is no `--config` flag, no `XDG_CONFIG_HOME` support and no `SABLIER_*` environment
variable anywhere in the source.

```yaml
server_url: https://sablier.facile.studio/api
```

| Key | Required | Default | What it does |
|---|---|---|---|
| `server_url` | yes | — | Base URL every request is appended to. A trailing `/` is trimmed |
| `token` | only without a keyring | `""` | Sent as `Authorization: Bearer <token>` on every request |

With a usable OS keyring the file holds `server_url` only. Without a token in the keyring or
the file, every command fails the `load_authed_config()` check with a message telling you to
sign in.

## Getting a token

```
sablier login
```

This prints an approval link, waits for you to approve it in any browser, and stores the token in
the OS keyring and `server_url` in `~/.sablier.yml` (mode `0600`). `sablier login` with no URL uses `https://sablier.facile.studio`; `sablier login <url>` targets another server. The URL may be the bare host or the API root —
it is normalised to end in `/api`, which is what every later request is appended to. On a
machine that already has a config, `sablier login` alone re-uses the stored `server_url`.

The CLI never handles a password. By default it runs the OAuth device grant against
`https://sso.facile.studio` with the public client `facile-cli`, then trades Registre's access
token for a Sablier token at `/api/auth/oidc/device/exchange`. Registre's token is never written
to disk. Nothing listens on this machine, so the approval link works from any browser, including
over SSH. The exchange only finds accounts that already signed in to the web app once.

`sablier login --listener` uses the loopback flow instead: it opens a loopback port, sends the
browser to `/api/auth/oidc?flow=cli&port=N`, and the API redirects back to that port with a
one-time code valid for sixty seconds and usable once. A server without the device exchange
(it answers 404) falls back to this flow automatically.

A token can still be generated in the dashboard under **Profile > API Token** and pasted into
the file as `token:`. The next command moves it to the keyring.

## Token storage

The token lives in the OS keyring under service `studio.facile.sablier`, account `session`:
Keychain on macOS, Credential Manager on Windows, the Secret Service on Linux. The Sablier
desktop app reads and writes the same entry, so the CLI, the app and its tray share one
sign-in.

- **Migration.** A `token:` still in `~/.sablier.yml` (from an older CLI, `facile login`, or a
  hand edit) moves to the keyring on the next command. The CLI reads the entry back, and
  removes the token from the file only once the keyring returned it.
- **Fallback.** Without a usable keyring, typically headless Linux with no Secret Service, the
  token stays in `~/.sablier.yml`, created `0600` and tightened to `0600` on every read. Each
  command then prints a notice on stderr naming the file.
- **Logout.** `sablier logout` deletes the keyring entry and the `token:` key, and keeps
  `server_url`.

## The `/api` trap

`ApiClient::url` is a plain concatenation:

```rust
format!("{}{}", self.base_url, path)
```

The paths passed in are `/projects`, `/time-entries/running` and friends — with **no** `/api`
prefix. The Sablier server mounts all of them under `/api`. So `server_url` must itself end
in `/api`:

```yaml
server_url: https://sablier.facile.studio/api      # correct
server_url: https://sablier.facile.studio          # every request 404s
```

A wrong `server_url` surfaces as a `404 Not Found` error with the server's HTML or JSON body
appended, not as a configuration error.

## Error messages you will actually see

| Symptom | Cause |
|---|---|
| `cannot read /Users/you/.sablier.yml` | The file does not exist |
| `invalid config at ...` | The YAML is malformed or `server_url` is missing |
| `no API token configured` | No token in the keyring or the file |
| `sablier: no usable OS keyring (...)` | No Secret Service or keychain: the token stays in the file |
| `401 Unauthorized: ...` | The token is wrong, revoked, or from another instance |
| `404 Not Found: ...` | `server_url` is missing the `/api` suffix |
| `error sending request` | The host is unreachable, or the 15-second timeout expired |

## Heartbeat state

`sablier heartbeat` keeps the last-sent time per `(source, entity)` in
`$XDG_STATE_HOME/sablier/heartbeats.json`. When `XDG_STATE_HOME` is unset or not absolute,
the path is `~/.local/state/sablier/heartbeats.json`. Deleting the file only resets the
2-minute rate limit.

## Request behavior

The HTTP client is built once per command with a 15-second timeout covering the whole
request. `sablier heartbeat` uses 3 seconds instead, so a hook never stalls its caller. The CLI configures nothing else on it: no retries, no backoff, no custom headers
beyond the bearer token, and no proxy settings.
