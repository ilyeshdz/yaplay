use std::sync::{Mutex, mpsc};

use std::thread;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{
    accept_async_with_config,
    tungstenite::{Message, protocol::WebSocketConfig},
};

use crate::state::{ServerMessage, SharedState};

struct Server {
    port: u16,
    shutdown: tokio::sync::oneshot::Sender<()>,
    thread: thread::JoinHandle<()>,
}

static SERVER: Mutex<Option<Server>> = Mutex::new(None);

pub fn start_server(port: u16, state: SharedState) -> Result<(), String> {
    let mut server = SERVER.lock().unwrap();
    if server.as_ref().is_some_and(|server| server.port == port) {
        return Ok(());
    }
    let (shutdown, stopped) = tokio::sync::oneshot::channel();
    let (sender, receiver) = mpsc::channel();

    let thread = thread::Builder::new()
        .name(format!("ws-server-{port}"))
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(err) => {
                    let _ = sender.send(Err(format!("Failed to create the Tokio runtime: {err}")));
                    return;
                }
            };
            runtime.block_on(async move {
                tokio::select! {
                    _ = server_loop(port, state, sender) => {},
                    _ = stopped => {},
                }
            });
        })
        .map_err(|err| format!("Failed to spawn the WebSocket server thread: {err}"))?;

    match receiver.recv_timeout(Duration::from_secs(3)) {
        Ok(Ok(())) => {
            if let Some(old) = server.replace(Server {
                port,
                shutdown,
                thread,
            }) {
                let _ = old.shutdown.send(());
                let _ = old.thread.join();
            }
            Ok(())
        }
        Ok(Err(error)) => Err(error),
        Err(_) => Err("Timed out while starting the WebSocket server".to_string()),
    }
}

async fn server_loop(port: u16, state: SharedState, sender: mpsc::Sender<Result<(), String>>) {
    let addr = format!("127.0.0.1:{}", port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(err) => {
            let _ = sender.send(Err(format!("Failed to bind to {}: {}", addr, err)));
            return;
        }
    };

    let _ = sender.send(Ok(()));
    println!("WebSocket server listening on {}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                println!("WebSocket client connected: {}", peer);
                let state = state.clone();
                tokio::spawn(async move {
                    handle_connection(stream, state).await;
                });
            }
            Err(err) => {
                eprintln!("Failed to accept WebSocket connection: {}", err);
            }
        }
    }
}

// Drop also runs when a port change shuts down the old runtime.
struct ControlConnection {
    state: SharedState,
    sender: tokio::sync::mpsc::Sender<crate::state::ControlAction>,
}

impl Drop for ControlConnection {
    fn drop(&mut self) {
        if let Ok(mut state) = self.state.write()
            && state
                .control_client
                .as_ref()
                .is_some_and(|sender| sender.same_channel(&self.sender))
        {
            state.control_client = None;
            state.apply(ServerMessage::VoiceState {
                in_call: false,
                members: vec![],
            });
        }
    }
}

