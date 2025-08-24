//! Advanced concurrent editing example demonstrating high-performance RGA CRDT.
//!
//! This example showcases:
//! - Concurrent operations across multiple replicas with lock-free data structures
//! - Advanced conflict resolution with sequence numbers
//! - Performance monitoring and metrics
//! - Realistic collaborative editing scenarios
//!
//! Run with: cargo run --example concurrent_editing

use crdt_rga::RGA;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    println!("=== Advanced Concurrent RGA CRDT Example ===\n");

    // Run different scenarios
    basic_concurrent_demo();
    println!();

    stress_test_demo();
    println!();

    performance_comparison_demo();
    println!();

    conflict_resolution_demo();
}

/// Demonstrates basic concurrent operations with multiple threads
fn basic_concurrent_demo() {
    println!("--- Basic Concurrent Operations ---");

    let num_threads = 4;
    let operations_per_thread = 100;
    let rga = Arc::new(RGA::new(1));
    let mut handles = Vec::new();

    println!(
        "Starting {} threads, each performing {} operations",
        num_threads, operations_per_thread
    );

    let start_time = Instant::now();

    // Spawn concurrent threads
    for thread_id in 0..num_threads {
        let rga_clone = Arc::clone(&rga);

        let handle = thread::spawn(move || {
            let thread_start = Instant::now();
            let mut operations_completed = 0;
            let mut last_id = rga_clone.sentinel_start_id();

            for i in 0..operations_per_thread {
                let ch = (b'A' + (thread_id * 4 + i % 26) as u8) as char;

                match rga_clone.insert_after(last_id, ch) {
                    Ok(new_id) => {
                        last_id = new_id;
                        operations_completed += 1;
                    }
                    Err(_) => {
                        // Fallback to start if reference becomes invalid
                        last_id = rga_clone.sentinel_start_id();
                    }
                }

                // Occasionally delete some characters
                if i % 10 == 0 && i > 0 {
                    if let Some(char_to_delete) = rga_clone.find_node_by_char(ch) {
                        let _ = rga_clone.delete(char_to_delete);
                    }
                }
            }

            println!(
                "Thread {} completed {} operations in {:?}",
                thread_id,
                operations_completed,
                thread_start.elapsed()
            );
            operations_completed
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    let mut total_operations = 0;
    for handle in handles {
        total_operations += handle.join().unwrap();
    }

    let total_time = start_time.elapsed();

    println!("All threads completed!");
    println!("Total operations: {}", total_operations);
    println!("Total time: {:?}", total_time);
    println!(
        "Operations per second: {:.2}",
        total_operations as f64 / total_time.as_secs_f64()
    );
    println!("Final document length: {}", rga.to_string().len());
    println!(
        "Total nodes (including tombstones): {}",
        rga.total_node_count()
    );
    println!("Visible nodes: {}", rga.visible_node_count());
}

/// Stress test with many replicas and operations
fn stress_test_demo() {
    println!("--- Stress Test: Multiple Replicas ---");

    let num_replicas = 8;
    let operations_per_replica = 200;
    let mut rgas = Vec::new();
    let mut handles = Vec::new();

    println!(
        "Creating {} replicas, each performing {} operations",
        num_replicas, operations_per_replica
    );

    // Create replicas
    for replica_id in 0..num_replicas {
        rgas.push(Arc::new(RGA::new(replica_id + 1)));
    }

    let start_time = Instant::now();
    let total_ops = Arc::new(AtomicUsize::new(0));

    // Spawn threads for each replica
    for (replica_idx, rga) in rgas.iter().enumerate() {
        let rga_clone = Arc::clone(rga);
        let ops_counter = Arc::clone(&total_ops);

        let handle = thread::spawn(move || {
            let mut local_ops = 0;
            let start_id = rga_clone.sentinel_start_id();
            let mut last_id = start_id;

            // Perform insertions
            for i in 0..operations_per_replica {
                let ch = match i % 5 {
                    0 => (b'A' + replica_idx as u8) as char,
                    1 => (b'a' + replica_idx as u8) as char,
                    2 => (b'0' + replica_idx as u8) as char,
                    3 => ' ',
                    _ => '.',
                };

                if let Ok(new_id) = rga_clone.insert_after(last_id, ch) {
                    last_id = new_id;
                    local_ops += 1;

                    // Occasionally insert after the start instead
                    if i % 20 == 0 {
                        last_id = start_id;
                    }
                }

                // Random deletions
                if i % 15 == 0 {
                    if let Some(delete_id) = rga_clone.find_node_by_char(ch) {
                        let _ = rga_clone.delete(delete_id);
                    }
                }
            }

            ops_counter.fetch_add(local_ops, Ordering::Relaxed);
            local_ops
        });

        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.join().unwrap();
    }

    let operation_time = start_time.elapsed();
    let total_operations = total_ops.load(Ordering::Relaxed);

    println!(
        "Phase 1 - Concurrent Operations completed in {:?}",
        operation_time
    );
    println!("Total operations: {}", total_operations);

    // Phase 2: Simulate network synchronization
    println!("\nPhase 2 - Network Synchronization...");
    let sync_start = Instant::now();

    // Collect all operations from all replicas
    let mut all_operations = Vec::new();
    for rga in &rgas {
        let nodes = rga.all_nodes();
        for node in nodes {
            if !node.is_sentinel() {
                all_operations.push(node);
            }
        }
    }

    println!("Collected {} total operations", all_operations.len());

    // Apply all operations to all replicas (simulating full mesh replication)
    for rga in &rgas {
        for operation in &all_operations {
            rga.apply_remote_op(operation.clone());
        }
    }

    let sync_time = sync_start.elapsed();

    // Verify convergence
    let reference_content = rgas[0].to_string();
    let mut all_converged = true;

    for (i, rga) in rgas.iter().enumerate() {
        let content = rga.to_string();
        if content != reference_content {
            println!("ERROR: Replica {} did not converge!", i);
            all_converged = false;
        }
    }

    if all_converged {
        println!("✓ All {} replicas successfully converged!", num_replicas);
    }

    println!("Synchronization completed in {:?}", sync_time);
    println!("Final document length: {}", reference_content.len());
    println!(
        "Total nodes across all replicas: {}",
        rgas[0].total_node_count()
    );
    println!(
        "Operations per second: {:.2}",
        total_operations as f64 / operation_time.as_secs_f64()
    );
    println!(
        "Sync throughput: {:.2} ops/sec",
        all_operations.len() as f64 / sync_time.as_secs_f64()
    );
}

/// Compare performance with and without concurrent access
fn performance_comparison_demo() {
    println!("--- Performance Comparison ---");

    let operations = 1000;

    // Sequential operations
    println!("Testing sequential operations...");
    let sequential_start = Instant::now();
    let rga_seq = RGA::new(1);
    let mut last_id = rga_seq.sentinel_start_id();

    for i in 0..operations {
        let ch = (b'A' + (i % 26) as u8) as char;
        last_id = rga_seq.insert_after(last_id, ch).unwrap();
    }

    let sequential_time = sequential_start.elapsed();
    println!(
        "Sequential: {} ops in {:?} ({:.2} ops/sec)",
        operations,
        sequential_time,
        operations as f64 / sequential_time.as_secs_f64()
    );

    // Concurrent operations
    println!("Testing concurrent operations...");
    let concurrent_start = Instant::now();
    let rga_conc = Arc::new(RGA::new(1));
    let threads = 4;
    let ops_per_thread = operations / threads;
    let mut handles = Vec::new();

    for thread_id in 0..threads {
        let rga_clone = Arc::clone(&rga_conc);

        let handle = thread::spawn(move || {
            let start_id = rga_clone.sentinel_start_id();
            let mut last_id = start_id;

            for i in 0..ops_per_thread {
                let ch = (b'A' + ((thread_id * ops_per_thread + i) % 26) as u8) as char;
                if let Ok(new_id) = rga_clone.insert_after(last_id, ch) {
                    last_id = new_id;
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let concurrent_time = concurrent_start.elapsed();
    println!(
        "Concurrent: {} ops in {:?} ({:.2} ops/sec)",
        operations,
        concurrent_time,
        operations as f64 / concurrent_time.as_secs_f64()
    );

    let speedup = sequential_time.as_secs_f64() / concurrent_time.as_secs_f64();
    println!("Concurrent speedup: {:.2}x", speedup);

    println!("Sequential result length: {}", rga_seq.to_string().len());
    println!("Concurrent result length: {}", rga_conc.to_string().len());
}

/// Demonstrate advanced conflict resolution with sequence numbers
fn conflict_resolution_demo() {
    println!("--- Advanced Conflict Resolution ---");

    let num_replicas = 6;
    let mut rgas = Vec::new();

    // Create replicas
    for replica_id in 0..num_replicas {
        rgas.push(Arc::new(RGA::new(replica_id + 1)));
    }

    println!("Testing conflict resolution with {} replicas", num_replicas);

    // Scenario: All replicas insert at the same position simultaneously
    println!("All replicas inserting at the same position...");
    let mut handles = Vec::new();

    for (replica_idx, rga) in rgas.iter().enumerate() {
        let rga_clone = Arc::clone(rga);

        let handle = thread::spawn(move || {
            let start_id = rga_clone.sentinel_start_id();

            // Each replica inserts multiple characters rapidly
            for i in 0..50 {
                let ch = (b'A' + replica_idx as u8) as char;
                let _ = rga_clone.insert_after(start_id, ch);

                // Small delay to create interleaving
                if i % 10 == 0 {
                    thread::sleep(Duration::from_nanos(100));
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all concurrent insertions
    for handle in handles {
        handle.join().unwrap();
    }

    println!("Concurrent insertions completed. Starting replication...");

    // Replicate all operations to demonstrate deterministic ordering
    for source_rga in &rgas {
        let operations = source_rga.all_nodes();
        for target_rga in &rgas {
            for operation in &operations {
                if !operation.is_sentinel() {
                    target_rga.apply_remote_op(operation.clone());
                }
            }
        }
    }

    // Verify deterministic conflict resolution
    let reference_content = rgas[0].to_string();
    let reference_nodes = rgas[0].all_nodes();

    println!("Verifying deterministic conflict resolution...");
    let mut all_identical = true;

    for (i, rga) in rgas.iter().enumerate() {
        let content = rga.to_string();
        let nodes = rga.all_nodes();

        if content != reference_content || nodes.len() != reference_nodes.len() {
            println!("ERROR: Replica {} has different state!", i);
            println!("  Content: '{}'", content);
            println!("  Nodes: {}", nodes.len());
            all_identical = false;
        }
    }

    if all_identical {
        println!("✓ Perfect conflict resolution! All replicas have identical state.");
        println!("✓ Final content length: {}", reference_content.len());
        println!("✓ Total nodes: {}", reference_nodes.len());

        // Show the deterministic ordering
        println!("\nDeterministic ordering sample:");
        let sample_chars: String = reference_content.chars().take(20).collect();
        println!("First 20 characters: '{}'", sample_chars);

        // Analyze the ordering pattern
        let mut char_counts = std::collections::HashMap::new();
        for ch in reference_content.chars() {
            *char_counts.entry(ch).or_insert(0) += 1;
        }

        println!("Character distribution:");
        for (ch, count) in &char_counts {
            println!("  '{}': {} occurrences", ch, count);
        }
    } else {
        println!("✗ Conflict resolution failed - replicas have different states");
    }

    // Show detailed node information for the first replica
    println!("\nDetailed node structure (first 10 nodes):");
    rgas[0].dump_nodes();
}
