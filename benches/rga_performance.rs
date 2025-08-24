//! Performance benchmarks for the RGA CRDT implementation.
//!
//! This module benchmarks various aspects of the RGA including:
//! - Sequential insertions and deletions
//! - Concurrent operations across multiple replicas
//! - Memory usage patterns
//! - Convergence time under load
//!
//! Run with: cargo bench

use crdt_rga::RGA;
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Benchmark sequential insertions
fn bench_sequential_insertions(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_insertions");

    for size in [100, 500, 1000, 5000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("insert_chars", size), size, |b, &size| {
            b.iter(|| {
                let rga = RGA::new(1);
                let mut last_id = rga.sentinel_start_id();

                for i in 0..size {
                    let ch = (b'A' + (i % 26) as u8) as char;
                    last_id = black_box(rga.insert_after(last_id, ch).unwrap());
                }

                black_box(rga.to_string())
            });
        });
    }
    group.finish();
}

/// Benchmark sequential deletions after insertions
fn bench_sequential_deletions(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_deletions");

    for size in [100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("delete_chars", size), size, |b, &size| {
            b.iter_batched(
                || {
                    // Setup: create RGA with characters
                    let rga = RGA::new(1);
                    let mut ids = Vec::new();
                    let mut last_id = rga.sentinel_start_id();

                    for i in 0..size {
                        let ch = (b'A' + (i % 26) as u8) as char;
                        last_id = rga.insert_after(last_id, ch).unwrap();
                        ids.push(last_id);
                    }
                    (rga, ids)
                },
                |(rga, ids)| {
                    // Benchmark: delete all characters
                    for &id in &ids {
                        black_box(rga.delete(id).unwrap());
                    }
                    black_box(rga.to_string())
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

/// Benchmark concurrent insertions across multiple replicas
fn bench_concurrent_insertions(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_insertions");

    for num_replicas in [2, 4, 8].iter() {
        for ops_per_replica in [100, 500].iter() {
            let total_ops = num_replicas * ops_per_replica;
            group.throughput(Throughput::Elements(total_ops as u64));
            group.bench_with_input(
                BenchmarkId::new(
                    format!("replicas_{}_ops_{}", num_replicas, ops_per_replica),
                    &(num_replicas, ops_per_replica),
                ),
                &(num_replicas, ops_per_replica),
                |b, &(num_replicas, ops_per_replica)| {
                    b.iter(|| {
                        let mut rgas = Vec::new();
                        let mut handles = Vec::new();

                        // Create replicas
                        for replica_id in 0..*num_replicas {
                            rgas.push(Arc::new(RGA::new(replica_id as u64 + 1)));
                        }

                        // Spawn concurrent insertion threads
                        for (replica_id, rga) in rgas.iter().enumerate() {
                            let rga_clone = Arc::clone(rga);
                            let ops = *ops_per_replica;

                            let handle = thread::spawn(move || {
                                let start_id = rga_clone.sentinel_start_id();
                                let mut last_id = start_id;

                                for i in 0..ops {
                                    let ch = (b'A' + ((replica_id * 26 + i) % 26) as u8) as char;
                                    if let Ok(new_id) = rga_clone.insert_after(last_id, ch) {
                                        last_id = new_id;
                                    }
                                }
                            });
                            handles.push(handle);
                        }

                        // Wait for all insertions to complete
                        for handle in handles {
                            handle.join().unwrap();
                        }

                        // Simulate replication (apply all operations to all replicas)
                        let start_replication = Instant::now();
                        for (source_idx, source_rga) in rgas.iter().enumerate() {
                            let nodes = source_rga.all_nodes();
                            for (target_idx, target_rga) in rgas.iter().enumerate() {
                                if source_idx != target_idx {
                                    for node in &nodes {
                                        if !node.is_sentinel() {
                                            target_rga.apply_remote_op(node.clone());
                                        }
                                    }
                                }
                            }
                        }
                        let replication_time = start_replication.elapsed();

                        // Verify convergence
                        let first_content = rgas[0].to_string();
                        for rga in &rgas[1..] {
                            assert_eq!(first_content, rga.to_string(), "Replicas did not converge");
                        }

                        black_box((first_content.len(), replication_time))
                    });
                },
            );
        }
    }
    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    group.bench_function("large_document_creation", |b| {
        b.iter(|| {
            let rga = RGA::new(1);
            let mut last_id = rga.sentinel_start_id();

            // Create a large document (simulate a book chapter)
            let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(100);
            for ch in text.chars() {
                last_id = black_box(rga.insert_after(last_id, ch).unwrap());
            }

            black_box(rga.visible_node_count())
        });
    });

    group.bench_function("heavy_deletion_patterns", |b| {
        b.iter_batched(
            || {
                // Setup: create a large document
                let rga = RGA::new(1);
                let mut ids = Vec::new();
                let mut last_id = rga.sentinel_start_id();
                let text = "The quick brown fox jumps over the lazy dog. ".repeat(50);

                for ch in text.chars() {
                    last_id = rga.insert_after(last_id, ch).unwrap();
                    ids.push(last_id);
                }
                (rga, ids)
            },
            |(rga, ids)| {
                // Delete every other character (simulate heavy editing)
                for (i, &id) in ids.iter().enumerate() {
                    if i % 2 == 0 {
                        black_box(rga.delete(id).unwrap());
                    }
                }
                black_box(rga.to_string())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark conflict resolution under extreme conditions
fn bench_conflict_resolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("conflict_resolution");

    group.bench_function("simultaneous_insertions_same_position", |b| {
        b.iter(|| {
            let num_replicas = 10;
            let mut rgas = Vec::new();
            let mut handles = Vec::new();

            // Create replicas
            for replica_id in 0..num_replicas {
                rgas.push(Arc::new(RGA::new(replica_id + 1)));
            }

            // All replicas insert at the same position simultaneously
            for (replica_id, rga) in rgas.iter().enumerate() {
                let rga_clone = Arc::clone(rga);

                let handle = thread::spawn(move || {
                    let start_id = rga_clone.sentinel_start_id();

                    // Each replica inserts 10 characters at the same position
                    for i in 0..10 {
                        let ch = (b'A' + replica_id as u8) as char;
                        rga_clone.insert_after(start_id, ch).unwrap();
                    }
                });
                handles.push(handle);
            }

            // Wait for all insertions
            for handle in handles {
                handle.join().unwrap();
            }

            // Replicate all operations
            for source_rga in &rgas {
                let nodes = source_rga.all_nodes();
                for target_rga in &rgas {
                    for node in &nodes {
                        if !node.is_sentinel() {
                            target_rga.apply_remote_op(node.clone());
                        }
                    }
                }
            }

            // Verify all replicas converged to the same deterministic order
            let first_content = rgas[0].to_string();
            let first_length = first_content.len();

            for rga in &rgas[1..] {
                let content = rga.to_string();
                assert_eq!(first_content, content, "Conflict resolution failed");
                assert_eq!(first_length, content.len());
            }

            black_box(first_length)
        });
    });

    group.finish();
}

/// Benchmark string conversion performance
fn bench_string_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_conversion");

    for size in [100, 500, 1000, 5000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("to_string", size), size, |b, &size| {
            // Setup: create RGA with mixed content (including deletions)
            let rga = RGA::new(1);
            let mut ids = Vec::new();
            let mut last_id = rga.sentinel_start_id();

            for i in 0..size {
                let ch = (b'A' + (i % 26) as u8) as char;
                last_id = rga.insert_after(last_id, ch).unwrap();
                ids.push(last_id);
            }

            // Delete every 3rd character to create tombstones
            for (i, &id) in ids.iter().enumerate() {
                if i % 3 == 0 {
                    rga.delete(id).unwrap();
                }
            }

            b.iter(|| black_box(rga.to_string()));
        });
    }
    group.finish();
}

/// Benchmark query operations
fn bench_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_operations");

    // Setup: create a moderately sized document
    let rga = RGA::new(1);
    let mut last_id = rga.sentinel_start_id();
    let text = "The quick brown fox jumps over the lazy dog. ".repeat(100);

    for ch in text.chars() {
        last_id = rga.insert_after(last_id, ch).unwrap();
    }

    // Delete some characters to create mixed state
    if let Some(space_id) = rga.find_node_by_char(' ') {
        rga.delete(space_id).unwrap();
    }

    group.bench_function("visible_node_count", |b| {
        b.iter(|| black_box(rga.visible_node_count()));
    });

    group.bench_function("total_node_count", |b| {
        b.iter(|| black_box(rga.total_node_count()));
    });

    group.bench_function("find_node_by_char", |b| {
        b.iter(|| black_box(rga.find_node_by_char('q')));
    });

    group.bench_function("all_nodes", |b| {
        b.iter(|| {
            let nodes = rga.all_nodes();
            black_box(nodes.len())
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_sequential_insertions,
    bench_sequential_deletions,
    bench_concurrent_insertions,
    bench_memory_patterns,
    bench_conflict_resolution,
    bench_string_conversion,
    bench_queries
);

criterion_main!(benches);
