//! # RGA CRDT - Replicated Growable Array
//!
//! A Conflict-free Replicated Data Type (CRDT) implementation of a replicated growable array,
//! suitable for collaborative text editing and similar applications where concurrent modifications
//! need to be merged consistently across distributed systems.
//!
//! ## Features
//!
//! - **Conflict-free**: Concurrent operations can be applied in any order and will converge
//! - **Causally consistent**: Operations maintain causal relationships through Lamport timestamps
//! - **Efficient**: Uses SkipMap for O(log n) operations
//! - **Tombstone-based deletion**: Supports safe deletion with eventual consistency
//!
//! ## Example
//!
//! ```rust
//! use crdt_rga::RGA;
//!
//! let mut rga = RGA::new(1); // replica ID = 1
//! // Insert operations...
//! println!("Content: {}", rga.to_string());
//! ```

pub mod crdt;

// Re-export the main public API from the CRDT module
pub use crdt::{LamportClock, LamportTimestamp, ReplicaId, UniqueId};
pub use crdt::{Node, RGA, SENTINEL_END_CHAR, SENTINEL_START_CHAR};
