# Changelog

All notable changes to this project are documented here. The format is
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html). While on
`0.x`, a breaking change bumps the minor.

Every entry below was reconstructed from git history on 2026-08-24, so they
record what shipped rather than what was written down at the time.

## [Unreleased]

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
