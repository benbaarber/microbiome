use std::{collections::HashMap, error::Error, net::SocketAddr, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use futures::{stream::SplitSink, SinkExt};
use tokio::sync::Mutex;

pub struct AppState {
    pub websockets: HashMap<SocketAddr, SplitSink<WebSocket, Message>>,
}

impl AppState {
    pub fn new() -> Arc<Mutex<AppState>> {
        let state = AppState {
            websockets: HashMap::new(),
        };

        Arc::new(Mutex::new(state))
    }

    pub fn add_websocket(&mut self, who: SocketAddr, websocket: SplitSink<WebSocket, Message>) {
        self.websockets.insert(who, websocket);
        tracing::debug!("{} websockets connected", self.websockets.len());
    }

    pub fn drop_websocket(&mut self, who: &SocketAddr) {
        self.websockets.remove(who);
        tracing::debug!("{} websockets connected", self.websockets.len());
    }

    pub async fn broadcast_to_websockets(&mut self, msg: Message) -> Result<(), Box<dyn Error>> {
        for sock in self.websockets.values_mut() {
            match sock.feed(msg.clone()).await {
                Err(e) => {
                    eprintln!("failed to feed websocket: {}", e);
                    continue;
                }
                _ => (),
            }

            match sock.flush().await {
                Err(e) => {
                    eprintln!("failed to flush websocket: {}", e);
                    continue;
                }
                _ => (),
            }
        }

        Ok(())
    }
}
