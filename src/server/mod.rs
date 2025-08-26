//! Web server module for the RGA CRDT service.
//!
//! This module contains the Axum web server implementation that provides
//! HTTP endpoints for interacting with the RGA CRDT.

pub mod routes;
pub mod websocket;

// Re-export main server functionality
pub use routes::*;
