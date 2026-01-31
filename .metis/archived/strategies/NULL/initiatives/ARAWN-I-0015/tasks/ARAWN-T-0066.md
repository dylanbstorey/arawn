---
id: geminiembedder-google-text
level: task
title: "GeminiEmbedder: Google text-embedding-004 Provider"
short_code: "ARAWN-T-0066"
created_at: 2026-01-29T04:43:46.937644+00:00
updated_at: 2026-01-29T04:43:46.937644+00:00
parent: ARAWN-I-0015
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0015
---

# GeminiEmbedder: Google text-embedding-004 Provider

## Parent Initiative

[[ARAWN-I-0015]]

## Objective

Implement a `GeminiEmbedder` in `crates/arawn-llm/src/embeddings.rs` that calls Google's Gemini embedding API, following the same pattern as the existing `OpenAiEmbedder`. Supports `text-embedding-004` (768 dimensions) with native batch embedding via `batchEmbedContents`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `GeminiEmbedder` struct implements the `Embedder` trait (`embed`, `embed_batch`, `dimensions`, `name`)
- [ ] Calls `https://generativelanguage.googleapis.com/v1beta/models/{model}:embedContent` for single embeds
- [ ] Calls `batchEmbedContents` endpoint for batch embeds (up to 100 inputs per request)
- [ ] Configurable via `GeminiEmbedderConfig` (api_key, model, timeout)
- [ ] Default model is `text-embedding-004`, 768 dimensions
- [ ] Unit tests with mocked HTTP responses
- [ ] All existing tests pass

## Implementation Notes

### Technical Approach
- Add `GeminiEmbedder` and `GeminiEmbedderConfig` alongside the existing `OpenAiEmbedder` in `crates/arawn-llm/src/embeddings.rs`
- Google API uses `?key=` query param for auth (not Bearer header)
- Request body: `{ "model": "models/text-embedding-004", "content": { "parts": [{ "text": "..." }] } }`
- Batch endpoint: `batchEmbedContents` accepts `requests` array
- Re-use existing `reqwest::Client` patterns from `OpenAiEmbedder`

### Dependencies
- None — standalone addition to arawn-llm

## Status Updates

*To be added during implementation*