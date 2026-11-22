<div align="center">
  <img src="assets/branding/banner.svg" width="870" alt="Yaplay" />
  <h1>Yaplay</h1>
  <p>An unofficial overlay for Yapin. Keep your call activity and notifications above your windows.</p>
  <p>
    <a href="https://github.com/ilyeshdz/yaplay/releases"><img src="https://img.shields.io/github/v/release/ilyeshdz/yaplay?color=ffb49b" alt="Latest release" /></a>
    <img src="https://img.shields.io/badge/Rust-GPUI-ffb49b" alt="Built with Rust and GPUI" />
    <img src="https://img.shields.io/badge/platforms-macOS%20%7C%20Windows-ffb49b" alt="macOS and Windows" />
  </p>
  <p><a href="https://github.com/ilyeshdz/yaplay/releases">Download</a> · <a href="https://ilyeshdz.github.io/yaplay/">Website</a> · <a href="https://ilyeshdz.github.io/yaplay/docs/">Documentation</a></p>
</div>

## Getting started

1. Download the build for your platform from [GitHub Releases](https://github.com/ilyeshdz/yaplay/releases).
2. Open Yaplay and press **Start** to save your settings and start the overlay.
3. Connect a compatible Yaplay integration from your modified Yapin client. Use **Cmd+Shift+Y** on macOS or **Ctrl+Shift+Y** on Windows to open the controls.

## About Yaplay

Yaplay shows your Yapin call roster, speaking activity, mute indicators, and notifications on top of your other windows. It is built in Rust with GPUI.

Yaplay connects to Yapin through client modification software such as **Yapmod** or **Yap++**. A compatible integration in the modified client forwards Yapin’s call state and notifications to Yaplay over a local WebSocket. Yapin handles the call and audio; Yaplay displays the overlay.

The integration can be a plugin, a wrapper, or a built-in feature of the client modification. This repository provides the overlay, the protocol, and an example wrapper for building that connection; it does not include a ready-to-install Yapmod or Yap++ plugin.

## Website and integration guide

The website uses **Fumadocs + Next.js static export** in [`site/`](site/). Documentation lives in [`site/content/docs/`](site/content/docs/), and the homepage is [`site/app/page.tsx`](site/app/page.tsx). The output is plain static files: no Node.js server is needed in production.

A reusable JavaScript wrapper is available in [`examples/yaplay-client.mjs`](examples/yaplay-client.mjs). It caches current state, reconnects automatically, and replays state without replaying old notifications. It is an example transport adapter, not a host-specific plugin API.

Use Node.js 22 or newer:

```sh
cd site
npm ci
npm run dev
```

Open `http://localhost:3000`. To build the static website:

```sh
npm run build
```

The generated files are in `site/out/`. For a GitHub Pages project path, build with `NEXT_PUBLIC_BASE_PATH=/yaplay npm run build`. The Website workflow derives this path from the repository name, builds the site, and deploys `site/out/`. Select **GitHub Actions** in **Settings → Pages**, then run the workflow. The expected URL is `https://ilyeshdz.github.io/yaplay/` once deployed. For a custom domain at the root, set the repository variable `PAGES_BASE_PATH` to `/`.

The build copies the wrapper into the static output for download. The `/releases/` page renders versioned Release Please entries from `CHANGELOG.md`. Download buttons detect the operating system and link directly to the latest GitHub release assets; macOS releases support Apple Silicon only. Run `npm test` and `npm run typecheck` inside `site/`, `node --test examples/yaplay-client.test.mjs` at the repository root, and `cargo test --locked` for Rust behavior.

## What it does

On first launch, press Start in the settings window to save your preferences. Future launches start the overlay automatically. Yaplay starts a local WebSocket server, closes the settings window, and opens a transparent overlay. The roster is in the top left and notifications are in the top right. The overlay is click through, so it does not block the windows below it. Open the control panel with Cmd+Shift+Y on macOS or Ctrl+Shift+Y on Windows, then use the settings icon to reopen settings. Save applies changes without restarting Yaplay.

Press `Ctrl+Shift+Y` on Windows or `Cmd+Shift+Y` on macOS to show the control panel. It has mute, deafen, leave-call, settings, and exit icons. With a control-capable integration, the mic, deafen, and leave-call buttons send requests to Yapin. The overlay updates when the client reports the resulting state. Call controls are disabled without a negotiated client. Exit still closes Yaplay.

To launch Yaplay when you sign in, enable **Start Yaplay when I sign in** in settings. This is off by default and applies immediately for your user account; turn it off in the same place. Install Yaplay in a permanent location first (Applications on macOS). If you move it, turn the option off and on from the new location. The setting uses a macOS LaunchAgent or a Windows startup entry, and is read from the operating system whenever settings open. Disable it before deleting Yaplay.

Settings include compact rows, scale, panel width, screen corner and margin, colors, opacity, rounded corners, avatar/name visibility, and a visible-member limit. Large calls prioritize speakers and show a `+N more` count within the screen bounds.

Yaplay listens on `127.0.0.1:<port>`. The default port is `8080`.

## Updates

Official builds check GitHub for stable updates shortly after startup and every
six hours. Open Settings to see the version and check manually. The settings
icon is highlighted when an update is available. Choose **Download and install**,
then **Restart Yaplay** when ready. Yaplay never downloads or restarts without
an action, and installation/restart buttons are disabled during a call.

Turn off **Automatically check for updates** and press **Save** to disable
background checks. Downloads require a SHA-256 checksum and a version-bound
minisign signature. Keep the app open while installation is in progress.

Updates use [gpui-updater](https://github.com/AprilNEA/gpui-updater): macOS replaces
the complete app from the DMG; Windows replaces the portable executable while
preserving its filename. The installation must be writable. Move the macOS app
out of its DMG before updating. Development builds and builds without the release
verification key offer manual downloads instead. Existing releases without this
feature must be updated manually once.

See [the build guide](site/content/docs/building.mdx#update-signing) for the signing setup required
before publishing updater-enabled builds. Replacement errors are handled by the
library; recovery after power loss or a crash in the new version is not automatic.

## Build

You need Rust 1.98 or newer. On Windows you also need the C++ Build Tools with the Windows SDK. On macOS you need the Xcode Command Line Tools.

```sh
cargo run --release
```

To build without running:

```sh
cargo build --release --locked
```

The binary is written to `target/release/`. The GitHub Actions workflow builds Windows and macOS releases:

- Windows: `yaplay-windows-x86_64.exe`
- macOS Apple Silicon: `Yaplay-aarch64-apple-darwin.dmg` and `Yaplay-aarch64-apple-darwin.app.zip`

Release Please opens a release PR on `master` with the next version and a [changelog](CHANGELOG.md). Merging that PR creates the tag and a draft GitHub Release, then builds, signs, and attaches these files before publishing. Desktop release packaging never runs on ordinary pushes or pull requests. Open the DMG and drag Yaplay to Applications, or unzip the app bundle and run it directly.

## Release lifecycle and checks

Write concise user-facing bullets under `## [Unreleased]` in `CHANGELOG.md` in the same PR as each product change. Group notes under `### Added`, `### Changed`, `### Fixed`, or `### Removed` and use inline links. Checks require updated notes for `src/` and the JavaScript client wrapper; docs, tests, and tooling-only PRs do not require entries. Add notes for user-visible dependency and packaging changes too.

Use Conventional Commit PR titles and squash-merge: `fix:` selects a patch, `feat:` a minor, and `!` or `BREAKING CHANGE:` a major. Release Please keeps one rolling PR on `master`, synchronizes `Cargo.toml` and `Cargo.lock`, and moves the authored notes under a dated version heading. An empty Unreleased section produces no release. Do not bump versions or move entries manually. Correct notes through normal PRs against master; direct edits to the rolling branch are replaced on refresh.

Review the release PR and merge after its checks pass. The workflow creates a draft and stable tag, packages and signs the downloads, uploads the checksum marker last, then makes the release public and announces it on Yapin. Published notes come from the changelog at the approved tag. This uses a pinned Release Please library and custom Rust strategy in `.github/release`; the stock action does not load that strategy.

Enable **Settings → Actions → General → Allow GitHub Actions to create and approve pull requests**. Set the Actions secret `YAPIN_RELEASE_WEBHOOK_URL` to the Yaplay webhook and variable `YAPIN_RELEASE_ROLE_ID` to its role ID. Announcements mention only that role, include the notes and release link, and shorten long notes at complete lines. The built-in token explicitly dispatches checks and packaging and calls the announcement workflow after publication.

For failed packaging, rerun failed jobs or dispatch **Release assets** from `master` with the existing tag. Existing assets remain immutable: after a partial upload, rerun the upload job using the saved signed artifacts. Rebuilding may produce different bytes and is deliberately rejected. Already-public releases are skipped, and an older retry cannot replace Latest. To change a public release, ship a newer version.

If announcement delivery fails, rerun the announcement job. Dispatch **Announce release on Yapin** with a published tag only to resend a message; a resend posts again. Announcement failures do not roll back publishing. Manual publication also triggers announcements. If release PR checks were not dispatched, run **Checks** on its branch; if a release dispatch failed, run **Release assets** with the existing tag.

Pull requests run formatting, JavaScript wrapper tests, workflow validation, and Rust Clippy/tests on macOS and Windows runners when Rust files change. Website changes also run static export and TypeScript/link checks. They do not build the Windows/macOS release matrix or package installers.

## Connecting your Yapin client

Build the connection in your Yapin client modification (for example, a Yapmod or Yap++ plugin). Subscribe to Yapin’s call and notification events, translate them into the messages below, and send them to Yaplay. The transport is standard WebSocket; there is no required SDK or application-level handshake, and an optional JavaScript wrapper is included. Connect to `ws://127.0.0.1:8080`, or whatever port you set in Yaplay, and send JSON text messages.

Each message has a `type` field. Multiple clients can connect at once. New `voice_state`, `mic_state`, and `deafen_state` messages replace the previous state. Notifications are added to a list (at most five) and disappear after about 8 seconds. Disabled notifications are discarded. Ending a call clears the roster and local mic/deafen state. Yaplay sends control requests only after the integration negotiates control support (see below).

### voice_state

Send the full roster whenever someone joins, leaves, speaks, mutes, or changes name or color.

```json
{
  "type": "voice_state",
  "in_call": true,
  "members": [
    {
      "id": "self",
      "name": "You",
      "color": [103, 80, 164],
      "speaking": false,
      "muted": false,
      "is_self": true
    },
    {
      "id": "ada",
      "name": "Ada",
      "color": [233, 30, 99],
      "speaking": true,
      "muted": false,
      "is_self": false
    }
  ]
}
```

- `in_call`: whether the user is in a call.
- `members`: everyone in the call.
- `id`: a stable id for the member.
- `name`: the display name.
- `color`: red, green, and blue values from 0 to 255.
- `speaking`: draws a border around the avatar.
- `muted`: dims the name and shows a red dot.
- `is_self`: adds "(you)" after the name.

To end the call, send:

```json
{ "type": "voice_state", "in_call": false, "members": [] }
```

### notification

```json
{ "type": "notification", "title": "Hello", "body": "Someone joined the call" }
```

### mic_state and deafen_state

Use these to update the control panel and the mute indicator for the local user.

```json
{ "type": "mic_state", "muted": true }
```

```json
{ "type": "deafen_state", "deafened": true }
```

Each roster member may also include an optional `avatar_url` with a public HTTPS thumbnail URL. Missing or failed images fall back to initials. See the website’s appearance guide for all customization options.

### Client controls

A bidirectional integration opts in after each connection:

```json
{ "type": "hello", "version": 1, "controls": true }
```

The most recently negotiated client owns call state and receives `{"type":"set_mute","muted":true}`, `{"type":"set_deafen","deafened":true}`, or `{"type":"leave_call"}`. Apply these through the host’s voice actions and send the resulting state back. Do not assume a microphone request succeeded: permissions or call roles may prevent it. Controls do not change the overlay state optimistically and are not queued for a future connection. Disconnecting the active control client clears its roster and disables call controls. Legacy state-only connections remain supported.

The JavaScript wrapper opts in when constructed with an `onControl(action)` callback. Apply `set_mute`, `set_deafen`, and `leave_call` through your host, then publish confirmed state with `setRoster`, `setMic`, and `setDeafen`. Handle host errors with `onControlError(error)`.

### Example

Install `ws` with `npm install ws`, then run this:

```js
import WebSocket from "ws";

const socket = new WebSocket("ws://127.0.0.1:8080");

socket.on("open", () => {
  socket.send(JSON.stringify({
    type: "voice_state",
    in_call: true,
    members: [
      {
        id: "self",
        name: "You",
        color: [103, 80, 164],
        speaking: false,
        muted: false,
        is_self: true
      }
    ]
  }));
});

function send(message) {
  if (socket.readyState === WebSocket.OPEN) {
    socket.send(JSON.stringify(message));
  }
}

export function setRoster(members) {
  send({ type: "voice_state", in_call: true, members });
}

export function notify(title, body) {
  send({ type: "notification", title, body });
}

export function setMic(muted) {
  send({ type: "mic_state", muted });
}

export function setDeafen(deafened) {
  send({ type: "deafen_state", deafened });
}
```

If the connection drops, reconnect and send the current roster again.

## Notes

- The server only listens on localhost and has no authentication.
- Send the whole roster instead of trying to update single members.
- Keep member ids stable between updates.
- Invalid messages are logged and ignored. WebSocket frames and messages are limited to 1 MiB.
- State-only clients should send the end-call message before unloading. Disconnecting the active negotiated control client also clears its roster automatically.
- Multiple clients share one overlay; prefer one active state producer.
- Browser connections may be restricted by mixed-content, CSP, or local-network policies. Prefer a desktop host bridge when needed.
- The macOS app is not notarized. Allow it in System Settings if macOS blocks it.

## Artwork

The overlay mascot, peach palette, and Geist Pixel Square wordmark form Yaplay’s visual identity. The [banner](assets/branding/banner.svg), [logo](assets/branding/logo.svg), and [app icon](assets/branding/icon.svg) share the same vector artwork. See the [artwork guide](assets/branding/README.md) for exports and regeneration, and the [Geist font license](assets/branding/Geist-LICENSE.txt).

<p align="center">Yaplay is an independent Yapin community project.</p>
