---
id: cli-commands-arawn-memory-reindex
level: task
title: "CLI Commands: arawn memory reindex and arawn memory stats"
short_code: "ARAWN-T-0070"
created_at: 2026-01-29T04:43:47.285575+00:00
updated_at: 2026-01-29T15:01:55.407616+00:00
parent: ARAWN-I-0015
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0015
---

# CLI Commands: arawn memory reindex and arawn memory stats

## Parent Initiative

[[ARAWN-I-0015]]

## Objective

Add `arawn memory reindex` and `arawn memory stats` CLI subcommands to `crates/arawn/src/commands/`. These expose the batch embedding pipeline and dimension migration to the user.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `arawn memory stats` prints: total memories, total embeddings, provider name, model, dimensions, stale status
- [ ] `arawn memory reindex` re-embeds all memories using current configured provider
- [ ] `arawn memory reindex --dry-run` prints count and estimated cost without doing work
- [ ] Progress bar during reindex (using `indicatif` or simple stderr output)
- [ ] Confirmation prompt before reindex ("This will re-embed N memories. Continue? [y/N]")
- [ ] `--yes` flag to skip confirmation
- [ ] Prints `BatchReport` summary on completion
- [ ] All existing tests pass

## Implementation Notes

### Technical Approach
- Add `memory.rs` subcommand module in `crates/arawn/src/commands/`
- `arawn memory` as a subcommand group with `stats` and `reindex` subcommands
- `stats`: open MemoryStore, read metadata table, count embeddings, print summary
- `reindex`: load config → build embedder → build BatchEmbedder → call MemoryStore::reindex()
- Progress: pass closure to BatchEmbedder that updates indicatif ProgressBar
- Wire into main CLI in `crates/arawn/src/main.rs`

### Dependencies
- T-0067 (BatchEmbedder) for batch processing
- T-0068 (Config) for provider selection
- T-0069 (Dimension migration) for reindex workflow on MemoryStore

## Status Updates

*To be added during implementation*