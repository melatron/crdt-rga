//! Simple standalone example of RGA CRDT usage.
//!
//! This example demonstrates the basic functionality of the RGA CRDT
//! in a simple, easy-to-understand scenario.
//!
//! Run with: cargo run --example simple

use crdt_rga::RGA;

fn main() {
    println!("=== Simple RGA CRDT Example ===\n");

    // Create two replicas representing two users
    let alice = RGA::new(1);
    let bob = RGA::new(2);

    println!("Alice (replica 1) and Bob (replica 2) start editing a document\n");

    // Alice types "Hello"
    println!("Alice types 'Hello':");
    let start_id = alice.sentinel_start_id();
    let mut last_alice_id = start_id;

    for ch in "Hello".chars() {
        last_alice_id = alice.insert_after(last_alice_id, ch).unwrap();
    }
    println!("  Alice's document: '{}'", alice.to_string());

    // Bob concurrently types "World!" starting from the beginning
    println!("\nBob concurrently types 'World!' (also from the start):");
    let mut last_bob_id = start_id;

    for ch in "World!".chars() {
        last_bob_id = bob.insert_after(last_bob_id, ch).unwrap();
    }
    println!("  Bob's document: '{}'", bob.to_string());

    println!("\n--- Before Synchronization ---");
    println!("  Alice sees: '{}'", alice.to_string());
    println!("  Bob sees:   '{}'", bob.to_string());

    // Synchronize: Alice receives Bob's changes
    println!("\n--- Synchronizing Changes ---");
    println!("Alice receives Bob's changes...");

    for node in bob.all_nodes() {
        if !node.is_sentinel() {
            alice.apply_remote_op(node);
        }
    }

    // Bob receives Alice's changes
    println!("Bob receives Alice's changes...");

    for node in alice.all_nodes() {
        if !node.is_sentinel() {
            bob.apply_remote_op(node);
        }
    }

    println!("\n--- After Synchronization ---");
    println!("  Alice sees: '{}'", alice.to_string());
    println!("  Bob sees:   '{}'", bob.to_string());

    // Verify convergence
    if alice.to_string() == bob.to_string() {
        println!("\n✓ SUCCESS: Both users converged to the same document!");
        println!("✓ Final content: '{}'", alice.to_string());
    } else {
        println!("\n✗ ERROR: Documents did not converge!");
    }

    // Show the ordering details
    println!("\n--- Technical Details ---");
    println!("The final ordering is determined by Lamport timestamps:");
    alice.dump_nodes();

    // Demonstrate deletion
    println!("\n=== Deletion Example ===");

    // Alice deletes the 'W' character
    if let Some(w_id) = alice.find_node_by_char('W') {
        println!("Alice deletes 'W'");
        alice.delete(w_id).unwrap();
        println!("  Alice's document: '{}'", alice.to_string());

        // Synchronize the deletion
        println!("Synchronizing deletion to Bob...");
        for node in alice.all_nodes() {
            bob.apply_remote_op(node);
        }

        println!("  Bob's document: '{}'", bob.to_string());

        if alice.to_string() == bob.to_string() {
            println!("✓ Deletion synchronized successfully!");
        }
    }

    println!("\n=== Example Complete ===");
    println!("This demonstrates how RGA CRDT ensures eventual consistency");
    println!("in collaborative editing scenarios without conflicts!");
}
