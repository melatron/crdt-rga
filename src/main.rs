//! Example usage and demonstrations of the RGA CRDT.
//!
//! This file showcases various features of the RGA implementation including:
//! - Basic insertion and deletion
//! - Concurrent operations across replicas
//! - Replication and convergence
//! - Error handling

use crdt_rga::{RGA, UniqueId};

fn main() {
    println!("=== RGA CRDT Demonstration ===\n");

    basic_operations_demo();
    println!();

    concurrent_operations_demo();
    println!();

    deletion_demo();
    println!();

    complex_scenario_demo();
}

/// Demonstrates basic insertion operations on a single replica.
fn basic_operations_demo() {
    println!("--- Basic Operations Demo ---");

    let rga = RGA::new(1);
    println!("Created RGA with replica ID 1");
    println!("Initial content: '{}'", rga.to_string());

    // Insert characters one by one
    let start_id = rga.sentinel_start_id();
    let h_id = rga.insert_after(start_id, 'H').unwrap();
    println!("After inserting 'H': '{}'", rga.to_string());

    let e_id = rga.insert_after(h_id, 'e').unwrap();
    println!("After inserting 'e': '{}'", rga.to_string());

    let l_id = rga.insert_after(e_id, 'l').unwrap();
    println!("After inserting 'l': '{}'", rga.to_string());

    let l2_id = rga.insert_after(l_id, 'l').unwrap();
    println!("After inserting 'l': '{}'", rga.to_string());

    rga.insert_after(l2_id, 'o').unwrap();
    println!("After inserting 'o': '{}'", rga.to_string());

    println!("Final content: '{}'", rga.to_string());
    println!(
        "Total nodes: {}, Visible nodes: {}",
        rga.total_node_count(),
        rga.visible_node_count()
    );
}

/// Demonstrates concurrent operations across multiple replicas.
fn concurrent_operations_demo() {
    println!("--- Concurrent Operations Demo ---");

    // Create two replicas
    let rga1 = RGA::new(1);
    let rga2 = RGA::new(2);

    println!("Created RGA replicas 1 and 2");

    // Both replicas start by inserting after the start sentinel
    let start_id = rga1.sentinel_start_id();

    // Replica 1 inserts "Hello"
    let h_id = rga1.insert_after(start_id, 'H').unwrap();
    let e_id = rga1.insert_after(h_id, 'e').unwrap();
    println!("RGA 1 after inserting 'He': '{}'", rga1.to_string());

    // Replica 2 concurrently inserts "World"
    let w_id = rga2.insert_after(start_id, 'W').unwrap();
    let o_id = rga2.insert_after(w_id, 'o').unwrap();
    println!("RGA 2 after inserting 'Wo': '{}'", rga2.to_string());

    // Now simulate replication - each replica receives the other's operations
    println!("\n--- Simulating Replication ---");

    // Send RGA 1's operations to RGA 2
    for node in rga1.all_nodes() {
        if !node.is_sentinel() {
            rga2.apply_remote_op(node);
        }
    }
    println!(
        "RGA 2 after receiving RGA 1's updates: '{}'",
        rga2.to_string()
    );

    // Send RGA 2's operations to RGA 1
    for node in rga2.all_nodes() {
        if !node.is_sentinel() && rga1.find_node_by_char(node.character).is_none() {
            rga1.apply_remote_op(node);
        }
    }
    println!(
        "RGA 1 after receiving RGA 2's updates: '{}'",
        rga1.to_string()
    );

    // Verify convergence
    assert_eq!(rga1.to_string(), rga2.to_string());
    println!("✓ Both replicas converged to: '{}'", rga1.to_string());

    // Show the ordering is deterministic based on UniqueId (Lamport timestamps)
    println!("\nNode ordering details:");
    rga1.dump_nodes();
}

