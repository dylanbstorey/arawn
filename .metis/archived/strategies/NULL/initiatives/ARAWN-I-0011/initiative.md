---
id: llm-router-multi-backend-request
level: initiative
title: "LLM Router: Multi-Backend Request Routing"
short_code: "ARAWN-I-0011"
created_at: 2026-01-28T20:57:55.418117+00:00
updated_at: 2026-01-29T03:38:31.474247+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: llm-router-multi-backend-request
---

# Multi-Backend Support and Interaction Logging

## Context

Arawn currently picks a single LLM backend at startup. We need the ability to configure multiple named backends and select between them. Originally scoped as a full request-routing system with heuristic classification, the scope was narrowed after architectural discussion:

- **Routing binds to workstreams** (persistent conversation contexts), not to individual requests
- Per-request heuristic classification is unnecessary — workstreams set their default model via config
- Agent/skill-level routing will come later as manual config
- The heuristic classifier and `RoutingBackend` wrapper are deferred

What remains: shared interaction logging infrastructure (for training data, debugging, analytics) and multi-backend wiring so that workstreams can select which backend to use.

## Goals & Non-Goals

**Goals:**
- **Interaction log infrastructure**: Shared structured capture of all LLM interactions (request, response, metadata) in JSONL format for training, debugging, and analytics
- **Multi-backend wiring**: Ability to configure multiple named LLM backends and instantiate them at startup
- **Backend selection by name**: Workstreams (and eventually agents/skills) can specify which backend to use by profile name

**Non-Goals (deferred):**
- Per-request heuristic routing (routing binds to workstreams, not requests)
- RoutingBackend wrapper / RoutingPolicy trait (deferred until workstream architecture is built)
- Embedding-based classifier
- Cost tracking/budgets
- Automatic fallback on backend failure

## Detailed Design

### Interaction Log Infrastructure

A shared structured log of every LLM interaction, designed for training data export and analysis. Replaces the ad-hoc TRACE-level logging in agent.rs with a proper store.

```rust
// crates/arawn-llm/src/interaction_log.rs

/// A complete record of one LLM request/response cycle
#[derive(Debug, Serialize, Deserialize)]
pub struct InteractionRecord {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
    
    // Context
    pub session_id: Option<String>,
    pub turn_id: Option<u32>,
    pub iteration: u32,           // which iteration in the tool-use loop
    
    // Request
    pub messages: Vec<Message>,   // full message history sent to LLM
    pub system_prompt: Option<String>,
    pub tools_available: Vec<String>,  // tool names
    pub tool_count: usize,
    pub model_requested: String,
    
    // Response
    pub model_used: String,       // actual model (may differ from requested)
    pub response_content: Vec<ContentBlock>,
    pub stop_reason: StopReason,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub has_tool_use: bool,
    pub tool_calls: Vec<ToolCallRecord>,
    
    // Routing (populated by router, None if no routing)
    pub routing: Option<RoutingMetadata>,
    
    // Tags for filtering/querying
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub tool_name: String,
    pub tool_call_id: String,
    pub arguments: Value,
    pub result_success: Option<bool>,     // filled in after execution
    pub result_content: Option<String>,   // truncated
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoutingMetadata {
    pub profile: String,
    pub reason: String,
    pub confidence: f32,
    pub features: RequestFeatures,
}
```

**Storage**: JSONL files in `~/.config/arawn/interactions/`. One file per day, matching the existing log rotation pattern. JSONL is easy to parse, grep, and feed into training pipelines.

```rust
pub struct InteractionLogger {
    writer: Arc<Mutex<BufWriter<File>>>,
    enabled: bool,
}

impl InteractionLogger {
    /// Log a complete interaction record
    pub fn log(&self, record: &InteractionRecord) {
        if !self.enabled { return; }
        let json = serde_json::to_string(record).unwrap();
        // Write to JSONL file
        writeln!(self.writer.lock(), "{}", json);
        
        // Also emit structured tracing event for console/log visibility
        tracing::info!(
            target: "arawn::interactions",
            id = %record.id,
            session = ?record.session_id,
            model = %record.model_used,
            input_tokens = record.input_tokens,
            output_tokens = record.output_tokens,
            tool_calls = record.tool_calls.len(),
            duration_ms = record.duration_ms,
            routing_profile = record.routing.as_ref().map(|r| r.profile.as_str()).unwrap_or("none"),
            "LLM interaction"
        );
    }
}
```

**Configuration**:
```toml
[logging.interactions]
enabled = true
path = "~/.config/arawn/interactions/"   # default
retention_days = 90                       # auto-cleanup old files
include_messages = true                   # full message content (large!)
include_responses = true                  # full response content
truncate_tool_results = 2048             # max chars for tool result content
```

