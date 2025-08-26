//! Replica identifier type and related functionality.
//!
//! This module contains the definition of ReplicaId, which uniquely identifies
//! each participant in the distributed CRDT system.

/// A unique identifier for each replica (collaborator) in the distributed system.
///
/// Each participant in the collaborative editing system should have a unique replica ID.
/// This ensures that operations from different replicas can be distinguished and ordered.
pub type ReplicaId = u64;
