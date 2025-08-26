//! WebSocket session management for RGA CRDT collaborative editing.
//!
//! This module handles WebSocket connections, message parsing, RGA operations,
//! and real-time synchronization between multiple clients.

use axum::extract::ws::{Message, WebSocket};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::crdt::RGA;

/// Shared application state containing the RGA CRDT instance
pub type AppState = Arc<RwLock<RGA>>;

/// WebSocket message protocol for RGA operations
#[derive(Serialize, Deserialize, Debug)]
pub struct RGAOperation {
    #[serde(rename = "type")]
    pub op_type: String,
    pub character: Option<char>,
    pub position: Option<usize>,
    pub after_id: Option<String>,
    pub delete_id: Option<String>,
}

/// Response messages sent to clients
#[derive(Serialize, Debug)]
pub struct RGAResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<usize>,
}

/// WebSocket session manager
pub struct WebSocketSession {
    socket: WebSocket,
    state: AppState,
    session_id: String,
}

impl WebSocketSession {
    /// Create a new WebSocket session
    pub fn new(socket: WebSocket, state: AppState, session_id: String) -> Self {
        Self {
            socket,
            state,
            session_id,
        }
    }

    /// Handle the WebSocket connection lifecycle
    pub async fn handle(mut self) {
        info!("WebSocket session {} established", self.session_id);

        // Send initial document state
        if let Err(e) = self.send_initial_state().await {
            error!("Failed to send initial state to {}: {}", self.session_id, e);
            return;
        }

        // Process incoming messages
        while let Some(msg) = self.socket.recv().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Err(e) = self.handle_text_message(&text).await {
                        error!("Error handling message from {}: {}", self.session_id, e);
                        break;
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket session {} closed by client", self.session_id);
                    break;
                }
                Ok(Message::Ping(data)) => {
                    if let Err(e) = self.socket.send(Message::Pong(data)).await {
                        error!("Failed to send pong to {}: {}", self.session_id, e);
                        break;
                    }
                }
                Ok(_) => {
                    // Ignore other message types (binary, pong)
                }
                Err(e) => {
                    warn!("WebSocket error for {}: {}", self.session_id, e);
                    break;
                }
            }
        }

        info!("WebSocket session {} ended", self.session_id);
    }

    /// Send initial document state to newly connected client
    async fn send_initial_state(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let rga = self.state.read().await;
        let content = rga.to_string();
        drop(rga);

        let response = RGAResponse {
            response_type: "init".to_string(),
            content,
            position: None,
        };

        self.send_response(&response).await
    }

    /// Handle incoming text messages
    async fn handle_text_message(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("Session {} received: {}", self.session_id, text);

        match serde_json::from_str::<RGAOperation>(text) {
            Ok(operation) => self.process_rga_operation(operation).await,
            Err(e) => {
                warn!("Failed to parse operation from {}: {}", self.session_id, e);
                Ok(()) // Don't break connection for parse errors
            }
        }
    }

    /// Process RGA operations
    async fn process_rga_operation(
        &mut self,
        operation: RGAOperation,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match operation.op_type.as_str() {
            "insert" => self.handle_insert_operation(operation).await,
            "get_content" => self.handle_get_content_operation().await,
            _ => {
                warn!(
                    "Unknown operation type '{}' from session {}",
                    operation.op_type, self.session_id
                );
                Ok(())
            }
        }
    }

    /// Handle character insertion operations
    async fn handle_insert_operation(
        &mut self,
        operation: RGAOperation,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(character) = operation.character else {
            warn!(
                "Insert operation missing character from session {}",
                self.session_id
            );
            return Ok(());
        };

        let position = operation.position.unwrap_or(0);

        let rga = self.state.write().await;

        // Calculate insertion point based on position
        let after_id = self.calculate_insertion_point(&rga, position);

        match rga.insert_after(after_id, character) {
            Ok(_new_id) => {
                let content = rga.to_string();
                drop(rga);

                let response = RGAResponse {
                    response_type: "update".to_string(),
                    content,
                    position: Some(position),
                };

                self.send_response(&response).await?;
                info!(
                    "Session {} inserted '{}' at position {}",
                    self.session_id, character, position
                );
            }
            Err(e) => {
                error!(
                    "Failed to insert character for session {}: {}",
                    self.session_id, e
                );
            }
        }

        Ok(())
    }

    /// Handle get content operations
    async fn handle_get_content_operation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let rga = self.state.read().await;
        let content = rga.to_string();
        drop(rga);

        let response = RGAResponse {
            response_type: "content".to_string(),
            content,
            position: None,
        };

        self.send_response(&response).await?;
        info!("Session {} requested content", self.session_id);
        Ok(())
    }

    /// Calculate the node ID to insert after based on position
    fn calculate_insertion_point(&self, rga: &RGA, position: usize) -> crate::crdt::UniqueId {
        let visible_nodes = rga.visible_nodes();

        if position == 0 {
            // Insert at beginning
            rga.sentinel_start_id()
        } else if position >= visible_nodes.len() {
            // Insert at end - find the last visible node
            if let Some(last_node) = visible_nodes.last() {
                last_node.id
            } else {
                rga.sentinel_start_id()
            }
        } else {
            // Insert after the node at position-1
            visible_nodes[position - 1].id
        }
    }

    /// Send a response message to the client
    async fn send_response(
        &mut self,
        response: &RGAResponse,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string(response)?;
        self.socket.send(Message::Text(json)).await?;
        Ok(())
    }
}

/// Generate a unique session ID
pub fn generate_session_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    format!("session_{}", timestamp)
}

/// Create and handle a new WebSocket session
pub async fn handle_websocket_connection(socket: WebSocket, state: AppState) {
    let session_id = generate_session_id();
    let session = WebSocketSession::new(socket, state, session_id);
    session.handle().await;
}