**Query CLI** (future, but the format supports it):
```
arawn interactions list --date 2026-01-28          # list day's interactions
arawn interactions export --format parquet          # export for training
arawn interactions stats                            # token usage, model distribution
arawn interactions search --tag routing_profile=fast # filter by tag
```

The agent turn loop, routing backend, and session indexer all write to the same `InteractionLogger`. This replaces the existing TRACE-level message/response logging in agent.rs with structured, queryable records.

### RoutingBackend

```rust
// crates/arawn-llm/src/routing.rs
pub struct RoutingBackend {
    backends: HashMap<String, Arc<dyn LlmBackend>>,
    policy: Box<dyn RoutingPolicy>,
    default_profile: String,
    logger: RoutingLogger,
}

#[async_trait]
impl LlmBackend for RoutingBackend {
    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let decision = self.policy.route(request);
        self.logger.log_decision(&decision, request);
        
        let backend = self.backends.get(&decision.profile)
            .unwrap_or_else(|| self.backends.get(&self.default_profile).unwrap());
        
        let response = backend.complete(request).await?;
        self.logger.log_outcome(&decision, &response);
        
        Ok(response)
    }
}
```

### RoutingPolicy Trait

```rust
pub trait RoutingPolicy: Send + Sync {
    fn route(&self, request: &CompletionRequest) -> RoutingDecision;
}

pub struct RoutingDecision {
    pub profile: String,           // e.g., "fast", "capable", "default"
    pub reason: RoutingReason,     // why this profile was chosen
    pub confidence: f32,           // 0.0-1.0, how sure the router is
    pub features: RequestFeatures, // extracted features (for logging)
}

pub enum RoutingReason {
    CallerHint(String),        // explicit profile requested
    HeuristicRule(String),     // which rule matched
    EmbeddingClassifier(f32),  // similarity score (v2)
    Default,                   // no rule matched, using default
}
```

### Layered Policy (v1: Heuristics)

```rust
pub struct LayeredPolicy {
    rules: Vec<RoutingRule>,
    default_profile: String,
}

impl RoutingPolicy for LayeredPolicy {
    fn route(&self, request: &CompletionRequest) -> RoutingDecision {
        // Layer 1: Caller hint (highest priority)
        if let Some(hint) = request.metadata.get("routing_profile") {
            return RoutingDecision::from_hint(hint);
        }
        
        // Layer 2: Heuristic rules
        let features = extract_features(request);
        for rule in &self.rules {
            if rule.matches(&features) {
                return RoutingDecision::from_rule(rule, &features);
            }
        }
        
        // Layer 3: Default
        RoutingDecision::default(&self.default_profile, &features)
    }
}
```

### Request Feature Extraction

```rust
pub struct RequestFeatures {
    pub message_length: usize,         // character count of last user message
    pub message_count: usize,          // messages in conversation
    pub has_tools: bool,               // tools attached to request
    pub tool_count: usize,             // number of tools available
    pub has_system_prompt: bool,       // custom system prompt present
    pub keyword_signals: Vec<String>,  // detected keywords suggesting complexity
    pub estimated_complexity: Complexity, // Simple / Moderate / Complex
}

pub enum Complexity {
    Simple,    // short Q&A, lookups, simple formatting
    Moderate,  // summarization, translation, short generation
    Complex,   // code generation, multi-step reasoning, tool-heavy
}

fn extract_features(request: &CompletionRequest) -> RequestFeatures {
    let last_msg = request.messages.last().map(|m| m.content_text()).unwrap_or("");
    
    let complexity = if request.tools.len() > 3 || last_msg.len() > 2000 {
        Complexity::Complex
    } else if last_msg.len() > 500 || contains_complexity_keywords(last_msg) {
        Complexity::Moderate
    } else {
        Complexity::Simple
    };
    
    // ... extract all features
}

fn contains_complexity_keywords(text: &str) -> bool {
    const COMPLEX_KEYWORDS: &[&str] = &[
        "analyze", "implement", "refactor", "debug", "architect",
        "compare", "evaluate", "design", "optimize", "explain why",
        "step by step", "write code", "fix the bug",
    ];
    COMPLEX_KEYWORDS.iter().any(|k| text.to_lowercase().contains(k))
}
```

### Routing Rules Config

