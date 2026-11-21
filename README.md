# Yaplay

Yaplay is a desktop overlay built for Yapin. It shows your Yapin call roster, speaking activity, mute indicators, and notifications on top of your other windows. It is built in Rust with Freya.

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

The build copies the wrapper into the static output for download. Run `npm run typecheck` inside `site/`, `node --test examples/yaplay-client.test.mjs` at the repository root, and `cargo test --locked` for Rust behavior.

## What it does

Press Start in the settings window and Yaplay starts a local WebSocket server, closes the settings window, and opens a transparent overlay. The roster is in the top left and notifications are in the top right. The overlay is click through, so it does not block the windows below it.

Press `Ctrl+Shift+Y` on Windows or `Cmd+Shift+Y` on macOS to show the control panel. It has mic, deafen, and exit buttons. These buttons only change what Yaplay displays. They do not send anything back to the connected client.

Yaplay listens on `127.0.0.1:<port>`. The default port is `8080`.

## Build

You need Rust 1.85 or newer. On Windows you also need the C++ Build Tools with the Windows SDK. On macOS you need the Xcode Command Line Tools.

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
- macOS Intel: `Yaplay-x86_64-apple-darwin.dmg` and `Yaplay-x86_64-apple-darwin.app.zip`

Release Please opens a release PR on `master` with the next version and a [changelog](CHANGELOG.md). Merging that PR creates the tag and GitHub Release, then builds and attaches these files. Desktop release packaging never runs on ordinary pushes or pull requests. Open the DMG and drag Yaplay to Applications, or unzip the app bundle and run it directly.

## Release lifecycle and checks

Use Conventional Commit PR titles (`feat:`, `fix:`, `docs:`, `ci:`, etc.) and squash-merge feature PRs. Release Please accumulates changes since the previous release, updates `Cargo.toml`, `Cargo.lock`, and `CHANGELOG.md` in one release PR, and publishes after that PR is merged. The initial release is `1.0.0`.

Pull requests run formatting, JavaScript wrapper tests, workflow validation, and Rust Clippy/tests on one macOS runner when Rust files change. Website changes also run static export and TypeScript/link checks. They do not build the Windows/macOS release matrix or package installers.

See [RELEASING.md](RELEASING.md) for repository setup, versioning rules, release review, and retrying failed packaging.

## Connecting your Yapin client

Build the connection in your Yapin client modification (for example, a Yapmod or Yap++ plugin). Subscribe to Yapin’s call and notification events, translate them into the messages below, and send them to Yaplay. The transport is standard WebSocket; there is no required SDK or application-level handshake, and an optional JavaScript wrapper is included. Connect to `ws://127.0.0.1:8080`, or whatever port you set in Yaplay, and send JSON text messages.

Each message has a `type` field. Multiple clients can connect at once. New `voice_state`, `mic_state`, and `deafen_state` messages replace the previous state. Notifications are added to a list (at most five) and disappear after about 8 seconds. Disabled notifications are discarded. Ending a call clears the roster and local mic/deafen state. Yaplay does not send replies.

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
- Invalid messages are logged and ignored. WebSocket frames and messages are limited to 64 KiB.
- Disconnecting does not clear shared state automatically. Send the end-call message before unloading your integration.
- Multiple clients share one overlay; prefer one active state producer.
- Browser connections may be restricted by mixed-content, CSP, or local-network policies. Prefer a desktop host bridge when needed.
- The macOS app is not notarized. Allow it in System Settings if macOS blocks it.