/// Demonstrates deletion operations and tombstones.
fn deletion_demo() {
    println!("--- Deletion Demo ---");

    let rga = RGA::new(1);
    let start_id = rga.sentinel_start_id();

    // Insert "ABCD"
    let a_id = rga.insert_after(start_id, 'A').unwrap();
    let b_id = rga.insert_after(a_id, 'B').unwrap();
    let c_id = rga.insert_after(b_id, 'C').unwrap();
    let d_id = rga.insert_after(c_id, 'D').unwrap();

    println!("Initial content: '{}'", rga.to_string());
    println!(
        "Total nodes: {}, Visible nodes: {}",
        rga.total_node_count(),
        rga.visible_node_count()
    );

    // Delete 'B'
    rga.delete(b_id).unwrap();
    println!("After deleting 'B': '{}'", rga.to_string());
    println!(
        "Total nodes: {}, Visible nodes: {}",
        rga.total_node_count(),
        rga.visible_node_count()
    );

    // Delete 'C'
    rga.delete(c_id).unwrap();
    println!("After deleting 'C': '{}'", rga.to_string());
    println!(
        "Total nodes: {}, Visible nodes: {}",
        rga.total_node_count(),
        rga.visible_node_count()
    );

    // Show that tombstones are preserved
    println!("\nAll nodes (including tombstones):");
    rga.dump_nodes();

    // Test error cases
    println!("\n--- Testing Error Cases ---");

    // Try to delete a non-existent node
    let fake_id = UniqueId::new(999, 999);
    match rga.delete(fake_id) {
        Ok(()) => println!("ERROR: Should not have been able to delete non-existent node"),
        Err(msg) => println!("✓ Expected error when deleting non-existent node: {}", msg),
    }

    // Try to delete a sentinel
    match rga.delete(start_id) {
        Ok(()) => println!("ERROR: Should not have been able to delete sentinel"),
        Err(msg) => println!("✓ Expected error when deleting sentinel: {}", msg),
    }
}

/// Demonstrates a more complex scenario with multiple replicas and mixed operations.
fn complex_scenario_demo() {
    println!("--- Complex Scenario Demo ---");

    // Simulate a collaborative editing session with 3 replicas
    let rga1 = RGA::new(1);
    let rga2 = RGA::new(2);
    let rga3 = RGA::new(3);

    let start_id = rga1.sentinel_start_id();

    println!("Scenario: Three users collaboratively editing a document");

    // User 1 types "Hello"
    println!("\nUser 1 types 'Hello':");
    let mut last_id = start_id;
    for ch in "Hello".chars() {
        last_id = rga1.insert_after(last_id, ch).unwrap();
    }
    println!("RGA 1: '{}'", rga1.to_string());

    // User 2 types " World" (they also start from the beginning, creating conflict)
    println!("\nUser 2 types ' World' (concurrent with User 1):");
    let mut last_id2 = start_id;
    for ch in " World".chars() {
        last_id2 = rga2.insert_after(last_id2, ch).unwrap();
    }
    println!("RGA 2: '{}'", rga2.to_string());

    // User 3 inserts "!" at the end (but doesn't know about others yet)
    println!("\nUser 3 adds '!' (concurrent with others):");
    rga3.insert_after(start_id, '!').unwrap();
    println!("RGA 3: '{}'", rga3.to_string());

    // Simulate network replication - everyone receives everyone else's operations
    println!("\n--- Network Synchronization ---");

    // Collect all operations
    let mut all_operations = Vec::new();
    for rga in [&rga1, &rga2, &rga3] {
        for node in rga.all_nodes() {
            if !node.is_sentinel() {
                all_operations.push(node);
            }
        }
    }

    // Apply all operations to all replicas
    for op in &all_operations {
        rga1.apply_remote_op(op.clone());
        rga2.apply_remote_op(op.clone());
        rga3.apply_remote_op(op.clone());
    }

    // Show final convergence
    println!("After synchronization:");
    println!("RGA 1: '{}'", rga1.to_string());
    println!("RGA 2: '{}'", rga2.to_string());
    println!("RGA 3: '{}'", rga3.to_string());

    // Verify all replicas converged
    assert_eq!(rga1.to_string(), rga2.to_string());
    assert_eq!(rga2.to_string(), rga3.to_string());

    println!("✓ All replicas converged!");

    // Show the final ordering
    println!("\nFinal document structure:");
    rga1.dump_nodes();

    // Demonstrate deletion in collaborative scenario
    println!("\n--- Collaborative Deletion ---");

    // User 1 deletes some characters
    if let Some(space_id) = rga1.find_node_by_char(' ') {
        rga1.delete(space_id).unwrap();
        println!("User 1 deleted a space");
    }

    // Replicate deletion
    for node in rga1.all_nodes() {
        rga2.apply_remote_op(node.clone());
        rga3.apply_remote_op(node.clone());
    }

    println!("After deletion synchronization:");
    println!("Final content: '{}'", rga1.to_string());

    // Verify consistency
    assert_eq!(rga1.to_string(), rga2.to_string());
    assert_eq!(rga2.to_string(), rga3.to_string());
    println!("✓ All replicas remain consistent after deletion!");
}
