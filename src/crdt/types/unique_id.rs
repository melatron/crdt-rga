//! Unique identifier implementation for RGA nodes.
//!
//! This module contains the UniqueId struct which serves as a globally unique
//! identifier for each node in the RGA, providing both identity and ordering.

use crate::crdt::types::replica::ReplicaId;
use crate::crdt::types::timestamp::LamportTimestamp;

/// A unique identifier for each character/node in the RGA.
///
/// This is derived directly from the Lamport timestamp, ensuring global uniqueness and ordering.
/// The UniqueId serves as both an identifier and determines the position of elements in the
/// final sequence.
///
/// # Design Notes
///
/// The UniqueId is a newtype wrapper around LamportTimestamp to provide type safety and
/// make the API clearer. It inherits all the ordering properties of LamportTimestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniqueId(pub LamportTimestamp);

impl UniqueId {
    /// Creates a new UniqueId from a counter and replica_id
    pub fn new(counter: u64, replica_id: ReplicaId) -> Self {
        UniqueId(LamportTimestamp {
            counter,
            replica_id,
            sequence: 0,
        })
    }

    /// Creates a new UniqueId with a specific sequence number
    pub fn new_with_sequence(counter: u64, replica_id: ReplicaId, sequence: u32) -> Self {
        UniqueId(LamportTimestamp {
            counter,
            replica_id,
            sequence,
        })
    }

    /// Gets the underlying LamportTimestamp
    pub fn timestamp(&self) -> LamportTimestamp {
        self.0
    }

    /// Gets the counter value from the timestamp
    pub fn counter(&self) -> u64 {
        self.0.counter
    }

    /// Gets the replica_id from the timestamp
    pub fn replica_id(&self) -> ReplicaId {
        self.0.replica_id
    }

    /// Gets the sequence number from the timestamp
    pub fn sequence(&self) -> u32 {
        self.0.sequence
    }
}

impl From<LamportTimestamp> for UniqueId {
    fn from(timestamp: LamportTimestamp) -> Self {
        UniqueId(timestamp)
    }
}

impl From<UniqueId> for LamportTimestamp {
    fn from(id: UniqueId) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_id_creation() {
        let id = UniqueId::new(5, 10);
        assert_eq!(id.counter(), 5);
        assert_eq!(id.replica_id(), 10);
        assert_eq!(id.sequence(), 0);
    }

    #[test]
    fn test_unique_id_with_sequence() {
        let id = UniqueId::new_with_sequence(5, 10, 3);
        assert_eq!(id.counter(), 5);
        assert_eq!(id.replica_id(), 10);
        assert_eq!(id.sequence(), 3);
    }

    #[test]
    fn test_unique_id_ordering() {
        let id1 = UniqueId::new(1, 1);
        let id2 = UniqueId::new(1, 2);
        let id3 = UniqueId::new(2, 1);

        assert!(id1 < id2);
        assert!(id1 < id3);
        assert!(id2 < id3);
    }

    #[test]
    fn test_conversion_between_types() {
        let timestamp = LamportTimestamp {
            counter: 42,
            replica_id: 7,
            sequence: 5,
        };
        let id: UniqueId = timestamp.into();
        let back_to_timestamp: LamportTimestamp = id.into();

        assert_eq!(timestamp, back_to_timestamp);
        assert_eq!(id.timestamp(), timestamp);
    }

    #[test]
    fn test_sequence_ordering() {
        let id1 = UniqueId::new_with_sequence(1, 1, 0);
        let id2 = UniqueId::new_with_sequence(1, 1, 1);
        let id3 = UniqueId::new_with_sequence(1, 2, 0);

        assert!(id1 < id2); // Same counter/replica, different sequence
        assert!(id1 < id3); // Same counter, different replica
        assert!(id3 < id2); // Lower replica wins even with higher sequence
    }
}
