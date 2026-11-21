# Changelog

## [1.2.0](https://github.com/ilyeshdz/yaplay/compare/v1.1.0...v1.2.0) (2026-11-21)


### Features

* migrate desktop UI to GPUI ([#6](https://github.com/ilyeshdz/yaplay/issues/6)) ([a434d29](https://github.com/ilyeshdz/yaplay/commit/a434d294c1080db2fde779ccd689ae0d39d706b0))

## [1.1.0](https://github.com/ilyeshdz/yaplay/compare/v1.0.0...v1.1.0) (2026-11-21)


### Features

* add Yaplay branding across app and website ([#4](https://github.com/ilyeshdz/yaplay/issues/4)) ([8b81764](https://github.com/ilyeshdz/yaplay/commit/8b8176470596ed3236a2a6dcd158a66aed852c0c))
* redesign the Yaplay website with Geist Pixel ([#5](https://github.com/ilyeshdz/yaplay/issues/5)) ([092222c](https://github.com/ilyeshdz/yaplay/commit/092222c5e2f483f542f9af215c3ad4933b6e5bbd))


### Documentation

* align Yaplay branding with its unofficial Yapin overlay tagline ([706a19a](https://github.com/ilyeshdz/yaplay/commit/706a19a1e3f11029a54e45a4a88de5d8022015e3))
* clarify Yapin integration through Yapmod and Yap++ ([71b9a1f](https://github.com/ilyeshdz/yaplay/commit/71b9a1f7a90daf88f2f265a70f9d831a18c7c8b3))


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

- Rust/Freya desktop overlay built for Yapin with a voice roster, speaking and mute
  indicators, and temporary notifications.
- Local WebSocket protocol for integrations in Yapin client modifications,
  including Yapmod and Yap++.
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
