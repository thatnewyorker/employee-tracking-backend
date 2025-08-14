// src/main.rs

use std::net::SocketAddr;
use std::process::exit;

use axum::{Router, routing::get};
use dotenvy::dotenv;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

mod config;
mod health;

use config::Config;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_env("RUST_LOG"))
        .init();

    // Load config
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to load config: {}", e);
            exit(1);
        }
    };

    #[cfg(feature = "no-auth")]
    info!("NO-AUTH MODE ENABLED");

    info!("Server starting on {}", config.bind_address);

    // Build the Axum router
    let app = Router::new()
        .route("/health", get(health::health))
        .layer(TraceLayer::new_for_http());

    // Run the server
    let listener = tokio::net::TcpListener::bind(config.bind_address)
        .await
        .unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Signal received, shutting down server gracefully");
}
