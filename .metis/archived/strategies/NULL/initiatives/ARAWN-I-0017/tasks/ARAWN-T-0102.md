---
id: implement-session-summarization
level: task
title: "Implement session summarization"
short_code: "ARAWN-T-0102"
created_at: 2026-01-31T04:09:07.360527+00:00
updated_at: 2026-02-01T03:51:15.759396+00:00
parent: ARAWN-I-0017
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0017
---

# Implement session summarization

## Objective

Implement session summarization: given a session's conversation history, use the configured LLM backend to generate a 2-3 sentence summary. Store the summary as a `ContentType::Summary` memory with session_id metadata.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `SummarizationPrompt` that formats history and instructs LLM to summarize (what was accomplished, key decisions, open questions)
- [ ] `summarize_session(history) -> Result<String>` function that calls LLM and returns summary text
- [ ] Summary stored as `Memory::new(ContentType::Summary, text)` with `session_id` in metadata
- [ ] Summary is embedded (if embedder available) so it's searchable via recall
- [ ] Graceful handling of empty sessions (no summary generated)
- [ ] Tests: prompt formatting, summary storage with correct content type and metadata

## Implementation Notes

### Files
- `crates/arawn-agent/src/indexing/summarization.rs` — prompt + summarize function

### Dependencies
- ARAWN-T-0095 (ContentType::Summary)
- ARAWN-T-0100 (indexing config)

## Status Updates

### Session 1
- Created `crates/arawn-agent/src/indexing/summarization.rs`
- `SummarizationPrompt::build(messages) -> Option<String>` — returns None for empty sessions
- Prompt instructs LLM to focus on: what was accomplished, key decisions, open questions/next steps
- Rules: be specific (file names, tools), 2-3 sentences, past tense for done / future for planned
- `clean_summary(raw) -> String` — strips "Summary:" prefixes, markdown headers, code fences
- Updated `indexing/mod.rs` to expose `pub mod summarization` and re-export `SummarizationPrompt`
- 11 tests: prompt formatting, empty session handling, all clean_summary edge cases
- `angreal check all` passes, `angreal test unit` passes (all 11 summarization tests green)

Note: The actual `summarize_session()` function that calls LLM and stores the Memory will live in the SessionIndexer (T-0103), which orchestrates extraction + summarization + storage. This task covers the prompt building and output cleaning.