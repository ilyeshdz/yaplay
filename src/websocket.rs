use std::sync::{Mutex, mpsc};

use std::thread;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{
    accept_async_with_config,
    tungstenite::{Message, protocol::WebSocketConfig},
};

use crate::state::{ActiveIntegration, Capability, Integration, ServerMessage, SharedState};

struct Server {
    port: u16,
    shutdown: tokio::sync::oneshot::Sender<()>,
    thread: thread::JoinHandle<()>,
}

static SERVER: Mutex<Option<Server>> = Mutex::new(None);

pub fn listening_port() -> Option<u16> {
    SERVER.lock().unwrap().as_ref().map(|server| server.port)
}

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
                if state
                    .read()
                    .is_ok_and(|state| !state.config.accept_connections)
                {
                    println!("Rejected WebSocket client: {}", peer);
                    drop(stream);
                    continue;
                }
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

impl ControlConnection {
    fn negotiate(&self, integration: Option<Integration>) {
        let mut state = self.state.write().unwrap();
        let current = state
            .active
            .as_ref()
            .is_some_and(|client| client.sender.same_channel(&self.sender));
        if current != integration.is_some() {
            state.apply(ServerMessage::VoiceState {
                in_call: false,
                members: vec![],
            });
        }
        if current || integration.is_some() {
            state.active = integration.map(|info| ActiveIntegration {
                info,
                sender: self.sender.clone(),
            });
            state.legacy_source = None;
        }
    }
}

impl Drop for ControlConnection {
    fn drop(&mut self) {
        self.negotiate(None);
        let mut state = self.state.write().unwrap();
        if state
            .legacy_source
            .as_ref()
            .is_some_and(|s| s.same_channel(&self.sender))
        {
            state.legacy_source = None;
        }
    }
}

fn valid_label(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 128 && !value.chars().any(char::is_control)
}

