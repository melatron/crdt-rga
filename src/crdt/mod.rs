//! CRDT (Conflict-free Replicated Data Type) implementation module.
//!
//! This module contains the RGA (Replicated Growable Array) CRDT implementation
//! and all its supporting types and structures.

pub mod node;
pub mod rga;
pub mod types;

// Re-export the main public API
pub use node::{Node, SENTINEL_END_CHAR, SENTINEL_START_CHAR};
pub use rga::RGA;
pub use types::{LamportClock, LamportTimestamp, ReplicaId, UniqueId};
