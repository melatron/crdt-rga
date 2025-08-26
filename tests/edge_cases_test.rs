//! Edge cases integration tests for the RGA CRDT implementation.
//!
//! These tests verify the robustness of the RGA CRDT under various edge conditions
//! including boundary values, error conditions, and stress scenarios.

use crdt_rga::{RGA, SENTINEL_END_CHAR, SENTINEL_START_CHAR, UniqueId};

#[test]
fn test_sentinel_deletion_protection() {
    let rga = RGA::new(1);

    // Cannot delete sentinel start
    let start_id = rga.sentinel_start_id();
    let result = rga.delete(start_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Cannot delete sentinel nodes");

    // Cannot delete sentinel end
    let end_id = rga.sentinel_end_id();
    let result = rga.delete(end_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Cannot delete sentinel nodes");

    // Verify sentinels are still there
    assert_eq!(rga.total_node_count(), 2); // Only sentinels
    assert_eq!(rga.visible_node_count(), 0);
}

#[test]
fn test_large_document_operations() {
    let rga = RGA::new(1);
    let start_id = rga.sentinel_start_id();

    // Insert a large number of characters
    let large_size = 10_000usize;
    let mut last_id = start_id;

    // Build a large document
    for i in 0..large_size {
        let ch = char::from_u32(65 + (i % 26) as u32).unwrap(); // A-Z cycling
        last_id = rga.insert_after(last_id, ch).unwrap();
    }

    assert_eq!(rga.visible_node_count(), large_size);
    assert_eq!(rga.to_string().len(), large_size);

    // Delete every other character
    let all_nodes = rga.all_nodes();
    let mut deleted_count = 0;
    for (i, node) in all_nodes.iter().enumerate() {
        if !node.is_sentinel() && i % 2 == 0 {
            rga.delete(node.id).unwrap();
            deleted_count += 1;
        }
    }

    assert_eq!(rga.visible_node_count(), large_size - deleted_count);
    assert_eq!(rga.total_node_count(), large_size + 2); // Including sentinels
}

#[test]
fn test_unicode_edge_cases() {
    let rga = RGA::new(1);
    let start_id = rga.sentinel_start_id();

    // Test various Unicode characters
    let unicode_chars = ['🦀', '∂', '∑', '∆', '€', '中', '🌟', '😀', '🔥'];
    let mut last_id = start_id;

    for &ch in &unicode_chars {
        last_id = rga.insert_after(last_id, ch).unwrap();
    }

    let result = rga.to_string();
    assert_eq!(result.chars().count(), unicode_chars.len());

    // Verify each character is present
    for &expected_char in &unicode_chars {
        assert!(result.contains(expected_char));
    }
}

#[test]
fn test_null_and_control_characters() {
    let rga = RGA::new(1);
    let start_id = rga.sentinel_start_id();

    // Test control characters and null
    let control_chars = ['\0', '\t', '\n', '\r', '\x1F', '\x7F'];
    let mut last_id = start_id;

    for &ch in &control_chars {
        last_id = rga.insert_after(last_id, ch).unwrap();
    }

    assert_eq!(rga.visible_node_count(), control_chars.len());
    let result = rga.to_string();
    assert_eq!(result.len(), control_chars.len());

    // Verify we can still operate on the document
    let all_visible = rga.visible_nodes();
    assert_eq!(all_visible.len(), control_chars.len());
}

#[test]
fn test_extreme_replica_ids() {
    // Test with maximum replica ID
    let rga_max = RGA::new(u64::MAX);
    let start_id = rga_max.sentinel_start_id();
    let _char_id = rga_max.insert_after(start_id, 'M').unwrap();
    assert_eq!(rga_max.to_string(), "M");

    // Test with zero replica ID
    let rga_zero = RGA::new(0);
    let start_id = rga_zero.sentinel_start_id();
    let _char_id = rga_zero.insert_after(start_id, 'Z').unwrap();
    assert_eq!(rga_zero.to_string(), "Z");

    // Test convergence between extreme IDs
    let node_from_max = rga_max
        .all_nodes()
        .into_iter()
        .find(|n| n.character == 'M' && !n.is_sentinel())
        .unwrap();
    let node_from_zero = rga_zero
        .all_nodes()
        .into_iter()
        .find(|n| n.character == 'Z' && !n.is_sentinel())
        .unwrap();

    rga_zero.apply_remote_op(node_from_max);
    rga_max.apply_remote_op(node_from_zero);

    assert_eq!(rga_zero.to_string(), rga_max.to_string());
    assert_eq!(rga_zero.visible_node_count(), 2);
}

#[test]
fn test_invalid_operations() {
    let rga = RGA::new(1);

    // Test insertion after non-existent ID
    let fake_id = UniqueId::new(999_999, 999_999);
    let result = rga.insert_after(fake_id, 'X');
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Reference node for insertion not found"
    );

    // Test deletion of non-existent ID
    let result = rga.delete(fake_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Node to delete not found");

    // Verify RGA state unchanged
    assert_eq!(rga.visible_node_count(), 0);
    assert_eq!(rga.total_node_count(), 2); // Only sentinels
}

#[test]
fn test_concurrent_deletion_same_node() {
    let rga1 = RGA::new(1);
    let rga2 = RGA::new(2);
    let start_id = rga1.sentinel_start_id();

    // Both replicas have the same character
    let char_id = rga1.insert_after(start_id, 'A').unwrap();
    let node_a = rga1
        .all_nodes()
        .into_iter()
        .find(|n| n.character == 'A')
        .unwrap();
    rga2.apply_remote_op(node_a.clone());

    // Both try to delete the same character
    rga1.delete(char_id).unwrap();
    rga2.delete(char_id).unwrap();

    // Sync the deletion operations
    let rga1_nodes = rga1.all_nodes();
    let rga2_nodes = rga2.all_nodes();

    for node in rga1_nodes {
        rga2.apply_remote_op(node);
    }
    for node in rga2_nodes {
        rga1.apply_remote_op(node);
    }

    // Both should converge to empty document
    assert_eq!(rga1.to_string(), "");
    assert_eq!(rga2.to_string(), "");
    assert_eq!(rga1.visible_node_count(), 0);
    assert_eq!(rga2.visible_node_count(), 0);
}

#[test]
fn test_empty_document_operations() {
    let rga = RGA::new(1);

    // Operations on empty document
    assert_eq!(rga.to_string(), "");
    assert_eq!(rga.visible_node_count(), 0);
    assert_eq!(rga.total_node_count(), 2); // Sentinels only

    let visible_nodes = rga.visible_nodes();
    assert!(visible_nodes.is_empty());

    let all_nodes = rga.all_nodes();
    assert_eq!(all_nodes.len(), 2); // Only sentinels

    // Verify sentinels are correct
    let has_start = all_nodes.iter().any(|n| n.character == SENTINEL_START_CHAR);
    let has_end = all_nodes.iter().any(|n| n.character == SENTINEL_END_CHAR);
    assert!(has_start);
    assert!(has_end);
}

#[test]
fn test_single_character_document() {
    let rga = RGA::new(1);
    let start_id = rga.sentinel_start_id();

    // Insert single character
    let char_id = rga.insert_after(start_id, 'X').unwrap();
    assert_eq!(rga.to_string(), "X");
    assert_eq!(rga.visible_node_count(), 1);

    // Delete the only character
    rga.delete(char_id).unwrap();
    assert_eq!(rga.to_string(), "");
    assert_eq!(rga.visible_node_count(), 0);
    assert_eq!(rga.total_node_count(), 3); // Two sentinels + one tombstone

    // Insert again
    let _char_id2 = rga.insert_after(start_id, 'Y').unwrap();
    assert_eq!(rga.to_string(), "Y");
    assert_eq!(rga.visible_node_count(), 1);
}

#[test]
fn test_rapid_operations_stress() {
    let rga = RGA::new(1);
    let start_id = rga.sentinel_start_id();
    let mut last_id = start_id;

    // Rapidly insert many characters
    let operations = 1000usize;
    for i in 0..operations {
        let ch = char::from_u32(65 + (i % 26) as u32).unwrap();
        last_id = rga.insert_after(last_id, ch).unwrap();
    }

    assert_eq!(rga.visible_node_count(), operations);

    // Rapidly delete characters by going through all nodes
    let all_nodes = rga.all_nodes();
    let mut deleted = 0;
    for node in all_nodes {
        if !node.is_sentinel() && deleted < operations / 2 {
            rga.delete(node.id).unwrap();
            deleted += 1;
        }
    }

    assert_eq!(rga.visible_node_count(), operations - deleted);
    assert_eq!(rga.total_node_count(), operations + 2); // Including sentinels
}

#[test]
fn test_find_node_by_char_edge_cases() {
    let rga = RGA::new(1);
    let start_id = rga.sentinel_start_id();

    // Test finding non-existent character
    assert!(rga.find_node_by_char('X').is_none());

    // Insert and find
    rga.insert_after(start_id, 'A').unwrap();
    let found_id = rga.find_node_by_char('A');
    assert!(found_id.is_some());

    // Delete and try to find (should return None for deleted nodes)
    rga.delete(found_id.unwrap()).unwrap();
    assert!(rga.find_node_by_char('A').is_none());

    // Test with duplicate characters
    let a1_id = rga.insert_after(start_id, 'B').unwrap();
    let _a2_id = rga.insert_after(a1_id, 'B').unwrap();

    // Should find first non-deleted occurrence
    let found = rga.find_node_by_char('B');
    assert!(found.is_some());
    assert_eq!(found.unwrap(), a1_id);
}

#[test]
fn test_clock_progression() {
    let rga = RGA::new(42);
    let initial_clock = rga.current_clock();
    let start_id = rga.sentinel_start_id();

    // Each operation should advance the clock
    rga.insert_after(start_id, 'A').unwrap();
    let clock_after_first = rga.current_clock();
    assert!(clock_after_first > initial_clock);

    let a_id = rga.find_node_by_char('A').unwrap();
    rga.insert_after(a_id, 'B').unwrap();
    let clock_after_second = rga.current_clock();
    assert!(clock_after_second > clock_after_first);

    // Deletion doesn't create new IDs, so clock shouldn't advance
    let b_id = rga.find_node_by_char('B').unwrap();
    rga.delete(b_id).unwrap();
    let clock_after_delete = rga.current_clock();
    assert_eq!(clock_after_delete, clock_after_second);
}

#[test]
fn test_memory_efficiency_with_many_deletes() {
    let rga = RGA::new(1);
    let start_id = rga.sentinel_start_id();
    let mut last_id = start_id;

    // Create and immediately delete many nodes
    let iterations = 1000usize;
    for i in 0..iterations {
        let ch = char::from_u32(65 + (i % 26) as u32).unwrap();
        let new_id = rga.insert_after(last_id, ch).unwrap();

        // Delete immediately (creates tombstones)
        rga.delete(new_id).unwrap();

        last_id = start_id; // Always insert after start for simplicity
    }

    // Should have no visible content but many tombstones
    assert_eq!(rga.visible_node_count(), 0);
    assert_eq!(rga.total_node_count(), iterations + 2); // Including sentinels
    assert_eq!(rga.to_string(), "");

    // All nodes should be either sentinels or deleted
    let all_nodes = rga.all_nodes();
    for node in all_nodes {
        assert!(node.is_sentinel() || node.is_deleted);
    }
}

#[test]
fn test_replica_convergence_with_mixed_operations() {
    let rga1 = RGA::new(1);
    let rga2 = RGA::new(2);
    let rga3 = RGA::new(3);

    // Each replica does different types of operations
    let start_id = rga1.sentinel_start_id();

    // RGA1: Insert sequence
    let mut last_id = start_id;
    for ch in ['A', 'B', 'C'] {
        last_id = rga1.insert_after(last_id, ch).unwrap();
    }

    // RGA2: Insert different sequence
    let mut last_id = start_id;
    for ch in ['X', 'Y', 'Z'] {
        last_id = rga2.insert_after(last_id, ch).unwrap();
    }

    // RGA3: Insert and delete
    let id1 = rga3.insert_after(start_id, 'M').unwrap();
    let _id2 = rga3.insert_after(id1, 'N').unwrap();
    rga3.delete(id1).unwrap(); // Delete 'M', keep 'N'

    // Collect all operations from all replicas
    let mut all_operations = Vec::new();
    for rga in [&rga1, &rga2, &rga3] {
        for node in rga.all_nodes() {
            if !node.is_sentinel() {
                all_operations.push(node);
            }
        }
    }

    // Apply all operations to all replicas
    for rga in [&rga1, &rga2, &rga3] {
        for op in &all_operations {
            rga.apply_remote_op(op.clone());
        }
    }

    // All replicas should converge to the same state
    let result1 = rga1.to_string();
    let result2 = rga2.to_string();
    let result3 = rga3.to_string();

    assert_eq!(result1, result2);
    assert_eq!(result2, result3);

    // Should contain ABC + XYZ + N (M was deleted)
    assert!(result1.contains('A') && result1.contains('B') && result1.contains('C'));
    assert!(result1.contains('X') && result1.contains('Y') && result1.contains('Z'));
    assert!(result1.contains('N'));
    assert!(!result1.contains('M')); // This was deleted

    assert_eq!(rga1.visible_node_count(), 7); // 6 + 1 (N, M deleted)
}
