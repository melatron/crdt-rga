//! Core RGA CRDT implementation.
//!
//! This module contains the main RGA (Replicated Growable Array) struct and its operations.
//! The RGA provides a conflict-free replicated data type suitable for collaborative text editing.

use crossbeam_skiplist::SkipMap;
use parking_lot::RwLock;
use std::sync::Arc;

use crate::crdt::node::Node;
use crate::crdt::types::{LamportClock, LamportTimestamp, ReplicaId, UniqueId};

/// The Replicated Growable Array (RGA) CRDT.
///
/// The RGA uses a concurrent SkipMap to store nodes, providing O(log n) operations
/// with lock-free concurrent access for high performance.
///
/// # Design
///
/// - Uses Lamport timestamps with sequence numbers for strong causal ordering
/// - SkipMap for concurrent lock-free operations
/// - Tombstone-based deletion for consistency
/// - Sentinel nodes for stable reference points
/// - Thread-safe Lamport clock for timestamp generation
pub struct RGA {
    /// The unique identifier for this replica
    replica_id: ReplicaId,
    /// Thread-safe Lamport clock for generating new timestamps
    clock: LamportClock,
    /// The core data store: a concurrent SkipMap mapping `UniqueId` to `Node`
    /// SkipMap provides lock-free concurrent operations with ordered traversal
    skipmap: Arc<SkipMap<UniqueId, Arc<RwLock<Node>>>>,
}

impl RGA {
    /// Creates a new RGA instance, initialized with sentinel nodes.
    ///
    /// # Arguments
    ///
    /// * `replica_id` - Unique identifier for this replica
    ///
    /// # Returns
    ///
    /// A new RGA instance with sentinel start and end nodes
    pub fn new(replica_id: ReplicaId) -> Self {
        let skipmap = Arc::new(SkipMap::new());

        // Insert sentinel nodes
        let start_node = Node::sentinel_start();
        let end_node = Node::sentinel_end();

        skipmap.insert(start_node.id, Arc::new(RwLock::new(start_node)));
        skipmap.insert(end_node.id, Arc::new(RwLock::new(end_node)));

        RGA {
            replica_id,
            clock: LamportClock::new(replica_id),
            skipmap,
        }
    }

    /// Gets the replica ID for this RGA instance.
    pub fn replica_id(&self) -> ReplicaId {
        self.replica_id
    }

    /// Gets the current clock value (for debugging/testing).
    pub fn current_clock(&self) -> u64 {
        self.clock.current_counter()
    }

    /// Generates a new unique identifier for a local operation.
    ///
    /// Uses the thread-safe Lamport clock to generate timestamps.
    fn new_local_id(&self) -> UniqueId {
        UniqueId::from(self.clock.tick())
    }

    /// Updates the local Lamport clock based on a received timestamp.
    ///
    /// This ensures causal consistency when receiving remote operations.
    fn update_clock(&self, received_timestamp: LamportTimestamp) {
        self.clock.update(received_timestamp);
    }

    /// Inserts a character after the node identified by `after_id`.
    ///
    /// This method generates a new `UniqueId` for the inserted character.
    /// The B-tree's natural ordering handles placement according to the
    /// total order defined by UniqueId.
    ///
    /// # Arguments
    ///
    /// * `after_id` - The UniqueId of the node to insert after
    /// * `character` - The character to insert
    ///
    /// # Returns
    ///
    /// * `Ok(UniqueId)` - The ID of the newly inserted node
    /// * `Err(&str)` - Error message if the operation fails
    pub fn insert_after(
        &self,
        after_id: UniqueId,
        character: char,
    ) -> Result<UniqueId, &'static str> {
        let new_node_id = self.new_local_id();
        let new_node = Node::new(new_node_id, character);

        // Check if `after_id` exists. If not, we can't insert after it.
        if !self.skipmap.contains_key(&after_id) {
            return Err("Reference node for insertion not found");
        }

