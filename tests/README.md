# RGA CRDT Test Suite

This directory contains comprehensive tests for the RGA (Replicated Growable Array) CRDT implementation. The test suite is organized into multiple levels to ensure robustness and reliability of the distributed data structure.

## Test Organization

### Unit Tests (Embedded in Source)
Located within the source code modules in `src/crdt/`, these tests verify individual components:

#### `src/crdt/node.rs` - Node Tests (6 tests)
- ✅ `test_node_creation` - Basic node construction
- ✅ `test_node_deletion` - Node deletion and tombstone behavior
- ✅ `test_sentinel_nodes` - Sentinel node properties and immutability
- ✅ `test_node_visibility` - Visibility rules for deleted/sentinel nodes
- ✅ `test_node_ordering` - Node ordering by UniqueId

#### `src/crdt/rga.rs` - Core RGA Tests (5 tests)
- ✅ `test_rga_creation` - RGA initialization with sentinels
- ✅ `test_basic_insertion` - Sequential character insertion
- ✅ `test_deletion` - Character deletion with tombstones
- ✅ `test_remote_operations` - Remote operation application
- ✅ `test_concurrent_operations` - Concurrent insertions and convergence

#### `src/crdt/types/` - Type System Tests (10 tests)
**Clock Tests (4 tests):**
- ✅ `test_lamport_clock` - Basic clock operations
- ✅ `test_lamport_clock_update` - Clock synchronization
- ✅ `test_clock_sequence_numbering` - Sequence number generation
- ✅ `test_clock_replica_id` - Replica ID handling

**Timestamp Tests (2 tests):**
- ✅ `test_lamport_timestamp_ordering` - Timestamp comparison
- ✅ `test_sequence_ordering` - Sequence-based ordering

**UniqueId Tests (4 tests):**
- ✅ `test_unique_id_creation` - ID construction
- ✅ `test_unique_id_with_sequence` - Sequence number support
- ✅ `test_unique_id_ordering` - ID comparison and ordering
- ✅ `test_conversion_between_types` - Type conversions

### Integration Tests (5 tests)
**File:** `tests/integration_test.rs`

These tests verify end-to-end functionality and CRDT properties:

- ✅ `test_basic_rga_operations` - Complete workflow (insert, delete, convergence)
- ✅ `test_concurrent_replicas_convergence` - Two-replica synchronization
- ✅ `test_deterministic_ordering` - Consistent ordering across runs
- ✅ `test_error_handling` - Invalid operation handling
- ✅ `test_three_way_merge` - Multi-replica convergence

### Edge Cases Tests (14 tests)
**File:** `tests/edge_cases_test.rs`

Comprehensive stress testing and boundary condition verification:

**Security & Robustness (3 tests):**
- ✅ `test_sentinel_deletion_protection` - Sentinel immutability
- ✅ `test_invalid_operations` - Error handling for bad operations
- ✅ `test_concurrent_deletion_same_node` - Race condition handling

**Scale & Performance (3 tests):**
- ✅ `test_large_document_operations` - 10,000 character documents
- ✅ `test_rapid_operations_stress` - High-frequency operations
- ✅ `test_memory_efficiency_with_many_deletes` - Tombstone management

**Data Integrity (4 tests):**
- ✅ `test_unicode_edge_cases` - Full Unicode support (emojis, symbols)
- ✅ `test_null_and_control_characters` - Control character handling
- ✅ `test_extreme_replica_ids` - Boundary replica IDs (0, u64::MAX)
- ✅ `test_clock_progression` - Clock advancement verification

**Edge Scenarios (4 tests):**
- ✅ `test_empty_document_operations` - Operations on empty RGA
- ✅ `test_single_character_document` - Minimal document handling
- ✅ `test_find_node_by_char_edge_cases` - Search edge cases
- ✅ `test_replica_convergence_with_mixed_operations` - Complex synchronization

## Test Statistics

| Test Category | Count | Status |
|---------------|-------|--------|
| Unit Tests | 21 | ✅ All Pass |
| Integration Tests | 5 | ✅ All Pass |
| Edge Cases Tests | 14 | ✅ All Pass |
| Doc Tests | 1 | ✅ All Pass |
| **Total** | **41** | ✅ **All Pass** |

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Categories
```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration_test

# Edge cases only
cargo test --test edge_cases_test

# Doc tests only
cargo test --doc
```

### Run with Verbose Output
```bash
cargo test -- --nocapture
```

### Run Tests with Logging
```bash
RUST_LOG=debug cargo test
```

## Test Coverage Areas

### ✅ CRDT Properties Verified
- **Convergence**: All replicas reach identical final state
- **Commutativity**: Operation order doesn't affect final result  
- **Associativity**: Grouping of operations doesn't matter
- **Idempotency**: Duplicate operations are handled correctly

### ✅ Concurrency Scenarios
- Concurrent insertions at same position
- Simultaneous deletions of same node
- Multi-replica synchronization (2-way, 3-way)
- Race conditions and conflict resolution

### ✅ Error Conditions
- Invalid node references
- Attempts to delete sentinels
- Operations on non-existent nodes
- Malformed operations

### ✅ Performance & Scale
- Large documents (10K+ characters)
- High-frequency operations (1K+ ops/sec)
- Memory efficiency with many tombstones
- Unicode and special character handling

### ✅ Boundary Conditions
- Empty documents
- Single character documents
- Extreme replica IDs (0, u64::MAX)
- Control characters and null bytes

## Contributing New Tests

When adding new tests, consider:

1. **Test Naming**: Use descriptive names that explain what is being tested
2. **Test Categories**: Place tests in appropriate files based on scope
3. **Edge Cases**: Consider boundary conditions and error scenarios  
4. **Performance**: Include tests for operations at scale
5. **Documentation**: Update this README with new test descriptions

## Test Dependencies

The test suite uses:
- `cargo test` - Rust's built-in test framework
- No external testing dependencies required
- Tests run in parallel by default
- Deterministic results across multiple runs