//! Thread-safe Lamport clock implementation for generating timestamps.
//!
//! This module contains the LamportClock struct which provides thread-safe
//! generation of Lamport timestamps for maintaining causal ordering in the CRDT.

use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};

use crate::crdt::types::replica::ReplicaId;
use crate::crdt::types::timestamp::LamportTimestamp;

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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_clock_sequence_numbering() {
        let clock = LamportClock::new(5);

        let ts1 = clock.tick();
        let ts2 = clock.tick();

        assert_eq!(ts1.replica_id, 5);
        assert_eq!(ts2.replica_id, 5);
        assert_eq!(ts1.sequence + 1, ts2.sequence);
        assert!(ts1 < ts2);
    }

    #[test]
    fn test_clock_replica_id() {
        let clock = LamportClock::new(42);
        assert_eq!(clock.replica_id(), 42);

        let ts = clock.tick();
        assert_eq!(ts.replica_id, 42);
    }
}
