//! Main entry point for the RGA CRDT web server.
//!
//! This binary provides an HTTP API for interacting with the RGA CRDT
//! using the Axum web framework.

use std::net::SocketAddr;
use tokio;
use tracing::{Level, info};
use tracing_subscriber;

mod crdt;
mod server;

use server::create_router;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting RGA CRDT Axum server...");

    // Build our application with routes from the server module
    let app = create_router();

    // Define the address to bind to
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    info!("Server listening on http://{}", addr);
    info!("Available endpoints:");
    info!("  GET  /        - Hello message");
    info!("  GET  /health  - Health check");
    info!("  POST /messages - Create message");
    info!("");
    info!("Try these commands:");
    info!("  curl http://localhost:3000/");
    info!("  curl http://localhost:3000/health");
    info!(
        "  curl -X POST http://localhost:3000/messages -H 'Content-Type: application/json' -d '{{\"content\":\"Hello World\"}}'"
    );

    // Run the server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
