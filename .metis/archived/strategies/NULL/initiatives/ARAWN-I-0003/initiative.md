---
id: arawn-llm-llm-client-and-embeddings
level: initiative
title: "arawn-llm: LLM Client and Embeddings"
short_code: "ARAWN-I-0003"
created_at: 2026-01-28T01:37:26.160522+00:00
updated_at: 2026-01-28T03:18:54.088453+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: arawn-llm-llm-client-and-embeddings
---

# arawn-llm: LLM Proxy and Embeddings

## Context

A unified LLM layer that acts as both a client (for the agent) and a proxy server (for external tools). Supports multiple providers with configurable routing, plus local embeddings for edge deployment.

Key capability: Claude OAuth proxy (muninn pattern) enables flat-rate billing with Claude Pro/Max subscriptions instead of per-token API costs.

## Goals & Non-Goals

**Goals:**
- Multi-provider support: Anthropic, OpenAI, Groq, Ollama (local)
- Claude OAuth proxy for flat-rate usage (muninn pattern)
- API key fallback when OAuth unavailable
- OpenAI-compatible proxy endpoint for external tools (Cursor, Continue, etc.)
- Configurable embeddings: local (all-MiniLM) or remote (OpenAI, Voyage)
- Request routing based on task complexity
- Streaming response support

**Non-Goals:**
- Fine-tuning or model training
- Token counting/billing (handled by providers)
- Caching responses (may add later)

## Detailed Design

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  arawn-llm                                                  │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  LlmClient (internal API for agent)                 │   │
│  │  - complete(messages, tools) -> Response            │   │
│  │  - stream(messages, tools) -> Stream<Chunk>         │   │
│  │  - embed(text) -> Vec<f32>                          │   │
│  └─────────────────────────────────────────────────────┘   │
│                              │                              │
│                              ▼                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Router                                              │   │
│  │  - Routes requests to appropriate provider           │   │
│  │  - Handles fallback on failure                       │   │
│  └─────────────────────────────────────────────────────┘   │
│                              │                              │
│         ┌────────────────────┼────────────────────┐        │
│         ▼                    ▼                    ▼        │
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────┐   │
│  │ Anthropic   │    │ OpenAI       │    │ Ollama      │   │
│  │ - API key   │    │ - API key    │    │ - Local     │   │
│  │ - OAuth     │    │              │    │             │   │
│  └─────────────┘    └──────────────┘    └─────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  ProxyServer (OpenAI-compatible endpoint)            │   │
│  │  POST /v1/chat/completions                          │   │
│  │  - External tools connect here                       │   │
│  │  - Routes through same LlmClient                     │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Embeddings                                          │   │
│  │  - Local: all-MiniLM-L6-v2 via ort (ONNX)           │   │
│  │  - Remote: OpenAI, Voyage (configurable)             │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### API Surface

```rust
// Client for agent use
pub struct LlmClient {
    config: LlmConfig,
    providers: HashMap<ProviderId, Box<dyn Provider>>,
    embedder: Box<dyn Embedder>,
}

impl LlmClient {
    pub async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
    pub async fn stream(&self, request: CompletionRequest) -> Result<impl Stream<Item = Chunk>>;
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    pub async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
}

// Provider trait
#[async_trait]
pub trait Provider: Send + Sync {
    async fn complete(&self, request: ProviderRequest) -> Result<ProviderResponse>;
    async fn stream(&self, request: ProviderRequest) -> Result<Box<dyn Stream<Item = Chunk>>>;
    fn supports_tools(&self) -> bool;
    fn model_info(&self) -> ModelInfo;
}

// Embedder trait
#[async_trait]
pub trait Embedder: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    fn dimensions(&self) -> usize;
}

// Configuration
pub struct LlmConfig {
    pub default_provider: ProviderId,
    pub providers: HashMap<ProviderId, ProviderConfig>,
    pub embeddings: EmbeddingConfig,
    pub routing: RoutingConfig,
}

pub enum ProviderConfig {
    Anthropic { api_key: Option<String>, oauth: Option<OAuthConfig> },
    OpenAI { api_key: String, base_url: Option<String> },
    Ollama { base_url: String },
    Groq { api_key: String },
}
```

### OAuth Flow (Claude Pro/Max)

```rust
// muninn-style OAuth for flat-rate Claude usage
pub struct ClaudeOAuth {
    tokens: TokenStore,
}

impl ClaudeOAuth {
    pub async fn authenticate(&self) -> Result<()>;      // Browser-based OAuth flow
    pub async fn refresh_if_needed(&self) -> Result<()>; // Auto-refresh tokens
    pub fn is_authenticated(&self) -> bool;
}
```

### Proxy Server

```rust
// OpenAI-compatible endpoint for external tools
pub struct ProxyServer {
    client: Arc<LlmClient>,
    auth: ProxyAuth,
}

impl ProxyServer {
    // POST /v1/chat/completions
    pub async fn chat_completions(&self, req: ChatRequest) -> Result<ChatResponse>;
    
    // POST /v1/embeddings
    pub async fn embeddings(&self, req: EmbedRequest) -> Result<EmbedResponse>;
}
```

### Local Embeddings

```rust
// ONNX runtime for local inference
pub struct LocalEmbedder {
    session: ort::Session,
    tokenizer: Tokenizer,
}

impl LocalEmbedder {
    pub fn load_minilm() -> Result<Self>;  // all-MiniLM-L6-v2, 384 dims
    // Could add larger models for 8GB+ systems
}
```

### Dependencies

```toml
[dependencies]
reqwest = { version = "0.12", features = ["json", "stream"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
async-trait = "0.1"
futures = "0.3"

# Local embeddings
ort = "2"
tokenizers = "0.20"

# OAuth
oauth2 = "4"
keyring = "2"  # Secure token storage
```

## Alternatives Considered

- **Claude API only**: Rejected - want flexibility for different providers and cost optimization
- **No local embeddings**: Rejected - edge deployment needs offline capability
- **Separate proxy service**: Rejected - single binary goal; proxy is just another endpoint

## Implementation Plan

1. Provider trait and Anthropic implementation (API key)
2. OpenAI provider
3. Ollama provider (local models)
4. Claude OAuth flow (muninn pattern)
5. Router with fallback logic
6. Local embeddings (ort + all-MiniLM)
7. Proxy server endpoint
8. Remote embeddings (OpenAI, Voyage)