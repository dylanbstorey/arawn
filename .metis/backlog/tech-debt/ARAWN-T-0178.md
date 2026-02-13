---
id: bounded-collections-library
level: task
title: "Bounded Collections Library"
short_code: "ARAWN-T-0178"
created_at: 2026-02-13T16:39:53.383709+00:00
updated_at: 2026-02-13T20:49:23.746708+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Bounded Collections Library

## Objective

Create reusable bounded collection types to prevent unbounded memory growth in message vectors and other collections.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P3 - Low (when time permits)

### Technical Debt Impact
- **Current Problems**: Multiple places implement ad-hoc bounds checking for vectors. No reusable types for bounded collections.
- **Benefits of Fixing**: Compile-time guarantees on collection bounds, reduced code duplication, clearer intent.
- **Risk Assessment**: LOW - Existing ad-hoc solutions work; this is a polish item.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Evaluate existing crates (`bounded-vec`, `ringbuf`, `arrayvec`) vs custom implementation
- [x] Custom: Create `BoundedVec<T>` with push-and-evict semantics (runtime capacity)
- [x] ~~Create `RingBuffer<T, const N: usize>`~~ (not needed - BoundedVec sufficient)
- [x] Replace ad-hoc bounds checking in TUI message vectors
- [x] Add unit tests for overflow behavior (10 tests)

## Implementation Notes

### Technical Approach

**Option A: Use existing crates**
- `ringbuf` - Lock-free ring buffer
- `arrayvec` - Stack-allocated bounded vec
- Pros: Battle-tested, maintained
- Cons: May not fit exact use case

**Option B: Custom implementation**
```rust
pub struct BoundedVec<T, const N: usize> {
    inner: VecDeque<T>,
}

impl<T, const N: usize> BoundedVec<T, N> {
    pub fn push(&mut self, item: T) {
        if self.inner.len() >= N {
            self.inner.pop_front();
        }
        self.inner.push_back(item);
    }
}
```

### Locations to Update
- `crates/arawn-tui/src/app.rs` - Message and tool vectors
- `crates/arawn-tui/src/input.rs` - History buffer (already uses VecDeque)

### Recommendation
Start with existing `ringbuf` crate unless specific needs arise.

## Status Updates

### Session 1 - 2026-02-13

**Decision:** Custom implementation over external crates because:
- `ringbuf` is overkill (lock-free features not needed)
- `arrayvec` requires const generics for size (stack-allocated)
- Simple ~120 lines of code avoids external dependency

**Implementation:**
- Created `crates/arawn-tui/src/bounded.rs` with `BoundedVec<T>`:
  - Runtime-configurable max capacity
  - Push evicts oldest 10% when at capacity
  - `replace_from_vec()` for bulk loading
  - Deref to `[T]` for slice methods
  - Indexing support
  - 10 unit tests covering eviction, edge cases

**Integration:**
- Updated `App.messages` and `App.tools` to use `BoundedVec`
- Simplified `push_message()` and `push_tool()` (bounds now automatic)
- Updated `do_fetch_session_messages()` to use `replace_from_vec()`

**All 33 arawn-tui tests passing.**