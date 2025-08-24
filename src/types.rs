//! Fundamental type definitions for the RGA CRDT.
//!
//! This module contains the core types used throughout the RGA implementation,
//! including replica identifiers, timestamps, and unique identifiers.

use std::cmp::Ordering;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};

/// A unique identifier for each replica (collaborator) in the distributed system.
///
/// Each participant in the collaborative editing system should have a unique replica ID.
/// This ensures that operations from different replicas can be distinguished and ordered.
pub type ReplicaId = u64;

/// A Lamport timestamp, consisting of a logical counter and the originating replica's ID.
///
/// This allows for a total ordering of events across replicas, which is essential for
/// ensuring convergence in the CRDT. The combination of counter and replica_id ensures
/// that no two operations will have the same timestamp.
///
/// # Ordering
///
/// Lamport timestamps are ordered first by counter, then by replica_id. This ensures
/// a deterministic global ordering of all operations across all replicas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LamportTimestamp {
    /// The logical clock value when this timestamp was created
    pub counter: u64,
    /// The ID of the replica that created this timestamp
    pub replica_id: ReplicaId,
    /// Sequence number within the same counter value for additional ordering
    pub sequence: u32,
}

impl PartialOrd for LamportTimestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LamportTimestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare by counter (logical time)
        match self.counter.cmp(&other.counter) {
            Ordering::Equal => {
                // If counters are equal, compare by sequence number
                match self.sequence.cmp(&other.sequence) {
                    Ordering::Equal => {
                        // If sequence is also equal, compare by replica_id for deterministic ordering
                        self.replica_id.cmp(&other.replica_id)
                    }
                    other => other,
                }
            }
            other => other,
        }
    }
}

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

/// A thread-safe clock for generating Lamport timestamps
pub struct LamportClock {
    counter: AtomicU64,
    replica_id: ReplicaId,
    sequence: AtomicU64,
}

impl LamportClock {
    /// Creates a new Lamport clock
    pub fn new(replica_id: ReplicaId) -> Self {
        LamportClock {
            counter: AtomicU64::new(0),
            replica_id,
            sequence: AtomicU64::new(0),
        }
    }

    /// Generates the next timestamp for this replica
    pub fn tick(&self) -> LamportTimestamp {
        let counter = self.counter.fetch_add(1, AtomicOrdering::SeqCst) + 1;
        let sequence = self.sequence.fetch_add(1, AtomicOrdering::SeqCst);

        LamportTimestamp {
            counter,
            replica_id: self.replica_id,
            sequence: sequence as u32,
        }
    }

    /// Updates the clock based on a received timestamp (for causal consistency)
    pub fn update(&self, received_timestamp: LamportTimestamp) {
        let current = self.counter.load(AtomicOrdering::SeqCst);
        let new_counter = current.max(received_timestamp.counter);

        // Use compare_and_swap in a loop to ensure we don't go backwards
        let mut current_val = current;
        while current_val < new_counter {
            match self.counter.compare_exchange_weak(
                current_val,
                new_counter,
                AtomicOrdering::SeqCst,
                AtomicOrdering::SeqCst,
            ) {
                Ok(_) => break,
                Err(actual) => current_val = actual,
            }
        }
    }

    /// Gets the current counter value (for debugging)
    pub fn current_counter(&self) -> u64 {
        self.counter.load(AtomicOrdering::SeqCst)
    }

    /// Gets the replica ID
    pub fn replica_id(&self) -> ReplicaId {
        self.replica_id
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
    fn test_lamport_timestamp_ordering() {
        let ts1 = LamportTimestamp {
            counter: 1,
            replica_id: 1,
            sequence: 0,
        };
        let ts2 = LamportTimestamp {
            counter: 1,
            replica_id: 2,
            sequence: 0,
        };
        let ts3 = LamportTimestamp {
            counter: 2,
            replica_id: 1,
            sequence: 0,
        };
        let ts4 = LamportTimestamp {
            counter: 1,
            replica_id: 1,
            sequence: 1,
        };

        // Same counter, different replica_id
        assert!(ts1 < ts2);

        // Different counter
        assert!(ts1 < ts3);
        assert!(ts2 < ts3);

        // Same counter and replica, different sequence
        assert!(ts1 < ts4);
    }

    #[test]
    fn test_unique_id_creation() {
        let id = UniqueId::new(5, 10);
        assert_eq!(id.counter(), 5);
        assert_eq!(id.replica_id(), 10);
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
    fn test_lamport_clock() {
        let clock = LamportClock::new(1);

        let ts1 = clock.tick();
        let ts2 = clock.tick();

        assert_eq!(ts1.replica_id, 1);
        assert_eq!(ts2.replica_id, 1);
        assert!(ts1 < ts2);
        assert_eq!(ts1.counter + 1, ts2.counter);
    }

    #[test]
    fn test_lamport_clock_update() {
        let clock = LamportClock::new(1);

        // Simulate receiving a timestamp from a future
        let future_ts = LamportTimestamp {
            counter: 100,
            replica_id: 2,
            sequence: 0,
        };

        clock.update(future_ts);
        let next_ts = clock.tick();

        assert!(next_ts.counter > future_ts.counter);
        assert_eq!(next_ts.replica_id, 1);
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