async fn handle_connection(stream: tokio::net::TcpStream, state: SharedState) {
    let config = WebSocketConfig::default()
        .max_message_size(Some(1024 * 1024))
        .max_frame_size(Some(1024 * 1024));
    let handshake = tokio::time::timeout(
        Duration::from_secs(5),
        accept_async_with_config(stream, Some(config)),
    )
    .await;
    let mut stream = match handshake {
        Ok(Ok(stream)) => stream,
        Ok(Err(err)) => {
            eprintln!("WebSocket handshake failed: {err}");
            return;
        }
        Err(_) => {
            eprintln!("WebSocket handshake timed out");
            return;
        }
    };
    if let Ok(mut state) = state.write() {
        state.push_notification("Client connected", "An application connected to Yaplay");
    }

    let (control_sender, mut controls) = tokio::sync::mpsc::channel(16);
    let _connection = ControlConnection {
        state: state.clone(),
        sender: control_sender.clone(),
    };
    loop {
        tokio::select! {
            message = stream.next() => match message {
                Some(Ok(Message::Text(text))) => match serde_json::from_str::<ServerMessage>(&text) {
                    Ok(ServerMessage::Hello { version: 1, controls: true }) => {
                        if let Ok(mut state) = state.write() {
                            // Only the most recently negotiated integration receives controls.
                            if !state.control_client.as_ref().is_some_and(|sender| sender.same_channel(&control_sender)) {
                                state.apply(ServerMessage::VoiceState { in_call: false, members: vec![] });
                            }
                            state.control_client = Some(control_sender.clone());
                        }
                    }
                    Ok(ServerMessage::Hello { .. }) => {
                        if let Ok(mut state) = state.write()
                            && state.control_client.as_ref().is_some_and(|sender| sender.same_channel(&control_sender)) {
                            state.control_client = None;
                            state.apply(ServerMessage::VoiceState { in_call: false, members: vec![] });
                        }
                    }
                    Ok(message) => {
                        if let Ok(mut state) = state.write() {
                            let owns_state = state.control_client.as_ref().is_none_or(|sender| sender.same_channel(&control_sender));
                            if owns_state || matches!(message, ServerMessage::Notification { .. }) { state.apply(message); }
                        }
                    }
                    Err(err) => eprintln!("Ignoring malformed WebSocket message: {err}"),
                },
                Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                Some(Ok(Message::Ping(_))) if stream.flush().await.is_err() => break,
                _ => {},
            },
            Some(action) = controls.recv() => {
                let owns_controls = state.read().is_ok_and(|state| state.in_call && state.control_client.as_ref()
                    .is_some_and(|sender| sender.same_channel(&control_sender)));
                if !owns_controls { continue; }
                let text = serde_json::to_string(&action).expect("control actions serialize");
                if !matches!(tokio::time::timeout(Duration::from_secs(2), stream.send(Message::Text(text.into()))).await, Ok(Ok(()))) { break; }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use std::sync::{Arc, RwLock};

    #[test]
    fn server_reuses_port_and_keeps_existing_listener_on_failed_change() {
        let state = Arc::new(RwLock::new(AppState::default()));
        let unused = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = unused.local_addr().unwrap().port();
        drop(unused);
        start_server(port, state.clone()).unwrap();
        start_server(port, state.clone()).unwrap();
        let occupied = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        assert!(start_server(occupied.local_addr().unwrap().port(), state.clone()).is_err());
        assert!(std::net::TcpStream::connect(("127.0.0.1", port)).is_ok());
        let next = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let next_port = next.local_addr().unwrap().port();
        drop(next);
        start_server(next_port, state.clone()).unwrap();
        assert!(std::net::TcpStream::connect(("127.0.0.1", port)).is_err());
        // Exercise the real listener's address-reuse settings. A plain std
        // listener can fail on macOS while the earlier connection is in TIME_WAIT.
        start_server(port, state).unwrap();
        let server = SERVER.lock().unwrap().take().unwrap();
        let _ = server.shutdown.send(());
        server.thread.join().unwrap();
    }

    #[tokio::test]
    async fn connection_ignores_invalid_messages_and_applies_valid_state() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let state = Arc::new(RwLock::new(AppState::default()));
        let shared = state.clone();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            handle_connection(stream, shared).await;
        });
        let (mut client, _) = tokio_tungstenite::connect_async(format!("ws://{address}"))
            .await
            .unwrap();
        client
            .send(Message::Text("invalid json".into()))
            .await
            .unwrap();
        client
            .send(Message::Text(r#"{"type":"mic_state","muted":true}"#.into()))
            .await
            .unwrap();
        client.close(None).await.unwrap();
        tokio::time::timeout(Duration::from_secs(2), server)
            .await
            .unwrap()
            .unwrap();
        let state = state.read().unwrap();
        assert!(state.mic_muted);
        assert_eq!(state.notifications.len(), 1);
    }

    #[tokio::test]
    async fn negotiated_controls_round_trip_without_optimistic_state() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let state = Arc::new(RwLock::new(AppState::default()));
        let shared = state.clone();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            handle_connection(stream, shared).await;
        });
        let (mut client, _) = tokio_tungstenite::connect_async(format!("ws://{address}"))
            .await
            .unwrap();
        for text in [
            r#"{"type":"hello","version":1,"controls":true}"#,
            r#"{"type":"voice_state","in_call":true,"members":[]}"#,
        ] {
            client.send(Message::Text(text.into())).await.unwrap();
        }
        client.send(Message::Ping(vec![1].into())).await.unwrap();
        assert!(matches!(client.next().await, Some(Ok(Message::Pong(_)))));
        {
            let state = state.read().unwrap();
            assert!(state.snapshot().controls_available);
            state.request_control(crate::state::ControlAction::SetMute { muted: true });
            assert!(!state.mic_muted);
        }
        let message = tokio::time::timeout(Duration::from_secs(2), client.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(
            message.into_text().unwrap(),
            r#"{"type":"set_mute","muted":true}"#
        );
        client
            .send(Message::Text(r#"{"type":"mic_state","muted":true}"#.into()))
            .await
            .unwrap();
        client.send(Message::Ping(vec![2].into())).await.unwrap();
        assert!(matches!(client.next().await, Some(Ok(Message::Pong(_)))));
        assert!(state.read().unwrap().mic_muted);
        client.close(None).await.unwrap();
        tokio::time::timeout(Duration::from_secs(2), server)
            .await
            .unwrap()
            .unwrap();
        let state = state.read().unwrap();
        assert!(!state.snapshot().controls_available);
        assert!(!state.in_call);
    }

    type Client = tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >;

    async fn connect_test(state: SharedState) -> (Client, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            handle_connection(stream, state).await;
        });
        let (client, _) = tokio_tungstenite::connect_async(format!("ws://{address}"))
            .await
            .unwrap();
        (client, server)
    }

    async fn publish(client: &mut Client, messages: &[&str]) {
        for message in messages {
            client.send(Message::Text((*message).into())).await.unwrap();
        }
        client.send(Message::Ping(vec![0].into())).await.unwrap();
        assert!(matches!(
            tokio::time::timeout(Duration::from_secs(2), client.next())
                .await
                .unwrap(),
            Some(Ok(Message::Pong(_)))
        ));
    }

    #[tokio::test]
    async fn active_client_owns_state_and_controls_and_cancellation_cleans_up() {
        let state = Arc::new(RwLock::new(AppState::default()));
        let (mut first, first_task) = connect_test(state.clone()).await;
        publish(
            &mut first,
            &[
                r#"{"type":"hello","version":1,"controls":true}"#,
                r#"{"type":"voice_state","in_call":true,"members":[]}"#,
            ],
        )
        .await;
        let (mut second, second_task) = connect_test(state.clone()).await;
        publish(
            &mut second,
            &[
                r#"{"type":"hello","version":1,"controls":true}"#,
                r#"{"type":"voice_state","in_call":true,"members":[]}"#,
            ],
        )
        .await;
        publish(
            &mut first,
            &[
                r#"{"type":"mic_state","muted":true}"#,
                r#"{"type":"voice_state","in_call":false,"members":[]}"#,
            ],
        )
        .await;
        assert!(!state.read().unwrap().mic_muted);
        assert!(state.read().unwrap().in_call);
        for action in [
            crate::state::ControlAction::SetDeafen { deafened: true },
            crate::state::ControlAction::LeaveCall {},
        ] {
            state.read().unwrap().request_control(action.clone());
            let message = tokio::time::timeout(Duration::from_secs(2), second.next())
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            assert_eq!(
                serde_json::from_str::<crate::state::ControlAction>(&message.into_text().unwrap())
                    .unwrap(),
                action
            );
        }
        first_task.abort();
        let _ = first_task.await;
        assert!(state.read().unwrap().snapshot().controls_available);
        second_task.abort();
        let _ = second_task.await;
        assert!(!state.read().unwrap().snapshot().controls_available);
        assert!(!state.read().unwrap().in_call);
    }

    #[tokio::test]
    async fn unsupported_hello_does_not_enable_controls_and_can_revoke_them() {
        let state = Arc::new(RwLock::new(AppState::default()));
        let (mut client, task) = connect_test(state.clone()).await;
        publish(
            &mut client,
            &[r#"{"type":"hello","version":2,"controls":true}"#],
        )
        .await;
        assert!(!state.read().unwrap().snapshot().controls_available);
        publish(
            &mut client,
            &[r#"{"type":"hello","version":1,"controls":true}"#],
        )
        .await;
        assert!(state.read().unwrap().snapshot().controls_available);
        publish(
            &mut client,
            &[r#"{"type":"hello","version":1,"controls":false}"#],
        )
        .await;
        assert!(!state.read().unwrap().snapshot().controls_available);
        task.abort();
        let _ = task.await;
    }

    #[tokio::test]
    async fn incomplete_handshake_does_not_notify() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let state = Arc::new(RwLock::new(AppState::default()));
        let client = tokio::net::TcpStream::connect(address).await.unwrap();
        let (stream, _) = listener.accept().await.unwrap();
        drop(client);
        handle_connection(stream, state.clone()).await;
        assert!(state.read().unwrap().notifications.is_empty());
    }
}
