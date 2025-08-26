//! Realistic collaborative editing example using RGA CRDT.
//!
//! This example simulates two users (Alice and Bob) collaboratively editing
//! a document in real-time. It demonstrates:
//! - Realistic typing patterns with delays
//! - Network synchronization simulation
//! - Conflict resolution in concurrent scenarios
//! - Real-world usage patterns for collaborative editors
//!
//! Run with: cargo run --example simple

use crdt_rga::RGA;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

struct CollaborativeSession {
    alice: RGA,
    bob: RGA,
    network_delay: Duration,
}

impl CollaborativeSession {
    fn new() -> Self {
        Self {
            alice: RGA::new(1), // Alice = replica 1
            bob: RGA::new(2),   // Bob = replica 2
            network_delay: Duration::from_millis(50),
        }
    }

    fn simulate_typing(&self, user: &str, text: &str, typing_speed: Duration) {
        print!("{} types: ", user);
        for ch in text.chars() {
            print!("{}", ch);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            thread::sleep(typing_speed);
        }
        println!();
    }

    fn sync_changes(&mut self) {
        // Simulate network synchronization
        thread::sleep(self.network_delay);

        // Alice's changes -> Bob
        for node in self.alice.all_nodes() {
            if !node.is_sentinel() {
                self.bob.apply_remote_op(node);
            }
        }

        // Bob's changes -> Alice
        for node in self.bob.all_nodes() {
            if !node.is_sentinel() {
                self.alice.apply_remote_op(node);
            }
        }
    }

    fn show_status(&self) {
        println!("  Alice sees: '{}'", self.alice.to_string());
        println!("  Bob sees:   '{}'", self.bob.to_string());

        if self.alice.to_string() == self.bob.to_string() {
            println!("  ✅ Synchronized!");
        } else {
            println!("  ⏳ Synchronizing...");
        }
        println!();
    }
}

fn main() {
    println!("=== Realistic Collaborative Text Editor with RGA CRDT ===");
    println!("🎯 Simulating Google Docs-style collaborative editing\n");

    let mut session = CollaborativeSession::new();

    // === Scenario 1: Turn-based editing (ideal case) ===
    println!("📝 Scenario 1: Turn-based Editing");
    println!("Users take turns, each building on the other's work.\n");

    // Alice starts the document
    println!("👩 Alice starts writing...");
    let start_id = session.alice.sentinel_start_id();
    let mut last_id = start_id;

    let alice_text = "Hello";
    session.simulate_typing("Alice", alice_text, Duration::from_millis(200));

    for ch in alice_text.chars() {
        last_id = session.alice.insert_after(last_id, ch).unwrap();
    }

    session.show_status();

    // Sync Alice's work to Bob
    println!("🌐 Network sync: Alice's changes → Bob");
    session.sync_changes();
    session.show_status();

    // Bob continues where Alice left off
    println!("👨 Bob continues the sentence...");
    let alice_last_char = session
        .bob
        .all_nodes()
        .into_iter()
        .filter(|n| !n.is_sentinel() && !n.is_deleted)
        .max_by_key(|n| n.id)
        .unwrap();

    let mut bob_last_id = alice_last_char.id;
    let bob_text = " World!";

    session.simulate_typing("Bob", bob_text, Duration::from_millis(180));

    for ch in bob_text.chars() {
        bob_last_id = session.bob.insert_after(bob_last_id, ch).unwrap();
    }

    session.show_status();

    // Final sync
    println!("🌐 Network sync: Bob's changes → Alice");
    session.sync_changes();
    session.show_status();

    // === Scenario 2: Concurrent editing (real-world case) ===
    println!("📝 Scenario 2: Concurrent Editing");
    println!("Both users type simultaneously - this is where CRDT shines!\n");

    // Reset for new scenario - simulate concurrent editing
    let mut concurrent_session = CollaborativeSession::new();

    println!("👩👨 Both users start typing at the same time...");

    // Simulate Alice typing "Fast" concurrently with Bob typing "Code"
    let start_id = concurrent_session.alice.sentinel_start_id();

    // Alice types "Fast" from beginning
    println!("🚀 Alice types from start:");
    let mut alice_last = start_id;
    for ch in "Fast".chars() {
        alice_last = concurrent_session
            .alice
            .insert_after(alice_last, ch)
            .unwrap();
        println!("  Alice typed: '{}'", ch);
        thread::sleep(Duration::from_millis(50));
    }

    // Bob types "Code" also from beginning (concurrent conflict!)
    println!("🚀 Bob types from start (concurrent with Alice):");
    let mut bob_last = start_id;
    for ch in "Code".chars() {
        bob_last = concurrent_session.bob.insert_after(bob_last, ch).unwrap();
        println!("  Bob typed: '{}'", ch);
        thread::sleep(Duration::from_millis(45));
    }

    println!("\nBefore synchronization:");
    println!("  Alice sees: '{}'", concurrent_session.alice.to_string());
    println!("  Bob sees:   '{}'", concurrent_session.bob.to_string());

    // Apply all operations to both replicas (simulating full sync)
    println!("\n🌐 Network synchronization...");
    concurrent_session.sync_changes();

    println!("\n📊 Final Results:");
    println!("  Alice's view: '{}'", concurrent_session.alice.to_string());
    println!("  Bob's view:   '{}'", concurrent_session.bob.to_string());

    if concurrent_session.alice.to_string() == concurrent_session.bob.to_string() {
        println!("  ✅ Perfect convergence despite concurrent edits!");
        println!("  🎯 Notice how characters are interleaved based on Lamport timestamps!");
    }

    // Show the technical details
    println!("\n🔍 Technical Analysis:");
    println!("The final order is determined by Lamport timestamps:");
    println!("(Lower counter wins, replica_id breaks ties)");
    concurrent_session.alice.dump_nodes();

    println!("\n🎯 Real-World Applications:");
    println!("• Google Docs, Notion, Figma - collaborative editors");
    println!("• Code editors like VS Code Live Share");
    println!("• Chat applications with message ordering");
    println!("• Collaborative whiteboards and design tools");
    println!("• Git-like version control systems");

    println!("\n💡 Why RGA CRDT is Perfect for This:");
    println!("• No central server coordination needed");
    println!("• Users never get blocked waiting for others");
    println!("• Deterministic conflict resolution");
    println!("• Works offline and syncs when reconnected");
    println!("• Scales to many concurrent users");
}
