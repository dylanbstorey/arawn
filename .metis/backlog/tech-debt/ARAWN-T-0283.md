---
id: add-streaming-mockbackend-and-sse
level: task
title: "Add streaming MockBackend and SSE response tests"
short_code: "ARAWN-T-0283"
created_at: 2026-03-08T03:17:27.702283+00:00
updated_at: 2026-03-08T03:17:27.702283+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
initiative_id: NULL
---

# Add streaming MockBackend and SSE response tests

## Objective

The existing `MockBackend` in `arawn-llm` only returns complete `CompletionResponse` objects — it has no streaming support. The TUI, WebSocket chat, and HTTP chat endpoints all use SSE streaming as the primary response mechanism, but this path is completely untestable. Create a `StreamingMockBackend` and use it to test SSE responses end-to-end.

### Priority
- [x] P1 - High (streaming is the primary user-facing response path)
- **Size**: M

### Current Problems
- `MockBackend` implements `LlmBackend::complete()` but not streaming variants
- No way to test partial response delivery, chunk boundaries, or stream interruption
- SSE responses from `/api/v1/chat` (streaming mode) never tested
- WebSocket chat response streaming never tested
- Agent `turn_stream()` tested minimally — `stream.rs` has only 6 tests
- TUI response rendering during streaming untestable

## Acceptance Criteria

- [ ] `StreamingMockBackend` created in `arawn-llm` (feature-gated `testing`)
- [ ] Yields configurable chunks: text deltas, tool_use blocks, stop reasons
- [ ] Supports configurable inter-chunk delay (for timing tests)
- [ ] Tests in `arawn-server/tests/` verify SSE chat endpoint streams correctly
- [ ] Tests in `arawn-agent` verify `turn_stream()` produces correct `StreamChunk` sequence
- [ ] Test for stream interruption/cancellation mid-response
- [ ] Test for tool_use appearing mid-stream

## Implementation Notes

### StreamingMockBackend API

```rust
// In arawn-llm/src/backend.rs (behind #[cfg(any(test, feature = "testing"))])
pub struct StreamingMockBackend {
    chunks: Vec<StreamChunk>,
    delay: Option<Duration>,
}

impl StreamingMockBackend {
    pub fn new(chunks: Vec<StreamChunk>) -> Self;
    pub fn with_delay(self, delay: Duration) -> Self;
    
    // Convenience builders
    pub fn text_response(text: &str, chunk_size: usize) -> Self;
    pub fn tool_then_text(tool_name: &str, args: Value, text: &str) -> Self;
}
```

### SSE test pattern

```rust
#[tokio::test]
async fn test_chat_sse_streaming() {
    let server = TestServer::builder()
        .with_streaming_backend(StreamingMockBackend::text_response("Hello world", 5))
        .build().await;
    
    let resp = server.post("/api/v1/chat")
        .header("Accept", "text/event-stream")
        .json(&json!({"message": "hi"}))
        .send().await.unwrap();
    
    assert_eq!(resp.status(), 200);
    // Read SSE events
    let events = collect_sse_events(resp).await;
    assert!(events.len() > 1); // Multiple chunks
    assert_eq!(reconstruct_text(&events), "Hello world");
}
```

### Key files
- `crates/arawn-llm/src/backend.rs` — Add StreamingMockBackend
- `crates/arawn-server/tests/chat_integration.rs` — Add SSE streaming tests
- `crates/arawn-agent/src/stream.rs` — Add stream behavior tests

### Dependencies
- Could be part of ARAWN-T-0279 (test utils) or standalone

## Status Updates

*To be added during implementation*