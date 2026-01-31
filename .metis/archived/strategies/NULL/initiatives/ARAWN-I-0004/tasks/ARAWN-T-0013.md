---
id: add-streaming-response-support-to
level: task
title: "Add streaming response support to Agent"
short_code: "ARAWN-T-0013"
created_at: 2026-01-28T03:20:08.870502+00:00
updated_at: 2026-01-28T03:50:15.826831+00:00
parent: ARAWN-I-0004
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0004
---

# Add streaming response support to Agent

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0004]]

## Objective

Add streaming response support to the Agent, allowing real-time token-by-token output while the LLM generates its response. Essential for good UX in interactive conversations.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `Agent::turn_stream()` method returning `impl Stream<Item = StreamChunk>`
- [x] `StreamChunk` enum: `Text(String)`, `ToolStart { name }`, `ToolEnd { result }`, `Done`, `Error`
- [x] Streams text deltas as they arrive from LLM
- [x] Tool execution happens between stream chunks (text -> tool -> more text)
- [x] Handles multi-turn tool loops with streaming
- [x] Graceful cancellation via `CancellationToken`
- [x] Integration test with MockBackend streaming
- [x] `cargo test -p arawn-agent` passes (51 tests)

## Implementation Notes

### Technical Approach
Used `async_stream::stream!` macro to create a streaming agent loop that:
1. Checks for cancellation at each iteration
2. Calls LLM with streaming enabled
3. Yields text deltas as they arrive via `StreamEvent::ContentBlockDelta`
4. After streaming completes, makes a sync call to detect tool use
5. Executes tools and yields ToolStart/ToolEnd chunks
6. Continues loop for multi-turn tool execution
7. Yields Done chunk when no more tool calls

### Dependencies
- arawn-llm streaming support (complete_stream)
- async-stream crate for stream! macro
- tokio-util for CancellationToken

## Status Updates **[REQUIRED]**

### Session 1 - Completed
**Files created/modified:**
- `crates/arawn-agent/src/stream.rs` - New streaming module
- `crates/arawn-agent/src/lib.rs` - Added stream module export
- `crates/arawn-agent/src/agent.rs` - Added `turn_stream()` method
- `crates/arawn-agent/Cargo.toml` - Added `async-stream` dependency
- `crates/arawn-llm/src/lib.rs` - Exported `ContentDelta` type

**Implementation details:**
- `StreamChunk` enum with variants: Text, ToolStart, ToolEnd, Done, Error
- `AgentStream` type alias for `Pin<Box<dyn Stream<Item = StreamChunk> + Send>>`
- `create_turn_stream()` function using `async_stream::stream!` macro
- `Agent::turn_stream()` method wrapping the stream creation
- Full tool execution loop within the stream
- CancellationToken integration for graceful cancellation

**Tests:** 51 tests passing, including 6 new tests for StreamChunk