//! Type definitions for the RGA CRDT.
//!
//! This module contains all the fundamental types used throughout the RGA implementation,
//! organized into focused submodules for better maintainability.

pub mod clock;
pub mod replica;
pub mod timestamp;
pub mod unique_id;

// Re-export all public types for backward compatibility
pub use clock::LamportClock;
pub use replica::ReplicaId;
pub use timestamp::LamportTimestamp;
pub use unique_id::UniqueId;
