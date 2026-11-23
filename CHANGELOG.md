# Changelog

## [Unreleased]

## [1.6.0] (2026-11-23)

### Added

- Call integrations can introduce themselves by name and optional plugin version in the [protocol v2 handshake](site/content/docs/protocol.mdx#protocol-v2). Yaplay replies with the actions it accepts on that connection.
- The client card at the top of **Settings → General** groups the integration name, version, and call status together, with supported controls directly underneath. When waiting for an integration, Settings shows the listening port. [Connection troubleshooting](site/content/docs/troubleshooting.mdx#my-client-wont-connect)
- [Roster-only integrations](site/content/docs/protocol.mdx#protocol-v2) can identify themselves and supply call state with an empty action list.

- Show [speakers only](site/content/docs/appearance.mdx#speaker-only-roster), with an adjustable delay before quiet speakers disappear.
- Choose the [notification duration, card limit, and title-only display](site/content/docs/appearance.mdx#notifications).
- Record a [global shortcut](site/content/docs/appearance.mdx#global-shortcuts) for the controls and optional mute and deafen shortcuts for compatible calls.

- Drag the controls, voice panel, and notifications anywhere on screen. Choose **Move panels freely** in settings or the move button in the controls, and Yaplay remembers each panel's position and display for the next launch.

### Changed

- The overlay shows only the call buttons a v2 integration supports. A mute-only integration gets a microphone button, and the [global mute and deafen shortcuts](site/content/docs/appearance.mdx#global-shortcuts) follow the same capabilities.
- The [JavaScript wrapper](site/content/docs/integration.mdx#handle-overlay-controls) negotiates capabilities on reconnect. On older Yaplay versions, integrations with all three actions can fall back to v1 controls, while partial-control integrations continue publishing state. [Compatibility table](site/content/docs/protocol.mdx#compatibility)

- [Settings](site/content/docs/appearance.mdx) now has a sidebar for General, Overlay, Appearance, and Updates. The current section stays highlighted as you hover over the sidebar, and Save or Start stays within reach while you edit.

- Yaplay appears in the Windows taskbar and macOS Dock only while [Settings](site/content/docs/appearance.mdx) is open.

## [1.5.0] (2026-11-22)

### Changed

- Release notes are now written in each change PR. New releases become public after signed downloads are ready and are announced on Yapin.

## [1.4.0](https://github.com/ilyeshdz/yaplay/compare/v1.3.0...v1.4.0) (2026-11-21)


### Features

* add configurable start at login ([#13](https://github.com/ilyeshdz/yaplay/issues/13)) ([ed7dec2](https://github.com/ilyeshdz/yaplay/commit/ed7dec2a7a1342ef57e747e17e22a83cf2a2b788))


### Bug fixes

* hide console window on Windows ([7a3450d](https://github.com/ilyeshdz/yaplay/commit/7a3450d63f7774a10256e47e06c90651181becab))
* hide console window on Windows ([#12](https://github.com/ilyeshdz/yaplay/issues/12)) ([3a7173a](https://github.com/ilyeshdz/yaplay/commit/3a7173a1fee047dabee29ad837ad025aac9c9e7b))

## [1.3.0](https://github.com/ilyeshdz/yaplay/compare/v1.2.0...v1.3.0) (2026-11-21)


### Features

* add direct downloads and release notes ([#8](https://github.com/ilyeshdz/yaplay/issues/8)) ([7d9104e](https://github.com/ilyeshdz/yaplay/commit/7d9104e310590a9fa296c98e7f0248d5efeafd89))
* add signed automatic updates ([#9](https://github.com/ilyeshdz/yaplay/issues/9)) ([e51d7ed](https://github.com/ilyeshdz/yaplay/commit/e51d7ed12eba645957a12cbe831cd557c3ee5e03))

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