```toml
[routing]
default = "capable"     # fallback profile
enabled = true          # false = always use default

# Backend profiles
[routing.profiles.fast]
backend = "groq"
model = "llama-3.3-70b-versatile"

[routing.profiles.capable]
backend = "claude-oauth"
model = "claude-sonnet-4-20250514"

# Rules evaluated in order, first match wins
[[routing.rules]]
name = "simple-questions"
condition = { complexity = "simple", has_tools = false }
profile = "fast"

[[routing.rules]]
name = "tool-heavy"
condition = { has_tools = true, tool_count_gt = 3 }
profile = "capable"

[[routing.rules]]
name = "long-context"
condition = { message_length_gt = 2000 }
profile = "capable"
```

### Routing → Interaction Log Integration

The `RoutingBackend` populates the `RoutingMetadata` field on each `InteractionRecord` written by the shared `InteractionLogger`. No separate routing log — everything lives in the interaction log:

```rust
impl RoutingBackend {
    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let decision = self.policy.route(request);
        let backend = self.backends.get(&decision.profile).unwrap();
        
        let start = Instant::now();
        let response = backend.complete(request).await?;
        let duration = start.elapsed();
        
        // Write to shared interaction log with routing metadata attached
        self.interaction_logger.log(&InteractionRecord {
            routing: Some(RoutingMetadata {
                profile: decision.profile.clone(),
                reason: format!("{:?}", decision.reason),
                confidence: decision.confidence,
                features: decision.features.clone(),
            }),
            duration_ms: duration.as_millis() as u64,
            // ... rest populated from request/response
            ..InteractionRecord::from_exchange(request, &response)
        });
        
        Ok(response)
    }
}
```

A future training pipeline can:
1. Read JSONL interaction logs
2. Filter for records with `routing` metadata present
3. Pair routing features with outcomes (tokens, tool_use, stop_reason)
4. Label: was the routing decision correct for this request class?
5. Train embedding classifier on (request_text, features) → profile mapping

### Path to Embedding Classifier (v2)

Once enough training data exists:

```rust
pub struct EmbeddingClassifierPolicy {
    embedder: Arc<dyn EmbeddingProvider>,  // from ARAWN-I-0015
    prototypes: HashMap<String, Vec<f32>>, // profile → centroid embedding
    heuristic_fallback: LayeredPolicy,     // fallback when confidence is low
    confidence_threshold: f32,             // below this, use heuristic
}

impl RoutingPolicy for EmbeddingClassifierPolicy {
    fn route(&self, request: &CompletionRequest) -> RoutingDecision {
        // Caller hint still takes priority
        if let Some(hint) = request.metadata.get("routing_profile") {
            return RoutingDecision::from_hint(hint);
        }
        
        // Embed the request, find nearest prototype
        let embedding = self.embedder.embed_sync(&request.last_message_text());
        let (best_profile, similarity) = self.nearest_prototype(&embedding);
        
        if similarity >= self.confidence_threshold {
            RoutingDecision::from_classifier(best_profile, similarity, &features)
        } else {
            // Low confidence, fall back to heuristics
            self.heuristic_fallback.route(request)
        }
    }
}
```

This is a future enhancement — v1 ships with heuristics only.

## Alternatives Considered

- **SLM for classification (TinyLlama, Phi-3)**: Generative model prompted to classify. Works but adds seconds of latency on CPU. Classification doesn't need generation — embedding similarity or heuristics are faster and sufficient.
- **LLM-in-the-loop routing**: Ask Claude to classify the request before routing. Defeats the cost savings purpose entirely.
- **Static config only (no analysis)**: User manually sets backend per-request or globally. Too coarse — the value is automatic routing based on request complexity.
- **Response quality feedback loop**: Route, observe quality, adjust. Interesting but hard to define "quality" automatically. Defer to future initiative.
- **Cost-based routing**: Route to stay within a budget. Requires cost tracking infrastructure that doesn't exist yet. Future initiative.

## Implementation Plan

1. **Interaction log infrastructure**: `InteractionRecord`, `InteractionLogger`, JSONL writer, config section — shared by all consumers
2. Wire `InteractionLogger` into agent turn loop, replacing ad-hoc TRACE logging in agent.rs
3. Define `RoutingPolicy` trait, `RoutingDecision`, `RequestFeatures` in `arawn-llm`
4. Implement `LayeredPolicy` with heuristic rules and feature extraction
5. Implement `RoutingBackend` wrapping multiple `LlmBackend` instances
6. Add `[routing]` config section with profiles and rules to `arawn-config`
7. `RoutingBackend` writes `InteractionRecord` with `RoutingMetadata` populated
8. Wire `RoutingBackend` into `start.rs` (replace single-backend creation)
9. Add `routing_profile` field to `ChatRequest` in server API
10. Integration tests with mock backends verifying routing decisions
11. (v2, future) Embedding classifier using logged training data