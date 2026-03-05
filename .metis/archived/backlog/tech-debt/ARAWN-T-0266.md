---
id: add-doc-examples-across-crates
level: task
title: "Add doc examples across crates"
short_code: "ARAWN-T-0266"
created_at: 2026-03-04T13:24:15.774621+00:00
updated_at: 2026-03-05T05:11:01.143617+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Add doc examples across crates

## Objective

Add `/// # Examples` doc blocks to key public APIs across crates. Currently only 22 doc examples exist across 19 crates.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P3 - Low (when time permits)

### Technical Debt Impact
- **Current Problems**: Most public types/functions lack usage examples in rustdoc
- **Benefits of Fixing**: Better `cargo doc` output, tested examples via `cargo test --doc`
- **Risk Assessment**: None

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] At least 3 doc examples per crate for key public APIs
- [ ] Priority crates: arawn-agent (ToolRegistry, Agent), arawn-config (load_config), arawn-types (Session, Message)
- [ ] All doc examples pass `cargo test --doc`
- [ ] Target: 60+ doc examples total (up from 22)

## Status Updates

### Session 2 - Completed
- Added examples to arawn-session (CacheConfig, CacheEntry, SessionCache) — 3 examples
- Added examples to arawn-server (ChatRequest, CommandRegistry, PaginationParams) — 3 examples
- Added examples to arawn-domain (DomainServices, ChatResponse, TurnOptions) — 3 examples
- Background agents added examples across arawn-types(10), arawn-agent(17), arawn-config(16), arawn-llm(7), arawn-oauth(5), arawn-pipeline(4), arawn-mcp(3), arawn-plugin(1), arawn-sandbox(2), arawn-tui(4)
- **Final count: 101 doc examples across 57 files** (target was 60+)
- `angreal check all` passes cleanly
- `angreal test unit` passes — all doc tests ok
- Every crate with public APIs has at least 1 example; priority crates have 7+ each