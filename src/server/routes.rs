//! Route handlers for the RGA CRDT web server.
//!
//! This module contains all the HTTP route handlers and related types for the Axum server.

use axum::{
    Router,
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct HelloResponse {
    pub message: String,
}

#[derive(Deserialize)]
pub struct CreateMessage {
    pub content: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub id: u32,
    pub content: String,
    pub timestamp: String,
}

/// Basic health check endpoint
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        message: "Server is running!".to_string(),
    })
}

/// Simple hello world endpoint
pub async fn hello() -> Json<HelloResponse> {
    Json(HelloResponse {
        message: "Hello from Axum server!".to_string(),
    })
}

/// Simple POST endpoint example
pub async fn create_message(Json(payload): Json<CreateMessage>) -> Json<MessageResponse> {
    Json(MessageResponse {
        id: 1,
        content: payload.content,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// Creates and configures the main application router
pub fn create_router() -> Router {
    Router::new()
        .route("/", get(hello))
        .route("/health", get(health))
        .route("/messages", post(create_message))
}
