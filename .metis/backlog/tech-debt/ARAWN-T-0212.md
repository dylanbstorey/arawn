---
id: enable-local-embeddings-feature-or
level: task
title: "Enable local-embeddings feature or improve fallback handling"
short_code: "ARAWN-T-0212"
created_at: 2026-02-19T18:13:24.997475+00:00
updated_at: 2026-02-20T02:02:44.463710+00:00
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

# Enable local-embeddings feature or improve fallback handling

## Objective

Fix the startup warning about local embeddings falling back to mock embedder, either by enabling the `local-embeddings` feature or improving how the fallback is handled.

## Warning

```
arawn_llm::embeddings: Local embeddings requested but 'local-embeddings' feature is not enabled. Falling back to mock embedder.
```

## Technical Debt Impact

- **Current Problems**: 
  - Warning shown on every server startup, cluttering logs
  - Mock embedder provides no real semantic search capability
  - Memory search and recall features are effectively disabled
  
- **Benefits of Fixing**: 
  - Clean startup logs
  - Functional semantic search for memory/recall
  - Better user experience with memory features
  
- **Risk Assessment**: Low risk - this is a configuration/feature flag issue

## Research Findings

### Current Architecture
- **Embedder implementations**: Mock, OpenAI, Local (ONNX) - all fully implemented
- **Feature flag**: `local-embeddings` in `arawn-llm` enables ONNX support
- **Default provider**: `Local` in config, but feature not enabled in binary
- **Result**: Warning + fallback to MockEmbedder (deterministic hashing, no semantics)

### Key Files
| File | Purpose |
|------|---------|
| `crates/arawn-llm/src/embeddings.rs` | All embedder implementations |
| `crates/arawn-llm/Cargo.toml` | Feature flag: `local-embeddings = ["ort", "tokenizers", "ndarray"]` |
| `crates/arawn-config/src/types.rs` | `EmbeddingConfig`, `EmbeddingProvider` enum |
| `crates/arawn/Cargo.toml` | Where feature would be enabled for binary |

### Dependencies (already in Cargo.lock)
- `ort = 2.0.0-rc.11` (ONNX Runtime)
- `tokenizers = 0.21` (HuggingFace)
- `ndarray = 0.16/0.17`

### Model Requirements
- Model: all-MiniLM-L6-v2 (~22MB ONNX + tokenizer)
- Location: `~/.local/share/arawn/models/embeddings/`
- Files: `model.onnx`, `tokenizer.json`

## Recommended Implementation

**Approach: Enable local embeddings by default with auto-download**

Philosophy: Semantic search should work out of the box. Local is the right default.

### Implementation Steps

1. **Enable `local-embeddings` feature** in the arawn binary
2. **Auto-download model files** on first startup if missing
3. **Silent fallback to mock** only if download fails (with info log, not warning)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] No warning on startup with default configuration
- [x] Default provider remains "local" 
- [x] Model files auto-download on first run (~22MB)
- [ ] Progress indicator during download (uses logging, not progress bar)
- [x] Graceful fallback to mock if download fails (info log, not warning)
- [x] Local embeddings work immediately after download

## Implementation Plan

### Files to Modify

```
crates/arawn/Cargo.toml
  - Enable local-embeddings feature:
    arawn-llm = { workspace = true, features = ["local-embeddings"] }

crates/arawn-llm/src/embeddings.rs
  - Add download_model() function for auto-download
  - Modify LocalEmbedder::new() to auto-download if files missing
  - Change warning to info for fallback case
  - Add progress callback for download status

crates/arawn-llm/Cargo.toml
  - Add reqwest dependency for downloads (or reuse existing)
```

### Download Details

**Source:** HuggingFace Hub
- Model: `sentence-transformers/all-MiniLM-L6-v2`
- Files needed:
  - `model.onnx` (~22MB) 
  - `tokenizer.json` (~700KB)

**Target:** `~/.local/share/arawn/models/embeddings/`

**Download Logic:**
```rust
// In build_embedder() when provider == "local":
1. Check if model files exist at default path
2. If missing, attempt download with progress
3. If download succeeds, load LocalEmbedder
4. If download fails, log info and use MockEmbedder
```

### Startup Behavior

**First run (no model):**
```
INFO Downloading embedding model (22MB)...
INFO [████████████████████] 100% - Complete
INFO Local embeddings ready (all-MiniLM-L6-v2)
```

**Subsequent runs:**
```
INFO Local embeddings loaded (all-MiniLM-L6-v2)
```

**Offline/download failed:**
```
INFO Could not download embedding model, using mock embeddings
INFO Semantic search will use deterministic matching
```

## Status Updates

### 2026-02-20: Implementation Complete
Successfully implemented local embeddings with auto-download:

**Changes Made:**
1. **Workspace Cargo.toml**: Added features to ort dependency: `["std", "ndarray", "download-binaries", "copy-dylibs", "tls-native"]`
2. **arawn/Cargo.toml**: Already had `local-embeddings` feature enabled
3. **arawn-llm/src/embeddings.rs**:
   - Made `build_embedder()` async to support downloads
   - Added `ensure_model_files()` function that auto-downloads model.onnx and tokenizer.json from HuggingFace
   - Added `download_file()` helper for downloading with atomic writes
   - Fixed ort 2.0 API compatibility:
     - Changed tensor creation from ndarray to tuple form `(shape, Vec<T>)`
     - Wrapped Session in Mutex for interior mutability (Session::run requires &mut self)
     - Fixed output extraction to use try_extract_tensor returning `(&Shape, &[f32])`
   - Removed ndarray import (no longer needed)
4. **arawn/src/commands/start.rs**: Updated call to `build_embedder()` with `.await`
5. **arawn/src/commands/memory.rs**: Updated call to `build_embedder()` with `.await`

**Model Files:**
- Downloaded to `~/.local/share/arawn/models/embeddings/`
- model.onnx (~22MB) from HuggingFace all-MiniLM-L6-v2
- tokenizer.json (~700KB)

**Behavior:**
- On first startup: automatically downloads model files with logging
- On subsequent runs: loads existing model files
- If download fails: gracefully falls back to mock embedder with info log

All tests pass. Build succeeds.

### 2026-02-19: Research Complete
Explored embeddings implementation. Found that local embedder is fully implemented but feature-gated. The warning occurs because default config is "local" but binary doesn't include ONNX support. Recommended fix: change default to "mock" to eliminate warning, then optionally enable local-embeddings feature with setup command for model download.