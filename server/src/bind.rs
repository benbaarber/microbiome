use axum::extract::ws::Message;
use serde_json::json;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::Config;
use crate::state::AppState;

fn load_socks(config: &Config) -> Result<zmq::Socket, Box<dyn Error>> {
    let context = zmq::Context::new();
    let sub_sock = context.socket(zmq::SUB)?;
    sub_sock.set_subscribe("mb_state".as_bytes())?;
    sub_sock.connect(&config.sub_at)?;

    tracing::debug!("microbiome sock listening at {}", config.sub_at);

    Ok(sub_sock)
}

pub fn build_websocket_msg(msgb: Vec<Vec<u8>>) -> Result<Option<Message>, Box<dyn Error>> {
    let cmd_str = String::from_utf8(msgb[1].to_vec())?;

    tracing::debug!(cmd_str);

    let json_msg = match cmd_str.as_str() {
        "state" => {
            let value = serde_json::from_slice::<serde_json::Value>(&msgb[2])?;

            json!({
                "event": cmd_str,
                "data": value,
            })
        }
        _ => return Ok(None),
    };

    let msgs = serde_json::to_string(&json_msg)?;

    let msg = Message::from(msgs);
    Ok(Some(msg))
}

pub fn mb_bind(config: Config, state: Arc<Mutex<AppState>>) {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => return tracing::error!("failed to create tokio runtime in mb_bind: {}", e),
    };

    let sub_sock = match load_socks(&config) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("failed to load microbiome sub sock: {}", e);
            return;
        }
    };

    let mut socks = [sub_sock.as_poll_item(zmq::POLLIN)];

    loop {
        // println!("polling");
        match zmq::poll(&mut socks, 10) {
            Err(e) => {
                tracing::error!("failed to poll microbiome: {}", e);
                continue;
            }
            _ => (),
        }

        while socks[0].is_readable() {
            let msgb = match sub_sock.recv_multipart(zmq::DONTWAIT) {
                Ok(msgb) => msgb,
                Err(_) => break,
            };

            if msgb.len() < 3 {
                continue;
            }

            match build_websocket_msg(msgb) {
                Ok(Some(msg)) => {
                    let mut s = state.blocking_lock();
                    match rt.block_on(s.broadcast_to_websockets(msg)) {
                        Err(e) => {
                            tracing::error!("failed to broadcast message to websockets: {}", e)
                        }
                        _ => (),
                    }
                }
                Ok(None) => {
                    tracing::error!("build null message");
                }
                Err(e) => {
                    tracing::error!("failed to build websocket message: {}", e);
                }
            }
        }
    }
}
