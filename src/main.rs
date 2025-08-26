//! Main entry point for the RGA CRDT web server.
//!
//! This binary provides an HTTP API for interacting with the RGA CRDT
//! using the Axum web framework.

use std::net::SocketAddr;
use std::sync::Arc;
use tokio;
use tokio::sync::RwLock;
use tracing::{Level, info};
use tracing_subscriber;

mod crdt;
mod server;

use crdt::RGA;
use server::{AppState, create_router};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting RGA CRDT Axum server...");

    // Create shared RGA state (replica ID = 1 for now)
    let rga = RGA::new(1);
    let state: AppState = Arc::new(RwLock::new(rga));

    // Build our application with routes from the server module
    let app = create_router().with_state(state);

    // Define the address to bind to
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    info!("Server listening on http://{}", addr);
    info!("Available endpoints:");
    info!("  GET  /health  - Health check");
    info!("  GET  /ws      - WebSocket for collaborative editing");
    info!("");
    info!("Try these commands:");
    info!("  curl http://localhost:3000/health");
    info!("  # Connect to WebSocket: ws://localhost:3000/ws");
    info!("  # Open frontend/index.html to test collaborative editing");

    // Run the server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
