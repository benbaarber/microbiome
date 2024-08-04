use std::{error::Error, path::PathBuf, str::FromStr, sync::Arc};

use axum::routing::get;
use tokio::task::JoinHandle;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::{bind::mb_bind, config::Config, state::AppState, ws::ws_handler};

pub fn make_app(config: &Config) -> Result<(axum::Router, JoinHandle<()>), Box<dyn Error>> {
    let static_dir = config.static_path.clone();
    let static_path = PathBuf::from_str(&static_dir)?;
    let serve_static = ServeDir::new(static_path.clone())
        .not_found_service(ServeFile::new(static_path.join("index.html")));

    tracing::debug!("serving static {}", static_dir);

    let state = AppState::new();

    let config_clone = config.clone();
    let state_clone = Arc::clone(&state);
    let mb_handler = tokio::task::spawn_blocking(move || mb_bind(config_clone, state_clone));

    let app = axum::Router::new()
        .route("/ping", get(|| async { "pong" }))
        .route("/ws", get(ws_handler))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::very_permissive())
        .fallback_service(serve_static);

    Ok((app, mb_handler))
}