        // The SkipMap automatically handles placing `new_node` according to its `id`.
        // The `UniqueId` (Lamport timestamp + replica ID + sequence) ensures a globally consistent sort order.
        self.skipmap
            .insert(new_node.id, Arc::new(RwLock::new(new_node)));
        Ok(new_node_id)
    }

    /// Logically deletes a character identified by its `UniqueId`.
    ///
    /// This sets the `is_deleted` flag to true (tombstone approach).
    ///
    /// # Arguments
    ///
    /// * `id_to_delete` - The UniqueId of the node to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the deletion was successful
    /// * `Err(&str)` - Error message if the operation fails
    pub fn delete(&self, id_to_delete: UniqueId) -> Result<(), &'static str> {
        if let Some(entry) = self.skipmap.get(&id_to_delete) {
            let mut node = entry.value().write();
            node.delete()
        } else {
            Err("Node to delete not found")
        }
    }

    /// Applies a remote operation by integrating a received `Node` into the local RGA.
    ///
    /// This implicitly handles concurrent inserts/deletes due to CRDT properties.
    /// The method updates the local Lamport clock and integrates the remote node.
    ///
    /// # Arguments
    ///
    /// * `remote_node` - The node received from a remote replica
    pub fn apply_remote_op(&self, remote_node: Node) {
        // Update local Lamport clock
        self.update_clock(remote_node.id.timestamp());

        // Insert or update the remote node. SkipMap handles sorting by UniqueId.
        // If a node with the same ID already exists, it gets replaced
        // (which is important for updates like `is_deleted`).
        self.skipmap
            .insert(remote_node.id, Arc::new(RwLock::new(remote_node)));
    }

    /// Returns the current visible content of the RGA as a String.
    ///
    /// Filters out deleted nodes and sentinel characters to show only
    /// the actual document content.
    pub fn to_string(&self) -> String {
        self.skipmap
            .iter()
            .filter_map(|entry| {
                let node = entry.value().read();
                if node.is_visible() {
                    Some(node.character)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns all nodes (including deleted and sentinel) for debugging.
    pub fn all_nodes(&self) -> Vec<Node> {
        self.skipmap
            .iter()
            .map(|entry| entry.value().read().clone())
            .collect()
    }

    /// Returns only visible nodes (excluding deleted and sentinel nodes).
    pub fn visible_nodes(&self) -> Vec<Node> {
        self.skipmap
            .iter()
            .filter_map(|entry| {
                let node = entry.value().read();
                if node.is_visible() {
                    Some(node.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Gets the number of total nodes (including deleted and sentinel).
    pub fn total_node_count(&self) -> usize {
        self.skipmap.len()
    }

    /// Gets the number of visible nodes (excluding deleted and sentinel).
    pub fn visible_node_count(&self) -> usize {
        self.skipmap
            .iter()
            .filter(|entry| entry.value().read().is_visible())
            .count()
    }

    /// For debugging: prints all nodes including sentinels and deleted.
    pub fn dump_nodes(&self) {
        println!("--- RGA Node Dump (Replica ID: {}) ---", self.replica_id);
        for entry in self.skipmap.iter() {
            let id = entry.key();
            let node = entry.value().read();
            let status = if node.is_sentinel() {
                "SENTINEL"
            } else if node.is_deleted {
                "DELETED"
            } else {
                "ACTIVE"
            };
            println!("{:?} -> Char: '{}', Status: {}", id, node.character, status);
        }
        println!("Content: '{}'", self.to_string());
        println!("------------------------------------");
    }

    /// Finds a node by its character (useful for examples/testing).
    /// Returns the first non-deleted node with the given character.
    pub fn find_node_by_char(&self, character: char) -> Option<UniqueId> {
        self.skipmap.iter().find_map(|entry| {
            let node = entry.value().read();
            if node.character == character && !node.is_deleted {
                Some(node.id)
            } else {
                None
            }
        })
    }

    /// Gets the sentinel start node ID.
    pub fn sentinel_start_id(&self) -> UniqueId {
        Node::sentinel_start().id
    }

    /// Gets the sentinel end node ID.
    pub fn sentinel_end_id(&self) -> UniqueId {
        Node::sentinel_end().id
    }
}

impl Clone for RGA {
    fn clone(&self) -> Self {
        let skipmap_clone = Arc::new(SkipMap::new());

        // Copy all entries from the original skipmap
        for entry in self.skipmap.iter() {
            let node = entry.value().read().clone();
            skipmap_clone.insert(*entry.key(), Arc::new(RwLock::new(node)));
        }

        RGA {
            replica_id: self.replica_id,
            clock: LamportClock::new(self.replica_id),
            skipmap: skipmap_clone,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rga_creation() {
        let rga = RGA::new(1);
        assert_eq!(rga.replica_id(), 1);
        assert_eq!(rga.current_clock(), 0);
        assert_eq!(rga.total_node_count(), 2); // Start and end sentinels
        assert_eq!(rga.visible_node_count(), 0);
        assert_eq!(rga.to_string(), "");
    }

    #[test]
    fn test_basic_insertion() {
        let rga = RGA::new(1);
        let start_id = rga.sentinel_start_id();

        let char_id = rga.insert_after(start_id, 'A').unwrap();
        assert_eq!(rga.to_string(), "A");
        assert_eq!(rga.visible_node_count(), 1);

        // Insert after the 'A'
        rga.insert_after(char_id, 'B').unwrap();
        assert_eq!(rga.to_string(), "AB");
        assert_eq!(rga.visible_node_count(), 2);
    }

    #[test]
    fn test_deletion() {
        let rga = RGA::new(1);
        let start_id = rga.sentinel_start_id();

        let char_id = rga.insert_after(start_id, 'A').unwrap();
        assert_eq!(rga.to_string(), "A");

        rga.delete(char_id).unwrap();
        assert_eq!(rga.to_string(), "");
        assert_eq!(rga.visible_node_count(), 0);
        assert_eq!(rga.total_node_count(), 3); // Still has the tombstone
    }

    #[test]
    fn test_remote_operations() {
        let rga1 = RGA::new(1);
        let rga2 = RGA::new(2);

        // RGA1 inserts 'A'
        let start_id = rga1.sentinel_start_id();
        let a_id = rga1.insert_after(start_id, 'A').unwrap();

        // Apply RGA1's operation to RGA2
        let node_a = rga1.all_nodes().into_iter().find(|n| n.id == a_id).unwrap();
        rga2.apply_remote_op(node_a);

        assert_eq!(rga1.to_string(), rga2.to_string());
        assert_eq!(rga2.to_string(), "A");
    }

    #[test]
    fn test_concurrent_operations() {
        let rga1 = RGA::new(1);
        let rga2 = RGA::new(2);
        let start_id = rga1.sentinel_start_id();

        // Concurrent insertions
        let a_id = rga1.insert_after(start_id, 'A').unwrap();
        let b_id = rga2.insert_after(start_id, 'B').unwrap();

        // Cross-replicate
        let node_a = rga1.all_nodes().into_iter().find(|n| n.id == a_id).unwrap();
        let node_b = rga2.all_nodes().into_iter().find(|n| n.id == b_id).unwrap();

        rga2.apply_remote_op(node_a);
        rga1.apply_remote_op(node_b);

        // Both should converge to the same state
        assert_eq!(rga1.to_string(), rga2.to_string());
        // Due to UniqueId ordering, 'A' (from replica 1) should come before 'B' (from replica 2)
        assert_eq!(rga1.to_string(), "AB");
    }
}
