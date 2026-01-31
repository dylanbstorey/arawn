---
id: batch-embedding-pipeline-openai
level: initiative
title: "Batch Embedding Pipeline: OpenAI, Gemini, and Local Providers"
short_code: "ARAWN-I-0015"
created_at: 2026-01-29T01:21:50.875323+00:00
updated_at: 2026-01-29T15:05:26.729310+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: batch-embedding-pipeline-openai
---

# Batch Embedding Pipeline: OpenAI, Gemini, and Local Providers Initiative

## Context

Arawn's memory system currently uses local ONNX inference (MiniLM-L6-v2, 384 dimensions) for embeddings via the `ort` crate. This works well for small-scale, real-time embedding of individual memories during conversation. However, it has limitations:

- **Model quality**: MiniLM-L6-v2 is fast but lower quality than models like `text-embedding-3-small` (OpenAI) or `text-embedding-004` (Gemini). For knowledge retrieval, embedding quality directly impacts recall accuracy.
- **Batch processing**: Indexing large document sets (codebases, note archives, research papers) requires batch embedding. Local ONNX inference on CPU is slow for thousands of documents.
- **No API fallback**: If the local model isn't available (missing ONNX runtime, ARM device without compatible build), there's no fallback to cloud embedding APIs.
- **Single model**: Can't mix embedding models — e.g., use a cheap fast model for session memories and a high-quality model for knowledge base entries.

## Goals & Non-Goals

**Goals:**
- Abstract embedding behind a provider trait supporting local (ONNX) and remote (OpenAI, Gemini) backends
- Implement batch embedding pipeline with configurable concurrency, rate limiting, and progress tracking
- Support OpenAI `text-embedding-3-small` and `text-embedding-3-large` via API
- Support Google Gemini `text-embedding-004` via API
- Keep local ONNX as the default (zero-config, offline-first)
- Add `arawn memory reindex` CLI command to batch-embed existing memories
- Support dimension configuration (384, 768, 1536) with automatic sqlite-vec schema adjustment

**Non-Goals:**
- Training or fine-tuning embedding models
- Multi-modal embeddings (image, audio) — future work
- Embedding model serving (we consume APIs, not serve them)
- Changing the vector database from sqlite-vec

## Detailed Design

### Embedding Provider Trait

```rust
// crates/arawn-memory/src/embedding.rs
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    fn name(&self) -> &str;
    fn dimensions(&self) -> usize;
    
    /// Embed a single text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    
    /// Batch embed multiple texts (default: sequential calls to embed)
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // Default implementation calls embed() sequentially
        // Providers override with native batch APIs
    }
    
    /// Max tokens per input text
    fn max_tokens(&self) -> usize;
}
```

### Providers

**`OnnxProvider`** (existing, refactored):
- Wraps current `ort` + `tokenizers` logic
- 384 dimensions (MiniLM-L6-v2)
- Batch: process in chunks of 32 using ONNX batch inference

**`OpenAiProvider`**:
- Uses `reqwest` to call `https://api.openai.com/v1/embeddings`
- Models: `text-embedding-3-small` (1536d), `text-embedding-3-large` (3072d)
- Supports `dimensions` parameter for reduced output (e.g., 512d)
- Batch: native batch API, up to 2048 inputs per request
- Rate limiting: configurable RPM/TPM limits

**`GeminiProvider`**:
- Uses `reqwest` to call Gemini embedding API
- Model: `text-embedding-004` (768d)
- Batch: native `batchEmbedContents` endpoint, up to 100 inputs
- Rate limiting: respect Gemini free tier limits

### Batch Pipeline

```rust
pub struct BatchEmbedder {
    provider: Arc<dyn EmbeddingProvider>,
    concurrency: usize,      // parallel batch requests
    chunk_size: usize,        // texts per batch call
    rate_limiter: RateLimiter,
    progress: Option<ProgressBar>,
}

impl BatchEmbedder {
    pub async fn embed_all(&self, texts: Vec<(MemoryId, String)>) -> Result<BatchReport> {
        // 1. Chunk texts into batches of chunk_size
        // 2. Process chunks with concurrency limit
        // 3. Rate limit between batches
        // 4. Store embeddings in MemoryStore
        // 5. Report: total, succeeded, failed, duration
    }
}
```

### Configuration

```toml
[embedding]
provider = "onnx"  # or "openai", "gemini"
model = "all-MiniLM-L6-v2"  # provider-specific model name
dimensions = 384

[embedding.batch]
concurrency = 4
chunk_size = 100
rate_limit_rpm = 3000  # requests per minute

[embedding.openai]
model = "text-embedding-3-small"
dimensions = 512  # reduced from 1536

[embedding.gemini]
model = "text-embedding-004"
```

### Dimension Migration

When switching providers with different dimensions, existing embeddings become incompatible. Handle this with:
1. Detect dimension mismatch on startup
2. Log warning: "Embedding dimensions changed (384 → 512). Run `arawn memory reindex` to re-embed."
3. `reindex` command: re-embeds all memories using current provider, drops old vector table, rebuilds

### CLI Commands

```
arawn memory reindex              # re-embed all memories with current provider
arawn memory reindex --dry-run    # show count and estimated cost
arawn memory stats                # show embedding stats (count, provider, dimensions)
```

## Alternatives Considered

- **Qdrant/Milvus instead of sqlite-vec**: Better performance at scale, but adds infrastructure dependency. sqlite-vec works well for single-user edge deployment. Revisit if we need multi-million vector scale.
- **Embedding as a separate microservice**: Overkill for single-user. In-process is simpler.
- **Only cloud providers**: Violates edge-first principle. Local ONNX must remain the default.
- **Automatic reindexing on provider change**: Too expensive to do silently. Explicit `reindex` command is safer and gives the user control over cost.

## Implementation Plan

1. Define `EmbeddingProvider` trait and refactor existing ONNX code into `OnnxProvider`
2. Implement `OpenAiProvider` with batch support and rate limiting
3. Implement `GeminiProvider` with batch support
4. Build `BatchEmbedder` pipeline with concurrency and progress
5. Add embedding configuration to `arawn-config`
6. Add `arawn memory reindex` and `arawn memory stats` CLI commands
7. Handle dimension migration (detection + reindex workflow)
8. Update `MemoryStore` to use provider trait instead of direct ONNX calls