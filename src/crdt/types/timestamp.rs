//! Lamport timestamp implementation for causal ordering in distributed systems.
//!
//! This module contains the LamportTimestamp struct which provides a total ordering
//! of events across replicas in the CRDT system.

use std::cmp::Ordering;

use crate::crdt::types::replica::ReplicaId;

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
    fn test_sequence_ordering() {
        let ts1 = LamportTimestamp {
            counter: 1,
            replica_id: 1,
            sequence: 0,
        };
        let ts2 = LamportTimestamp {
            counter: 1,
            replica_id: 1,
            sequence: 1,
        };
        let ts3 = LamportTimestamp {
            counter: 1,
            replica_id: 2,
            sequence: 0,
        };

        assert!(ts1 < ts2); // Same counter/replica, different sequence
        assert!(ts1 < ts3); // Same counter, different replica
        assert!(ts3 < ts2); // Lower replica wins even with higher sequence
    }
}
