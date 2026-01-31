---
id: interaction-log-infrastructure
level: task
title: "Interaction Log Infrastructure: InteractionRecord, JSONL Writer, and Config"
short_code: "ARAWN-T-0052"
created_at: 2026-01-29T02:39:58.206310+00:00
updated_at: 2026-01-29T02:55:58.564971+00:00
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

# Interaction Log Infrastructure: InteractionRecord, JSONL Writer, and Config

## Parent Initiative

[[ARAWN-I-0011]]

## Objective

Create the shared `InteractionRecord` type, `InteractionLogger` with daily-rotating JSONL file output, and the `[logging.interactions]` config section. This is foundational infrastructure consumed by the router, agent turn loop, session indexer, and future training pipelines.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `InteractionRecord` struct defined with all fields (id, timestamp, duration, session context, request messages, system prompt, tools available, response content, token usage, stop reason, tool calls, routing metadata, tags)
- [ ] `ToolCallRecord` struct for capturing tool name, call ID, arguments, result success/content
- [ ] `RoutingMetadata` struct for routing decision data (profile, reason, confidence, features)
- [ ] `InteractionLogger` with daily-rotating JSONL file writer (one file per day in `~/.config/arawn/interactions/`)
- [ ] Logger emits `tracing::info` event alongside JSONL write for console visibility
- [ ] `InteractionRecord::from_exchange()` constructor that builds a record from `CompletionRequest` + `CompletionResponse`
- [ ] `[logging.interactions]` config section: enabled, path, retention_days, include_messages, include_responses, truncate_tool_results
- [ ] Config defaults: enabled=true, retention_days=90, truncate_tool_results=2048
- [ ] Unit tests: record serialization/deserialization roundtrip, JSONL format validation
- [ ] `cargo test` passes

## Implementation Notes

### Technical Approach
- Create `interaction_log.rs` in `arawn-llm` crate (since it depends on `CompletionRequest`/`CompletionResponse` types)
- Use `std::fs::File` with `BufWriter` wrapped in `Arc<Mutex<>>` for thread-safe writes
- Daily file rotation: filename format `interactions-YYYY-MM-DD.jsonl`, check date on each write
- Retention cleanup: on logger init, delete files older than `retention_days`
- `serde_json::to_string` for each record, one line per record

### Dependencies
- `arawn-llm` types: `CompletionRequest`, `CompletionResponse`, `Message`, `ContentBlock`, `StopReason`
- `arawn-config` for the new config section
- `uuid`, `chrono`, `serde`, `serde_json` crates (all already in workspace)

## Status Updates

*To be added during implementation*