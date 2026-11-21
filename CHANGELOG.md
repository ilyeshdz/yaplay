# Changelog

## 1.0.0 (2026-11-21)


### Features

* add integration website and automated releases ([#1](https://github.com/ilyeshdz/yaplay/issues/1)) ([d61f4f2](https://github.com/ilyeshdz/yaplay/commit/d61f4f2cc9cec98573fd8057a1a8c4f74a1d3570))
* basic working ([18ba726](https://github.com/ilyeshdz/yaplay/commit/18ba7265ca4a03f5fbba9bba55f2174e2ab1707b))
* working on the overlay ([03656dc](https://github.com/ilyeshdz/yaplay/commit/03656dcafb24fdaad8118f2102e09c8986886d9b))


### Bug fixes

* keep global hotkey manager alive ([eabcc5f](https://github.com/ilyeshdz/yaplay/commit/eabcc5fa50ec5b1e313fff36399562e4ab37fade))


### Automation

* add cross-platform release packaging ([9cbcdbf](https://github.com/ilyeshdz/yaplay/commit/9cbcdbfbb86b4bcaa8d42abd6093b74f1eb81d50))


### Maintenance

* initial commit ([d724cbd](https://github.com/ilyeshdz/yaplay/commit/d724cbda5d53a603d7f698937d78ca14e7452e17))
* prepare first stable release ([a37e91e](https://github.com/ilyeshdz/yaplay/commit/a37e91e193cd3b8386dd0e2951849050193140c2))

<!-- Release Please adds dated version entries above this initial development record. -->

## Before the first release

This section records the initial development baseline before release automation.
Dated version entries appear above it. Each release PR collects changes since
the previous release.

### Features

- Independent Rust/Freya desktop overlay with a voice roster, speaking and mute
  indicators, and temporary notifications.
- Local WebSocket protocol for integrations from any application.
- Display selection, notification preferences, a configurable port, and global
  shortcut controls.
- Native macOS titlebar overlay in the settings window.
- Minimal monochrome Fumadocs website with static export for GitHub Pages.
- Installation, protocol, integration, plugin lifecycle, and troubleshooting guides.
- JavaScript wrapper with automatic reconnect and current-state replay.

### Bug fixes

- Keep the global hotkey manager alive for the lifetime of the process.
- Remove the fake call roster at startup and clear stale state when leaving a call.
- Respect disabled notifications, expire old notifications, and retain at most five.
- Notify only after a successful WebSocket handshake; limit handshake duration
  and message/frame sizes.

### Automation

- Keep PR title edits from restarting code checks, and avoid reporting cancelled
  checks as failures.

- Release Please manages version bumps, changelog entries, tags, and release notes.
- Windows and macOS release binaries are packaged only for an actual release,
  with downloadable SHA-256 checksums.
- Pull requests run formatting, lint, tests, and website validation without a
  desktop release build matrix.
