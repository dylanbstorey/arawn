---
id: routingpolicy-trait
level: task
title: "RoutingPolicy Trait, RequestFeatures, and Heuristic LayeredPolicy"
short_code: "ARAWN-T-0054"
created_at: 2026-01-29T02:40:06.819096+00:00
updated_at: 2026-01-29T02:59:13.758812+00:00
parent: ARAWN-I-0011
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0011
---

# RoutingPolicy Trait, RequestFeatures, and Heuristic LayeredPolicy

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0011]]

## Objective

Implement the `RoutingPolicy` trait, `RequestFeatures` extraction, `RoutingDecision` type, and the heuristic `LayeredPolicy` that classifies requests by complexity using keyword matching, message length, tool presence, and conversation depth.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `RoutingPolicy` trait defined: `fn route(&self, request: &CompletionRequest) -> RoutingDecision`
- [ ] `RoutingDecision` struct: profile, reason (CallerHint/HeuristicRule/EmbeddingClassifier/Default), confidence, features
- [ ] `RequestFeatures` struct: message_length, message_count, has_tools, tool_count, has_system_prompt, keyword_signals, estimated_complexity
- [ ] `Complexity` enum: Simple, Moderate, Complex
- [ ] `extract_features()` function producing `RequestFeatures` from `CompletionRequest`
- [ ] `contains_complexity_keywords()` matching against keyword list (analyze, implement, refactor, debug, etc.)
- [ ] `RoutingRule` struct with condition matching against `RequestFeatures`
- [ ] `LayeredPolicy` implementing `RoutingPolicy`: caller hint → heuristic rules → default (in priority order)
- [ ] Unit tests: feature extraction for simple/moderate/complex messages, rule matching, layered priority
- [ ] `cargo test` passes

## Implementation Notes

### Technical Approach
- Create `routing.rs` module in `arawn-llm` crate
- All types are pure logic — no I/O, no async. Easy to unit test.
- `LayeredPolicy` takes a `Vec<RoutingRule>` and `default_profile: String`
- Rules are evaluated in order; first match wins
- Caller hint checked via `request.metadata.get("routing_profile")`

### Dependencies
- `arawn-llm` types: `CompletionRequest` (needs a `metadata: HashMap<String, String>` field if not already present)

## Status Updates **[REQUIRED]**

*To be added during implementation*