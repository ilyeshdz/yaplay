use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use futures_util::StreamExt;
use tokio::net::TcpListener;
use tokio_tungstenite::{
    accept_async_with_config,
    tungstenite::{Message, protocol::WebSocketConfig},
};

use crate::state::{ServerMessage, SharedState};

pub fn start_server(port: u16, state: SharedState) -> Result<(), String> {
    let (sender, receiver) = mpsc::channel();

    thread::Builder::new()
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
            runtime.block_on(server_loop(port, state, sender));
        })
        .map_err(|err| format!("Failed to spawn the WebSocket server thread: {err}"))?;

    match receiver.recv_timeout(Duration::from_secs(3)) {
        Ok(result) => result,
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

async fn handle_connection(stream: tokio::net::TcpStream, state: SharedState) {
    let config = WebSocketConfig::default()
        .max_message_size(Some(64 * 1024))
        .max_frame_size(Some(64 * 1024));
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

    while let Some(message) = stream.next().await {
        match message {
            Ok(Message::Text(text)) => match serde_json::from_str::<ServerMessage>(&text) {
                Ok(message) => {
                    if let Ok(mut state) = state.write() {
                        state.apply(message);
                    }
                }
                Err(err) => eprintln!("Ignoring malformed WebSocket message: {}", err),
            },
            Ok(Message::Close(_)) | Err(_) => break,
            Ok(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use futures_util::SinkExt;
    use std::sync::{Arc, RwLock};

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
