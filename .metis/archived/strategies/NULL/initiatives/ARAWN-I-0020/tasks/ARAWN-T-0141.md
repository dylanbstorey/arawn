---
id: context-injection-from-parent-to
level: task
title: "Context injection from parent to subagent"
short_code: "ARAWN-T-0141"
created_at: 2026-02-06T03:47:50.326352+00:00
updated_at: 2026-02-06T14:02:10.653785+00:00
parent: ARAWN-I-0020
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0020
---

# Context injection from parent to subagent

## Parent Initiative

[[ARAWN-I-0020]] - Subagent Delegation

## Objective

Pass relevant context from the parent session to the subagent so it has useful background information without polluting its session history.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `context` parameter in delegate tool is passed to subagent
- [x] Context injected as system-level preamble, not as conversation turn
- [x] Subagent sees context but doesn't include it in its response history
- [x] Empty/missing context doesn't inject anything
- [x] Context truncated if too long (configurable limit)
- [ ] Integration test: subagent can reference injected context (deferred - requires E2E setup)

## Implementation Notes

### Context Injection Approach

Option A: **System prompt injection** (preferred)
- Append context to subagent's system prompt
- Clean separation from conversation history

Option B: **Synthetic first turn**
- Add context as fake "user" message
- Subagent sees it as first message

### Implementation

```rust
fn create_subagent_session(context: Option<&str>, max_context_len: usize) -> Session {
    let mut session = Session::new();
    
    if let Some(ctx) = context {
        let truncated = if ctx.len() > max_context_len {
            format!("{}...(truncated)", &ctx[..max_context_len])
        } else {
            ctx.to_string()
        };
        
        session.set_context_preamble(format!(
            "[Context from parent agent]\n{}",
            truncated
        ));
    }
    
    session
}
```

### Session API Addition

May need to add `set_context_preamble()` to `Session` struct if not present:

```rust
impl Session {
    /// Set a context preamble that's included in prompts but not in history
    pub fn set_context_preamble(&mut self, preamble: String) {
        self.context_preamble = Some(preamble);
    }
}
```

### Configuration

- Default max context length: 4000 chars
- Could be configurable per-agent in plugin config

### Dependencies

- [[ARAWN-T-0138]] - Basic execution must work first

## Status Updates

### 2026-02-06: Implementation Plan (Option B)

**Approach**: Add `context_preamble` field to `Session` - a general-purpose pattern for dynamic system context injection.

**Files to modify:**

1. **`crates/arawn-agent/src/types.rs`** - Session struct
   - Add `context_preamble: Option<String>` field
   - Add `set_context_preamble()` and `context_preamble()` methods

2. **`crates/arawn-agent/src/agent.rs`** - Agent turn loop
   - Include `session.context_preamble` in the system message when building CompletionRequest

3. **`crates/arawn-plugin/src/agent_spawner.rs`** - Subagent delegation
   - Replace user message context injection with `session.set_context_preamble()`
   - Add truncation logic (default 4000 chars)

**Key insight**: The preamble is included in the LLM request's system message but NOT stored in `Session.turns`, so it doesn't pollute history.

### 2026-02-06: Implementation Complete

**Changes made:**

1. **`crates/arawn-agent/src/types.rs`** - Session struct
   - Added `context_preamble: Option<String>` field
   - Added `set_context_preamble()`, `clear_context_preamble()`, `context_preamble()` methods
   - Added unit tests for preamble functionality

2. **`crates/arawn-agent/src/agent.rs`** - Agent turn loop
   - Updated `build_request()` to accept `context_preamble: Option<&str>`
   - Preamble injected as `[Session Context]\n{preamble}\n\n---\n\n{base_prompt}` format
   - Call site updated: `self.build_request(&messages, session.context_preamble())`

3. **`crates/arawn-agent/src/context.rs`** - ContextBuilder
   - Updated `build()` to pass `session.context_preamble()` through
   - Updated `build_request()` with same system prompt injection pattern

4. **`crates/arawn-plugin/src/agent_spawner.rs`** - Subagent delegation
   - Added `DEFAULT_MAX_CONTEXT_LEN = 4000` constant
   - Added `truncate_context()` helper with word-boundary-aware truncation
   - `delegate()` and `delegate_background()` now use `session.set_context_preamble()`
   - Removed context embedding from user message
   - Added 5 unit tests for truncation logic

**Tests:** All 145 tests in arawn-plugin pass, plus new tests in arawn-agent for Session preamble and ContextBuilder.