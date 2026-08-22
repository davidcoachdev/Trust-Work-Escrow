use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use tokio::sync::broadcast;

static WS_TX: once_cell::sync::OnceCell<broadcast::Sender<String>> = once_cell::sync::OnceCell::new();

fn tx() -> &'static broadcast::Sender<String> {
    WS_TX.get_or_init(|| broadcast::channel(100).0)
}

pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let mut rx = tx().subscribe();
    loop {
        tokio::select! {
            msg = rx.recv() => {
                if let Ok(m) = msg {
                    if socket.send(Message::Text(m.into())).await.is_err() { break; }
                }
            }
            _ = socket.recv() => { break; }
        }
    }
}

pub fn broadcast(msg: String) {
    let _ = tx().send(msg);
}
