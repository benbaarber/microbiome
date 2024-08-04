use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, State,
    },
    response::IntoResponse,
};
use axum_extra::{headers, TypedHeader};
use futures::{stream::StreamExt, SinkExt};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

use crate::state::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<Mutex<AppState>>>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };

    println!("`{user_agent}` at {addr} connected.");

    ws.on_upgrade(move |socket| handle_socket(socket, addr, Arc::clone(&state)))
}

pub async fn handle_socket(mut socket: WebSocket, who: SocketAddr, state: Arc<Mutex<AppState>>) {
    if socket
        .send(Message::Ping("ping".as_bytes().to_vec()))
        .await
        .is_ok()
    {
        tracing::debug!("{who} websocket connected");
    } else {
        tracing::error!("{who} websocket failed to connect");
        return;
    }

    socket.flush().await.unwrap();

    let (sender, mut receiver) = socket.split();

    let mut s = (*state).lock().await;
    s.add_websocket(who.clone(), sender);
    drop(s);

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            let msg_str = match msg.to_text() {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("failed to parse websocket message: {}", e);
                    break;
                }
            };

            tracing::debug!("received from socket: {}", msg_str);
        }
    });

    let (_,) = tokio::join!(recv_task);

    tracing::debug!("websocket disconnected for {}", who);

    let mut s = (*state).lock().await;
    s.drop_websocket(&who);
}
