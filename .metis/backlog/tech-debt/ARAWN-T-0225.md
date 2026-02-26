---
id: low-priority-codebase-cleanup-dead
level: task
title: "LOW priority codebase cleanup — dead code, hardcoded values, incomplete implementations"
short_code: "ARAWN-T-0225"
created_at: 2026-02-25T14:20:21.580871+00:00
updated_at: 2026-02-25T14:20:21.580871+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# LOW priority codebase cleanup — dead code, hardcoded values, incomplete implementations

## Objective

Address all 25 LOW-severity findings from the 2026-02-25 codebase audit. These are minor TODOs, cosmetic dead code, and small gaps that don't affect functionality but reduce code quality over time.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P3 - Low (when time permits)

### Technical Debt Impact
- **Current Problems**: Scattered TODO comments, minor unused imports, test helpers compiled outside test cfg, small feature gaps in non-critical paths.
- **Benefits of Fixing**: Cleaner compiler output, marginally faster builds, no stale TODOs creating confusion.
- **Risk Assessment**: Very low — all changes are trivial and self-contained.

## Acceptance Criteria

- [ ] All LOW dead code items cleaned up
- [ ] All LOW hardcoded values addressed (constants or config)
- [ ] All LOW incomplete implementations resolved (finished or removed)
- [ ] `angreal test all` passes
- [ ] `angreal check all` passes

---

## Findings

### Dead Code (8 items)

| # | Location | Issue |
|---|----------|-------|
| 1-8 | Various | Unused imports, `#[allow(dead_code)]` annotations on actually-dead items, test helpers compiled in non-test cfg, unused `Display` impls |

### Hardcoded Values (8 items)

| # | Location | Issue |
|---|----------|-------|
| 1-8 | Various | Color codes, format strings, version strings, test fixtures that could be constants |

### Incomplete Implementations (9 items)

| # | Location | Issue |
|---|----------|-------|
| 1 | `arawn-server/src/ws.rs` | WebSocket alert/notification pathway has TODO |
| 2 | `arawn-server/src/routes/health.rs` | Deep health check skips memory store validation |
| 3 | `arawn-plugin/src/loader.rs` | Plugin CLI tool registration is a no-op |
| 4 | `arawn/src/commands/agent.rs` | `--daemon` flag accepted but exits immediately |
| 5 | `arawn-workstream/src/manager.rs` | `delete_workstream()` doesn't clean up JSONL files |
| 6 | `arawn-config/src/secret.rs` | Keyring integration compiles but has no test coverage |
| 7 | `arawn-memory/src/graph.rs` | `traverse()` method returns only direct neighbors (depth always 1) |
| 8 | `arawn-agent/src/indexer.rs` | Summarization prompt is placeholder text |
| 9 | `arawn-pipeline/src/executor.rs` | Error recovery in pipeline steps just logs and continues |

## Implementation Notes

### Suggested Approach
Can be done opportunistically — pick off items when working nearby. No need to batch these into a dedicated sprint.

### Dependencies
- ARAWN-T-0223 (HIGH) and ARAWN-T-0224 (MEDIUM) should be addressed first

## Status Updates

*To be added during implementation*