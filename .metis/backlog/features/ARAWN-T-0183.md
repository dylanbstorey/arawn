---
id: trait-based-rate-limit-handling
level: task
title: "Trait-based rate limit handling for LLM backends"
short_code: "ARAWN-T-0183"
created_at: 2026-02-16T15:18:24.402978+00:00
updated_at: 2026-02-17T13:46:44.674130+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Trait-based rate limit handling for LLM backends

## Objective

Implement a trait-based approach for handling rate limit errors gracefully across different LLM providers (Groq, OpenAI, Anthropic), enabling automatic retry with provider-specified delays and proper HTTP 429 propagation to clients.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement  

### Priority
- [ ] P1 - High (important for user experience)

### Business Justification
- **User Value**: Automatic recovery from rate limits instead of cryptic 500 errors; transparent retry behavior
- **Business Value**: Better reliability during high usage; reduced user frustration from transient failures
- **Effort Estimate**: M (Medium)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `LlmError::RateLimit` stores `RateLimitInfo` struct with optional `retry_after: Duration`
- [ ] `RateLimitParser` trait exists with implementations for Groq, OpenAI, Anthropic
- [ ] `is_retryable()` returns true for rate limit errors
- [ ] `with_retry()` respects provider-specified retry delays
- [ ] Groq "Please try again in Xs" message is parsed into Duration
- [ ] OpenAI/Anthropic `Retry-After` headers are parsed
- [ ] Server returns HTTP 429 (not 500) when upstream rate limit occurs
- [ ] 429 response includes `Retry-After` header when timing is known
- [ ] All existing tests pass



## Implementation Notes

### Current State (Problem)
- `LlmError::RateLimit(String)` only stores error message, loses retry timing
- `is_retryable()` returns false for rate limits (only true for Network errors)
- `with_retry()` won't retry rate limits even though they're inherently retryable
- Groq returns "Please try again in 6.57792s" but timing is discarded
- Server returns HTTP 500 for upstream rate limits instead of 429

### Technical Approach

**Phase 1: Enhanced Error Type**
```rust
pub struct RateLimitInfo {
    pub message: String,
    pub retry_after: Option<Duration>,
    pub limit_type: Option<RateLimitType>,
}
```

**Phase 2: Parser Trait**
```rust
pub trait RateLimitParser {
    fn parse_rate_limit(status: u16, headers: &HeaderMap, body: &str) -> Option<RateLimitInfo>;
}
```

**Phase 3: Retry Logic**
- Update `is_retryable()` to include rate limits
- Update `with_retry()` to use `retry_after` duration when available

**Phase 4: HTTP Backends**
- Groq: Parse "Please try again in Xs" from body text
- OpenAI: Use `Retry-After` header + `x-ratelimit-*` headers
- Anthropic: Use `retry-after` header

**Phase 5: Server Propagation**
- Map upstream rate limits to HTTP 429 responses
- Include `Retry-After` header in response

### Files to Modify
- `crates/arawn-llm/src/error.rs` - RateLimitInfo struct
- `crates/arawn-llm/src/backend.rs` - with_retry() logic
- `crates/arawn-llm/src/openai.rs` - Parser for Groq/OpenAI
- `crates/arawn-server/src/error.rs` - 429 response mapping

### Dependencies
- None (self-contained in arawn-llm and arawn-server)

### Risk Considerations
- Breaking change to `LlmError::RateLimit` variant signature
- Need to handle providers that don't include timing info (fall back to exponential backoff)

## Status Updates

### Progress

**Implementation Complete:**

1. **RateLimitInfo struct** (`arawn-llm/src/error.rs`)
   - Added `RateLimitInfo` with `message`, `retry_after: Option<Duration>`, `limit_type: Option<RateLimitType>`
   - Added `parse_groq()` for Groq's "try again in Xs" format
   - Added `parse_openai()` for Retry-After header parsing

2. **Updated LlmError::RateLimit** 
   - Changed from `RateLimit(String)` to `RateLimit(RateLimitInfo)`
   - Added `retry_after()` and `is_retryable()` methods
   - Rate limits now retryable alongside network errors

3. **Updated with_retry()** (`arawn-llm/src/backend.rs`)
   - Uses provider-specified retry delay when available
   - Falls back to exponential backoff otherwise
   - Adds 100ms buffer to provider timing

4. **OpenAI backend** (`arawn-llm/src/openai.rs`)
   - Parses Retry-After header
   - Detects Groq timing in error messages
   - Creates proper RateLimitInfo

5. **Anthropic backend** (`arawn-llm/src/anthropic.rs`)
   - Parses Retry-After header
   - Creates proper RateLimitInfo

6. **AgentError** (`arawn-agent/src/error.rs`)
   - Added `is_rate_limit()`, `llm_error()`, `retry_after()` methods

7. **ServerError** (`arawn-server/src/error.rs`)
   - Added `RateLimitError` struct with retry timing
   - Server returns HTTP 429 for upstream rate limits
   - Adds Retry-After header when timing is known

All tests pass.