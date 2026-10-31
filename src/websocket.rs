use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use futures_util::StreamExt;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::state::{ServerMessage, SharedState};

pub fn start_server(port: u16, state: SharedState) -> Result<(), String> {
    let (sender, receiver) = mpsc::channel();

    thread::Builder::new()
        .name(format!("ws-server-{port}"))
        .spawn(move || {
            let runtime =
                match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                    Ok(runtime) => runtime,
                    Err(err) => {
                        let _ =
                            sender.send(Err(format!("Failed to create the Tokio runtime: {err}")));
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
                if let Ok(mut state) = state.write() {
                    state.push_notification("Client connected", format!("{} joined", peer));
                }
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
    let mut stream = match accept_async(stream).await {
        Ok(stream) => stream,
        Err(err) => {
            eprintln!("WebSocket handshake failed: {}", err);
            return;
        }
    };

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