async fn handle_connection(stream: tokio::net::TcpStream, state: SharedState) {
    if state
        .read()
        .is_ok_and(|state| !state.config.accept_connections)
    {
        return;
    }
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
    // Re-check under the same lock that hands out the disconnect signal, so a
    // disable that raced the handshake cannot slip through.
    let Some(mut disconnect) = state
        .write()
        .ok()
        .and_then(|mut state| state.subscribe_disconnects())
    else {
        return;
    };
    if let Ok(mut state) = state.write()
        && state.config.connection_notifications
    {
        state.push_notification("Client connected", "An application connected to Yaplay");
    }

    let (control_sender, mut controls) = tokio::sync::mpsc::channel(16);
    let connection = ControlConnection {
        state: state.clone(),
        sender: control_sender.clone(),
    };
    loop {
        tokio::select! {
            _ = disconnect.recv() => {
                let _ = stream.close(None).await;
                break;
            }
            message = stream.next() => match message {
                Some(Ok(Message::Text(text))) => match serde_json::from_str::<ServerMessage>(&text) {
                    Ok(ServerMessage::Hello { version: 2, name: Some(name), client_version, actions: Some(actions), .. })
                        if valid_label(&name) && client_version.as_deref().is_none_or(valid_label) => {
                        let integration = Integration {
                            name,
                            version: client_version,
                            actions: actions.iter().filter_map(|name| Capability::from_wire(name)).collect(),
                        };
                        let reply = serde_json::json!({"type": "hello_ack", "version": 2, "actions": integration.actions});
                        connection.negotiate(Some(integration));
                        if !matches!(tokio::time::timeout(Duration::from_secs(2), stream.send(Message::Text(reply.to_string().into()))).await, Ok(Ok(()))) { break; }
                    }
                    Ok(ServerMessage::Hello { version: 1, controls: Some(true), .. }) => {
                        connection.negotiate(Some(Integration {
                            name: "Unnamed integration".into(),
                            version: None,
                            actions: Capability::ALL.into_iter().collect(),
                        }));
                    }
                    Ok(ServerMessage::Hello { .. }) => connection.negotiate(None),
                    Ok(message) => {
                        if let Ok(mut state) = state.write() {
                            let owns_state = state.active.as_ref().is_none_or(|client| client.sender.same_channel(&control_sender));
                            if state.active.is_none() && !matches!(message, ServerMessage::Notification { .. }) {
                                state.legacy_source = Some(control_sender.clone());
                            }
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
                let owns_controls = state.read().is_ok_and(|state| state.permits(&action) && state.active.as_ref()
                    .is_some_and(|client| client.sender.same_channel(&control_sender)));
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
    async fn connection_notification_can_be_disabled() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let state = Arc::new(RwLock::new(AppState::default()));
        state.write().unwrap().config.connection_notifications = false;
        let shared = state.clone();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            handle_connection(stream, shared).await;
        });
        let (mut client, _) = tokio_tungstenite::connect_async(format!("ws://{address}"))
            .await
            .unwrap();
        client.close(None).await.unwrap();
        tokio::time::timeout(Duration::from_secs(2), server)
            .await
            .unwrap()
            .unwrap();
        assert!(state.read().unwrap().notifications.is_empty());
    }

    #[tokio::test]
    async fn disabled_connections_are_rejected() {
        use tokio::io::AsyncReadExt;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let state = Arc::new(RwLock::new(AppState::default()));
        state.write().unwrap().config.accept_connections = false;
        let (client, _) = tokio::join!(tokio::net::TcpStream::connect(address), async {
            let (stream, _) = listener.accept().await.unwrap();
            handle_connection(stream, state.clone()).await;
        });
        let mut client = client.unwrap();
        let mut buffer = [0u8; 1];
        assert_eq!(client.read(&mut buffer).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn disabling_connections_closes_the_active_client() {
        let state = Arc::new(RwLock::new(AppState::default()));
        let (mut client, server) = connect_test(state.clone()).await;
        publish(
            &mut client,
            &[
                r#"{"type":"hello","version":1,"controls":true}"#,
                r#"{"type":"voice_state","in_call":true,"members":[]}"#,
            ],
        )
        .await;
        assert!(state.read().unwrap().in_call);

        {
            let mut state = state.write().unwrap();
            state.config.accept_connections = false;
            state.disconnect_clients();
        }

        let message = tokio::time::timeout(Duration::from_secs(2), client.next())
            .await
            .unwrap();
        assert!(matches!(
            message,
            Some(Ok(Message::Close(_))) | None | Some(Err(_))
        ));
        tokio::time::timeout(Duration::from_secs(2), server)
            .await
            .unwrap()
            .unwrap();
        let state = state.read().unwrap();
        assert!(state.integration().is_none());
        assert!(!state.in_call);
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
            assert!(state.integration().is_some());
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
        assert!(state.integration().is_none());
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
        assert!(state.read().unwrap().integration().is_some());
        second_task.abort();
        let _ = second_task.await;
        assert!(state.read().unwrap().integration().is_none());
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
        assert!(state.read().unwrap().integration().is_none());
        publish(
            &mut client,
            &[r#"{"type":"hello","version":1,"controls":true}"#],
        )
        .await;
        assert!(state.read().unwrap().integration().is_some());
        publish(
            &mut client,
            &[r#"{"type":"hello","version":1,"controls":false}"#],
        )
        .await;
        assert!(state.read().unwrap().integration().is_none());
        task.abort();
        let _ = task.await;
    }

    #[tokio::test]
    async fn v2_negotiates_actions_and_can_renegotiate_as_roster_only() {
        use crate::state::{AppState, ControlAction};
        let state = Arc::new(RwLock::new(AppState::default()));
        let (mut client, task) = connect_test(state.clone()).await;
        for (requested, accepted) in [
            (vec!["set_mute", "future", "set_mute"], vec!["set_mute"]),
            (vec![], vec![]),
        ] {
            let hello = serde_json::json!({"type":"hello","version":2,"name":"Demo","client_version":"1.2","actions": requested});
            client
                .send(Message::Text(hello.to_string().into()))
                .await
                .unwrap();
            let reply = tokio::time::timeout(Duration::from_secs(2), client.next())
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&reply.into_text().unwrap()).unwrap(),
                serde_json::json!({"type":"hello_ack","version":2,"actions":accepted})
            );
            if accepted.is_empty() {
                assert!(state.read().unwrap().in_call);
            } else {
                publish(
                    &mut client,
                    &[r#"{"type":"voice_state","in_call":true,"members":[]}"#],
                )
                .await;
            }
            let locked = state.read().unwrap();
            let info = locked.integration().unwrap();
            assert_eq!(info.name, "Demo");
            assert_eq!(info.version.as_deref(), Some("1.2"));
            assert!(!locked.permits(&ControlAction::LeaveCall {}));
            assert_eq!(
                locked.permits(&ControlAction::SetMute { muted: true }),
                !accepted.is_empty()
            );
        }
        let (mut legacy, other) = connect_test(state.clone()).await;
        publish(
            &mut legacy,
            &[r#"{"type":"voice_state","in_call":false,"members":[]}"#],
        )
        .await;
        assert!(state.read().unwrap().in_call);
        task.abort();
        let _ = task.await;
        assert!(state.read().unwrap().integration().is_none());
        assert!(!state.read().unwrap().in_call);
        other.abort();
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
