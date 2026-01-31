---
id: routingbackend-and-multi-backend
level: task
title: "RoutingBackend and Multi-Backend Wiring in start.rs"
short_code: "ARAWN-T-0055"
created_at: 2026-01-29T02:40:10.598196+00:00
updated_at: 2026-01-29T03:38:24.401165+00:00
parent: ARAWN-I-0011
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0011
---

# RoutingBackend and Multi-Backend Wiring in start.rs

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0011]]

## Objective

Update `start.rs` to create multiple named `LlmBackend` instances from all configured `llm_profiles`, and expose them as a `HashMap<String, SharedBackend>` so that workstreams (and eventually agents/skills) can select a backend by profile name.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `start.rs` iterates `llm_profiles` and creates a `SharedBackend` for each
- [ ] Named backends stored as `HashMap<String, SharedBackend>`
- [ ] Default backend resolved from `[llm]` section (existing behavior preserved)
- [ ] Agent still receives a single backend (the default) — workstream-based selection comes in the Workstream initiative
- [ ] Backwards compatible: single `[llm]` config with no profiles still works
- [ ] `cargo test` passes

## Implementation Notes

### Technical Approach
- In `start.rs`, after loading config, iterate `config.llm_profiles` and call `LlmClient::create_backend()` for each
- Store in a `HashMap<String, SharedBackend>` alongside the default backend
- Pass the default to the agent as today — the map is available for future workstream integration
- No `RoutingBackend` wrapper needed — backend selection is a lookup by name, not a policy decision

### Dependencies
- ARAWN-T-0056 (config must support multiple profiles — already does via `llm_profiles`)

## Status Updates **[REQUIRED]**

*To be added during implementation*