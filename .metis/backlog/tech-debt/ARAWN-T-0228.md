---
id: auto-download-gliner-model-files
level: task
title: "Auto-download GLiNER model files like embeddings"
short_code: "ARAWN-T-0228"
created_at: 2026-02-26T01:49:12.647713+00:00
updated_at: 2026-02-26T17:00:56.055696+00:00
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

# Auto-download GLiNER model files like embeddings

## Objective

GLiNER NER is behind a feature flag (`gliner`) and currently requires the user to manually download ONNX model + tokenizer files and set `ner_model_path` / `ner_tokenizer_path` in config. This is friction that doesn't exist for the embedding model, which auto-downloads from HuggingFace via `ensure_model_files()` in `embeddings.rs`.

Add auto-download for GLiNER model files so that when the `gliner` feature is enabled and `ner_model_path` is not set, Arawn downloads the default model on first use — matching the embedding model UX.

Also: make the download URLs configurable in `arawn.toml` (both for GLiNER and the existing embedding model URLs which are currently hardcoded as `MODEL_URL`/`TOKENIZER_URL` constants in `embeddings.rs:668-669`).

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P3 - Low (when time permits)

### Technical Debt Impact
- **Current Problems**: Users must manually find, download, and configure GLiNER model files. Embedding model download URLs are hardcoded constants — can't point at a mirror or different model version without code changes.
- **Benefits of Fixing**: Zero-config GLiNER setup when feature is enabled. Configurable model URLs for both embeddings and NER. Consistent auto-download pattern across all local models.
- **Risk Assessment**: Low — additive change. Existing manual config path still works. Auto-download is opt-in (only triggers when paths aren't set).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] When `gliner` feature is enabled and `ner_model_path` is unset, auto-download default GLiNER model from HuggingFace to `~/.local/share/arawn/models/ner/`
- [x] Download URLs for both embedding and NER models configurable in `arawn.toml` (with sensible HuggingFace defaults)
- [x] Existing `ner_model_path` / `ner_tokenizer_path` config overrides still work (explicit path wins over auto-download)
- [x] Progress/status logged during download (consistent with embedding download logging)
- [ ] ~`--dry-run` or equivalent to show what would be downloaded without doing it~ (Deferred — no existing dry-run infrastructure for model downloads)

## Implementation Notes

### Key Files
- `arawn-llm/src/embeddings.rs:658-715` — existing `ensure_model_files()` / `download_file()` pattern to copy
- `arawn-llm/src/embeddings.rs:668-669` — hardcoded `MODEL_URL` / `TOKENIZER_URL` to make configurable
- `arawn-config/src/types.rs:691-718` — `IndexingConfig` (add optional download URL fields)
- `arawn/src/commands/start.rs:900-930` — GLiNER wiring (add auto-download before `GlinerEngine::new()`)

### Technical Approach
1. Extract the `ensure_model_files()` / `download_file()` pattern into a shared utility (or keep per-module)
2. Add `ner_model_url` / `ner_tokenizer_url` fields to `IndexingConfig` with HuggingFace defaults
3. Add `model_url` / `tokenizer_url` fields to embedding config (or `EmbeddingLocalConfig`) for the existing hardcoded URLs
4. In `start.rs` GLiNER wiring: if `ner_model_path` is unset and `gliner` feature is enabled, call `ensure_ner_model_files()` and use the downloaded path
5. Default cache dir: `~/.local/share/arawn/models/ner/` (parallel to `embeddings/`)

## Status Updates

### Session 1 (2026-02-26)
**Completed all core work:**

**Config layer (`arawn-config/src/types.rs`):**
- Added `model_url: Option<String>` and `tokenizer_url: Option<String>` to `EmbeddingLocalConfig`
- Added `ner_model_url: Option<String>` and `ner_tokenizer_url: Option<String>` to `IndexingConfig`
- Updated `ner_model_path` docstring to note auto-download behavior when unset

**Download infrastructure (`arawn-llm/src/embeddings.rs`):**
- Made `download_file()` public (was private + feature-gated)
- Made embedding URL constants public: `DEFAULT_EMBEDDING_MODEL_URL`, `DEFAULT_EMBEDDING_TOKENIZER_URL`
- Added NER URL constants: `DEFAULT_NER_MODEL_URL`, `DEFAULT_NER_TOKENIZER_URL`
- Updated `ensure_model_files()` to accept optional URL overrides (falls back to defaults)
- Created public `ensure_ner_model_files()` — downloads to `~/.local/share/arawn/models/ner/`
- Created public `default_ner_model_dir()` helper
- Added `local_model_url` and `local_tokenizer_url` to `EmbedderSpec`

**Wiring (`arawn/src/commands/start.rs`):**
- `build_embedder_spec()` now passes `model_url`/`tokenizer_url` from local config through to `EmbedderSpec`
- GLiNER block rewritten: when `ner_model_path` is unset, calls `ensure_ner_model_files()` for auto-download; when set, uses explicit path as before
- URL overrides from config (`ner_model_url`/`ner_tokenizer_url`) passed through to download function

**Also fixed:** `memory.rs` EmbedderSpec construction site updated with new fields.

**All tests pass, `angreal check all` clean.**