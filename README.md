# Yaplay

Yaplay is a small desktop overlay app built with Rust and Freya. It shows a voice call roster and notifications on top of your other windows, plus a little control panel you can toggle with a keyboard shortcut.

## What it does

When you press Start in the settings window, Yaplay starts a WebSocket server, closes the settings window, and opens a transparent overlay. The overlay has a voice roster in the top left corner and notifications in the top right. It is click through, so it never blocks the apps underneath. Press Ctrl+Shift+Y on Windows or Linux, or Cmd+Shift+Y on macOS, to show a small control panel at the bottom center with mic, deafen, and exit buttons.

Yaplay listens for WebSocket connections on 127.0.0.1 with the port you pick in the settings, 8080 by default.

## Building and running

You need Rust. Then run:

```
cargo run
```

This opens the settings window. Pick your display, choose a port, and press Start.

## Making a bridge plugin

A bridge plugin is just any program that connects to Yaplay over WebSocket and sends it messages. You can write it in whatever language you like, Node, Python, Rust, anything that has a WebSocket client. There is no special SDK and nothing to install into Yaplay. The plugin and Yaplay talk to each other over the socket.

First connect to the WebSocket at ws://127.0.0.1:8080, or whichever port you configured. Once you are connected, you send plain JSON messages. Each message has a type field that tells Yaplay what to do with it.

To show the voice roster, send a voice state message. The members array holds everyone in the call. Give each member an id, a name, a color as three numbers for red, green, and blue, and flags for whether they are speaking, muted, or you. Set in_call to true when someone is in a call, and false when the call ends. Here is an example:

```
{
  "type": "voice_state",
  "in_call": true,
  "members": [
    { "id": "self", "name": "You", "color": [103, 80, 164], "speaking": false, "muted": false, "is_self": true },
    { "id": "ada", "name": "Ada", "color": [233, 30, 99], "speaking": true, "muted": false, "is_self": false }
  ]
}
```

To pop up a notification in the top right corner, send a notification message with a title and a body:

```
{ "type": "notification", "title": "Hello", "body": "Someone joined the call" }
```

You can also push the mic and deafen state so the control panel reflects what is happening on your side. Send mic_state with muted set to true or false, and deafen_state with deafened set to true or false.

That is really all there is to it. Connect, send JSON, and the overlay updates. Send a voice state whenever the call changes, send notifications when something happens, and you have a working bridge.
