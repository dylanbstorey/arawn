---
id: batchembedder-pipeline-concurrent
level: task
title: "BatchEmbedder Pipeline: Concurrent Batch Processing with Rate Limiting"
short_code: "ARAWN-T-0067"
created_at: 2026-01-29T04:43:47.038441+00:00
updated_at: 2026-01-29T04:43:47.038441+00:00
parent: ARAWN-I-0015
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0015
---

# BatchEmbedder Pipeline: Concurrent Batch Processing with Rate Limiting

## Parent Initiative

[[ARAWN-I-0015]]

## Objective

Build a `BatchEmbedder` in `crates/arawn-llm/src/batch.rs` that processes large sets of texts through any `Embedder` with configurable concurrency, chunking, rate limiting, and progress reporting. This is the engine behind `arawn memory reindex`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `BatchEmbedder` struct wraps a `SharedEmbedder` and processes `Vec<(String, String)>` (id, text) pairs
- [ ] Chunks inputs into batches of configurable size (default: 100)
- [ ] Processes chunks with bounded concurrency via `tokio::sync::Semaphore` (default: 4)
- [ ] Token-bucket rate limiter (configurable RPM) with `tokio::time::sleep`
- [ ] Returns `BatchReport` with total, succeeded, failed counts and duration
- [ ] Failed items collected with error messages for retry/reporting
- [ ] Progress callback (`Fn(usize, usize)` — completed, total) for CLI progress bars
- [ ] Unit tests with `MockEmbedder`
- [ ] All existing tests pass

## Implementation Notes

### Technical Approach
- New file: `crates/arawn-llm/src/batch.rs`
- `BatchEmbedder` takes `SharedEmbedder`, `BatchConfig` (chunk_size, concurrency, rate_limit_rpm)
- Use `futures::stream::iter` + `buffer_unordered` for concurrent chunk processing
- Simple rate limiter: track request timestamps, sleep if exceeding RPM
- `BatchReport`: `{ total: usize, succeeded: usize, failed: Vec<(String, String)>, duration: Duration }`
- Export from `crates/arawn-llm/src/lib.rs`

### Dependencies
- T-0066 (GeminiEmbedder) — not blocking, but useful to have all providers available
- `futures` crate for stream concurrency

## Status Updates

*To be added during implementation*