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
