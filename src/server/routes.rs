//! Route handlers for the RGA CRDT web server.
//!
//! This module contains HTTP route definitions and delegates WebSocket handling
//! to the dedicated websocket module.

use axum::{
    Router,
    extract::{State, ws::WebSocketUpgrade},
    response::{Json, Response},
    routing::get,
};
use serde::Serialize;

use crate::server::websocket::{AppState, handle_websocket_connection};

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub message: String,
}

/// Basic health check endpoint
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        message: "Server is running!".to_string(),
    })
}

/// WebSocket connection handler for collaborative editing
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_websocket_connection(socket, state))
}

/// Creates and configures the main application router
pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/ws", get(ws_handler))
}
