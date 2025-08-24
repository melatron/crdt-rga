//! Node definition and related constants for the RGA CRDT.
//!
//! This module contains the Node struct which represents individual characters
//! in the RGA, along with sentinel constants used to mark document boundaries.

use crate::types::UniqueId;

/// Special sentinel characters that mark the beginning and end of the document.
/// These are fixed points of reference for all replicas.
///
/// These characters are chosen from Unicode's "Miscellaneous Technical" block
/// to avoid conflicts with normal text content.
pub const SENTINEL_START_CHAR: char = '\u{2388}'; // Symbol for "begin"
pub const SENTINEL_END_CHAR: char = '\u{2389}'; // Symbol for "end"

/// Represents a single character within the RGA.
///
/// Each node contains:
/// - A unique identifier that determines its position in the total order
/// - The character content
/// - A deletion flag that acts as a tombstone for logical deletion
///
/// # Tombstone Deletion
///
/// Instead of physically removing nodes, the RGA uses logical deletion by setting
/// `is_deleted` to true. This ensures that the structure remains consistent across
/// replicas and allows for proper handling of concurrent operations.
#[derive(Debug, Clone)]
pub struct Node {
    /// Unique identifier that determines this node's position in the sequence
    pub id: UniqueId,
    /// The character content of this node
    pub character: char,
    /// Whether this node has been logically deleted (tombstone)
    pub is_deleted: bool,
}

impl Node {
    /// Creates a new node with the given ID and character.
    /// The node is initially not deleted.
    pub fn new(id: UniqueId, character: char) -> Self {
        Node {
            id,
            character,
            is_deleted: false,
        }
    }

    /// Creates a new deleted node (tombstone) with the given ID and character.
    pub fn new_deleted(id: UniqueId, character: char) -> Self {
        Node {
            id,
            character,
            is_deleted: true,
        }
    }

    /// Creates the sentinel start node.
    /// This node always has the smallest possible UniqueId to ensure it appears first.
    pub fn sentinel_start() -> Self {
        Node {
            id: UniqueId::new(0, 0),
            character: SENTINEL_START_CHAR,
            is_deleted: false,
        }
    }

    /// Creates the sentinel end node.
    /// This node always has the largest possible UniqueId to ensure it appears last.
    pub fn sentinel_end() -> Self {
        Node {
            id: UniqueId::new(u64::MAX, u64::MAX),
            character: SENTINEL_END_CHAR,
            is_deleted: false,
        }
    }

    /// Returns true if this node is a sentinel (start or end).
    pub fn is_sentinel(&self) -> bool {
        self.character == SENTINEL_START_CHAR || self.character == SENTINEL_END_CHAR
    }

    /// Returns true if this node is visible (not deleted and not a sentinel).
    pub fn is_visible(&self) -> bool {
        !self.is_deleted && !self.is_sentinel()
    }

    /// Marks this node as deleted (creates a tombstone).
    /// Sentinel nodes cannot be deleted.
    pub fn delete(&mut self) -> Result<(), &'static str> {
        if self.is_sentinel() {
            Err("Cannot delete sentinel nodes")
        } else {
            self.is_deleted = true;
            Ok(())
        }
    }

    /// Marks this node as not deleted (resurrects a tombstone).
    /// This is useful for handling concurrent operations.
    pub fn undelete(&mut self) {
        self.is_deleted = false;
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::UniqueId;

    #[test]
    fn test_node_creation() {
        let id = UniqueId::new(1, 1);
        let node = Node::new(id, 'A');

        assert_eq!(node.id, id);
        assert_eq!(node.character, 'A');
        assert!(!node.is_deleted);
    }

    #[test]
    fn test_node_deletion() {
        let id = UniqueId::new(1, 1);
        let mut node = Node::new(id, 'A');

        assert!(node.delete().is_ok());
        assert!(node.is_deleted);
    }

    #[test]
    fn test_sentinel_nodes() {
        let start = Node::sentinel_start();
        let end = Node::sentinel_end();

        assert!(start.is_sentinel());
        assert!(end.is_sentinel());
        assert!(start < end); // Start should come before end

        // Cannot delete sentinels
        let mut start_mut = start;
        let mut end_mut = end;
        assert!(start_mut.delete().is_err());
        assert!(end_mut.delete().is_err());
    }

    #[test]
    fn test_node_visibility() {
        let id = UniqueId::new(1, 1);
        let mut node = Node::new(id, 'A');
        let start = Node::sentinel_start();

        assert!(node.is_visible());
        assert!(!start.is_visible()); // Sentinel not visible

        node.delete().unwrap();
        assert!(!node.is_visible()); // Deleted not visible
    }

    #[test]
    fn test_node_ordering() {
        let id1 = UniqueId::new(1, 1);
        let id2 = UniqueId::new(2, 1);
        let node1 = Node::new(id1, 'A');
        let node2 = Node::new(id2, 'B');

        assert!(node1 < node2);
    }
}
