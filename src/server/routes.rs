//! Route handlers for the RGA CRDT web server.
//!
//! This module contains all the HTTP route handlers and related types for the Axum server.

use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::{Json, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::crdt::RGA;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub message: String,
}

/// Shared application state
pub type AppState = Arc<RwLock<RGA>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct RGAOperation {
    #[serde(rename = "type")]
    pub op_type: String,
    pub character: Option<char>,
    pub after_id: Option<String>,
    pub delete_id: Option<String>,
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
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle individual WebSocket connections
async fn handle_socket(mut socket: WebSocket, state: AppState) {
    info!("New WebSocket connection established");

    // Send current document content
    {
        let rga = state.read().await;
        let content = rga.to_string();
        if socket
            .send(Message::Text(format!(
                r#"{{"type":"init","content":"{}"}}"#,
                content
            )))
            .await
            .is_err()
        {
            return;
        }
    }

    // Handle incoming messages
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                info!("Received: {}", text);

                // Try to parse as RGA operation
                if let Ok(operation) = serde_json::from_str::<RGAOperation>(&text) {
                    let mut rga = state.write().await;

                    match operation.op_type.as_str() {
                        "insert" => {
                            if let (Some(character), Some(after_id_str)) =
                                (operation.character, operation.after_id)
                            {
                                // For now, insert after start (we'll improve this later)
                                let start_id = rga.sentinel_start_id();
                                if let Ok(_new_id) = rga.insert_after(start_id, character) {
                                    let content = rga.to_string();
                                    let response =
                                        format!(r#"{{"type":"update","content":"{}"}}"#, content);
                                    if socket.send(Message::Text(response)).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                        "get_content" => {
                            let content = rga.to_string();
                            let response =
                                format!(r#"{{"type":"content","content":"{}"}}"#, content);
                            if socket.send(Message::Text(response)).await.is_err() {
                                break;
                            }
                        }
                        _ => {
                            info!("Unknown operation type: {}", operation.op_type);
                        }
                    }
                } else {
                    info!("Failed to parse operation: {}", text);
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket connection closed");
                break;
            }
            _ => {}
        }
    }

    info!("WebSocket connection ended");
}

/// Creates and configures the main application router
pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/ws", get(ws_handler))
}
