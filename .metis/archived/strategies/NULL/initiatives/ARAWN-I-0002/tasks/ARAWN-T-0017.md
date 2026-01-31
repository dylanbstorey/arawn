---
id: integrate-sqlite-vec-for-vector
level: task
title: "Integrate sqlite-vec for vector storage and search"
short_code: "ARAWN-T-0017"
created_at: 2026-01-28T04:11:25.623939+00:00
updated_at: 2026-01-28T04:34:26.215865+00:00
parent: ARAWN-I-0002
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0002
---

# Integrate sqlite-vec for vector storage and search

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0002]]

## Objective

Integrate sqlite-vec extension for vector similarity search. Store embeddings alongside memories and enable semantic search queries.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Add sqlite-vec dependency to Cargo.toml
- [x] `vector.rs`: VectorStore struct wrapping sqlite-vec operations
- [x] Create vec0 virtual table for memory embeddings (configurable dimensions)
- [x] `store_with_embedding()`: Store memory with vector embedding
- [x] `search_similar()`: Find top-k similar memories by distance
- [x] Support for different embedding dimensions (configurable via `init_vectors(dims)`)
- [x] Unit tests for vector storage and search
- [x] Integration test: store 100 memories, search returns relevant results
- [x] `cargo test -p arawn-memory` passes (27 tests)

## Status Updates

### Session 1 (2026-01-27)
- Added sqlite-vec v0.1.6 and zerocopy v0.8 dependencies
- Created `vector.rs` with:
  - `init_vector_extension()`: Registers sqlite-vec via sqlite3_auto_extension
  - `create_vector_table()`: Creates vec0 virtual table with configurable dimensions
  - `store_embedding()`: Insert/update embedding for memory (delete+insert pattern since vec0 doesn't support REPLACE)
  - `delete_embedding()`: Remove embedding
  - `search_similar()`: KNN search returning MemoryId + distance
  - `search_similar_filtered()`: KNN within a subset of memory IDs
  - `has_embedding()`, `count_embeddings()`: Utility functions
- Integrated vector ops into MemoryStore:
  - `init_vectors(dims)`: Initialize vector table
  - `insert_memory_with_embedding()`: Atomic memory + embedding insert
  - `search_similar_memories()`: Returns full Memory objects with distances
  - Added embedding_count to StoreStats
- All 27 tests passing including integration test with 100 memories