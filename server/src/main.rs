use std::{net::SocketAddr, process};

use app::make_app;
use config::Config;

mod app;
mod bind;
mod config;
mod state;
mod ws;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let config = Config::read_env().unwrap();
    let addr = config.host.clone() + ":" + config.port.as_str();

    let (app, mb_thread) = match make_app(&config) {
        Ok(a) => a,
        Err(e) => {
            tracing::error!("failed to create router: {}", e);
            process::exit(1);
        }
    };

    let app_thread = tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => {
                tracing::debug!("listening on {}", l.local_addr().unwrap());
                l
            }
            Err(e) => {
                tracing::error!("failed to bind at {}: {}", addr, e);
                process::exit(1)
            }
        };

        match axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        {
            Err(e) => tracing::error!("app failed: {}", e),
            _ => (),
        }
    });

    match tokio::join!(mb_thread, app_thread) {
        (Err(e), _) => tracing::error!("microbiome thread died: {}", e),
        (_, Err(e)) => tracing::error!("app thread died: {}", e),
        _ => (),
    }
}
