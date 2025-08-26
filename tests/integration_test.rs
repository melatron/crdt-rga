//! Integration tests for the RGA CRDT implementation.
//!
//! These tests verify the correctness of the RGA CRDT across multiple scenarios
//! including basic operations, concurrent editing, and convergence properties.

use crdt_rga::RGA;

#[test]
fn test_basic_rga_operations() {
    let rga = RGA::new(1);
    assert_eq!(rga.to_string(), "");
    assert_eq!(rga.visible_node_count(), 0);

    // Insert characters
    let start_id = rga.sentinel_start_id();
    let a_id = rga.insert_after(start_id, 'A').unwrap();
    let b_id = rga.insert_after(a_id, 'B').unwrap();
    let _c_id = rga.insert_after(b_id, 'C').unwrap();

    assert_eq!(rga.to_string(), "ABC");
    assert_eq!(rga.visible_node_count(), 3);

    // Delete middle character
    rga.delete(b_id).unwrap();
    assert_eq!(rga.to_string(), "AC");
    assert_eq!(rga.visible_node_count(), 2);
    assert_eq!(rga.total_node_count(), 5); // Including tombstones and sentinels
}

#[test]
fn test_concurrent_replicas_convergence() {
    let rga1 = RGA::new(1);
    let rga2 = RGA::new(2);
    let start_id = rga1.sentinel_start_id();

    // Concurrent insertions
    let _x_id = rga1.insert_after(start_id, 'X').unwrap();
    let _y_id = rga2.insert_after(start_id, 'Y').unwrap();

    // Before synchronization, different content
    assert_eq!(rga1.to_string(), "X");
    assert_eq!(rga2.to_string(), "Y");

    // Cross-replicate operations
    let rga1_ops: Vec<_> = rga1
        .all_nodes()
        .into_iter()
        .filter(|n| !n.is_sentinel())
        .collect();
    let rga2_ops: Vec<_> = rga2
        .all_nodes()
        .into_iter()
        .filter(|n| !n.is_sentinel())
        .collect();

    for op in &rga2_ops {
        rga1.apply_remote_op(op.clone());
    }
    for op in &rga1_ops {
        rga2.apply_remote_op(op.clone());
    }

    // After synchronization, both converged
    assert_eq!(rga1.to_string(), rga2.to_string());
    assert!(!rga1.to_string().is_empty());
}

#[test]
fn test_deterministic_ordering() {
    // Test that the same operations always result in the same final order
    for _ in 0..10 {
        let rga1 = RGA::new(1);
        let rga2 = RGA::new(2);
        let start_id = rga1.sentinel_start_id();

        // Different operations from each replica
        rga1.insert_after(start_id, 'X').unwrap();
        rga2.insert_after(start_id, 'Y').unwrap();

        // Sync both ways
        for node in rga1.all_nodes() {
            if !node.is_sentinel() {
                rga2.apply_remote_op(node);
            }
        }
        for node in rga2.all_nodes() {
            if !node.is_sentinel() {
                rga1.apply_remote_op(node);
            }
        }

        // Should always converge to the same result
        let result = rga1.to_string();
        assert_eq!(result, rga2.to_string());
        assert_eq!(result.len(), 2);
        assert!(result.contains('X') && result.contains('Y'));
    }
}

#[test]
fn test_error_handling() {
    let rga = RGA::new(1);

    // Cannot delete non-existent node
    let fake_id = crdt_rga::UniqueId::new(999, 999);
    assert!(rga.delete(fake_id).is_err());

    // Cannot delete sentinel
    let start_id = rga.sentinel_start_id();
    assert!(rga.delete(start_id).is_err());

    // Cannot insert after non-existent node
    assert!(rga.insert_after(fake_id, 'X').is_err());
}

#[test]
fn test_three_way_merge() {
    let rga1 = RGA::new(1);
    let rga2 = RGA::new(2);
    let rga3 = RGA::new(3);
    let start_id = rga1.sentinel_start_id();

    // Each replica makes different changes
    rga1.insert_after(start_id, '1').unwrap();
    rga2.insert_after(start_id, '2').unwrap();
    rga3.insert_after(start_id, '3').unwrap();

    // Collect all operations
    let mut all_ops = Vec::new();
    for rga in [&rga1, &rga2, &rga3] {
        for node in rga.all_nodes() {
            if !node.is_sentinel() {
                all_ops.push(node);
            }
        }
    }

    // Apply all operations to all replicas
    for rga in [&rga1, &rga2, &rga3] {
        for op in &all_ops {
            rga.apply_remote_op(op.clone());
        }
    }

    // All should converge to same state
    let result1 = rga1.to_string();
    let result2 = rga2.to_string();
    let result3 = rga3.to_string();

    assert_eq!(result1, result2);
    assert_eq!(result2, result3);
    assert_eq!(result1.len(), 3);
}
