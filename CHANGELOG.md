# Changelog

All notable changes to this project are documented here. The format is
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html). While on
`0.x`, a breaking change bumps the minor.

Every entry below was reconstructed from git history on 2026-08-24, so they
record what shipped rather than what was written down at the time.

## [Unreleased]

### Changed

- The token lives in the OS keyring (Keychain, Credential Manager, Secret Service) under `studio.facile.sablier` / `session`, the entry the Sablier desktop app shares. `~/.sablier.yml` keeps `server_url` only.
- A token already in `~/.sablier.yml` moves to the keyring on the next command, and leaves the file only after the keyring returned it.
- Without a usable keyring, such as headless Linux with no Secret Service, the token stays in `~/.sablier.yml` (mode 0600) and each command says so on stderr.
- `sablier logout` clears the keyring entry and the file's token.

## [0.5.1] - 2026-11-11

### Added

- sablier-cli is released under the [MIT License](LICENSE).

### Fixed

- Dependencies carry the patched quinn-proto, rustls, rustls-webpki, webpki-roots, h2, tokio, rustix, anyhow and lru releases. ratatui moves to 0.30, tui-big-text to 0.8 and crossterm to 0.29.

## [0.5.0] - 2026-11-03

### Changed

- `sablier login` signs in with the OAuth device grant. It prints an approval link to open in any browser and needs no port on this machine, so it works over SSH. Nothing to type or paste.
- The old loopback redirect is behind `sablier login --listener`. A server without `/api/auth/oidc/device/exchange` falls back to it on its own.
- A 401 from the exchange says to sign in once at the web app in a browser, since the exchange never creates an account.

## [0.4.2] - 2026-11-03

### Fixed

- `Cargo.lock` now carries the release version. The v0.4.0 and v0.4.1 tags failed the locked release build and published nothing.

## [0.4.1] - 2026-11-03

### Fixed

- Release version bump. The v0.4.1 tag published nothing, see 0.4.2.

## [0.4.0] - 2026-11-03

### Changed

- `sablier login` with no URL signs in to `https://sablier.facile.studio` instead of failing. The server can also be passed as `sablier login <url>`; `--server` still works.
- The login checks that the server offers single sign-on before opening the browser, prints one line while waiting and one on success, and shows the URL only when no browser opens.
- The page the browser lands on after sign-in matches the suite sign-in page, without a card border.

## [0.3.1] - 2026-11-03

### Fixed

- The release workflow passes `--repo` to `gh release download`, so the Homebrew formula is published again. It had failed since 0.2.0.

## [0.3.0] - 2026-11-03

### Added

- `sablier heartbeat --source <s> [--entity <path>] [--project <name>] [--write] [--stdin-claude]` posts one heartbeat to `POST /api/heartbeats`. It needs Sablier 1.4.0. It always exits 0, waits at most 3 s, and sends at most one heartbeat per source and entity every 2 minutes unless `--write` is set.
- `sablier hooks install claude` and `sablier hooks uninstall claude` add or remove Claude Code hooks in `~/.claude/settings.json`, after backing it up.
- `sablier hooks print git` prints a post-commit hook snippet.

### Changed

- Fixed the clippy 1.98 lints in `keys` and `tui`.

## [0.2.0] - 2026-09-01

### Added

- `sablier keys {list,create,revoke}` command group for managing API keys.
- `sablier keys list` supports filtering keys by application name with `--app`.
- `sablier keys create` creates secret or public API keys with optional `--origins` and `--quota` flags.
- `sablier keys revoke` revokes API keys by id.
- Full `--json` support for all `sablier keys` commands.

## [0.1.1] — 2026-08-10

### Fixed

- Login requires the server to echo the nonce the CLI sent, so another local
  process cannot inject a callback.

### Changed

- `install.sh` bootstraps the `facile` CLI from `get.facile.studio`.

## [0.1.0] — 2026-08-10

### Added

- First release. A TUI and CLI client for Sablier time tracking: browse
  projects, watch the running timer, and `start` a timer from the command line.
- Sign in through the browser, and a `logout` that keeps the server URL so the
  next login does not have to be told again.
- Vim navigation (`g`/`G`) in lists and popups, inline task creation, and
  Tab/Shift-Tab to cycle screens directly.
- A large ASCII timer display, keybind hints under the running timer, and
  footer hints.
- `install.sh`, a `self-update` command and prebuilt binaries published on tag.
- AI agent skill registration.
- Documentation harmonized against the suite standard.

### Changed

- TLS is built with rustls, which is what makes linux/arm64 cross-compile.
- `~/.sablier.yml` is tightened on read if it was left loose.
- The login request goes to the API root rather than to the raw `--server`
  value.
- `upgrade` shows cargo's output instead of hiding it behind `--quiet`.
- Muted text in the timer center and secondary text elsewhere are readable
  against the app palette; the running timer is green to match it.

### Fixed

- API response deserialization matches what the server sends, including the
  task `status` field.
- The `gg` vim keybind, task creation deserialization and the timer color.

[Unreleased]: https://github.com/FacileStudio/sablier-cli/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/FacileStudio/sablier-cli/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/FacileStudio/sablier-cli/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/FacileStudio/sablier-cli/releases/tag/v0.1.0
