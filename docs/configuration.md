# sablier-cli — Configuration

Everything the CLI reads at runtime: one YAML file, two keys, no environment variables.

## The config file

`~/.sablier.yml`, resolved as `dirs::home_dir().join(".sablier.yml")`. The path is fixed —
there is no `--config` flag, no `XDG_CONFIG_HOME` support and no `SABLIER_*` environment
variable anywhere in the source.

```yaml
server_url: https://sablier.facile.studio/api
token: your-api-token
```

| Key | Required | Default | What it does |
|---|---|---|---|
| `server_url` | yes | — | Base URL every request is appended to. A trailing `/` is trimmed |
| `token` | no in YAML, yes in practice | `""` | Sent as `Authorization: Bearer <token>` on every request |

`token` is declared `#[serde(default)]`, so the file parses without it — but every command
then fails the `load_authed_config()` check with a message telling you to generate one.

## Getting a token

Generate it in the Sablier dashboard under **Profile > API Token**, then paste it into
`~/.sablier.yml`. The CLI has no `login` command, never writes the config file, and never
prompts for credentials.

## Token storage

The token is stored in plaintext in `~/.sablier.yml`. The CLI does not use the OS keychain,
does not encrypt the file, and does not chmod it. If that matters on a shared machine,
restrict it yourself:

```sh
chmod 600 ~/.sablier.yml
```

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
| `No API token configured.` | `token` is absent or empty |
| `401 Unauthorized: ...` | The token is wrong, revoked, or from another instance |
| `404 Not Found: ...` | `server_url` is missing the `/api` suffix |
| `error sending request` | The host is unreachable, or the 15-second timeout expired |

## Request behavior

The HTTP client is built once per command with a 15-second timeout covering the whole
request. The CLI configures nothing else on it: no retries, no backoff, no custom headers
beyond the bearer token, and no proxy settings.
