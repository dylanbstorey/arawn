//! Types for the RLM (Recursive Language Model) exploration module.

/// Configuration for an RLM exploration run.
#[derive(Debug, Clone)]
pub struct RlmConfig {
    /// Model to use for exploration (e.g., "claude-sonnet-4-20250514").
    pub model: String,
    /// Max iterations per agent turn (should be 1 for orchestrator control).
    pub max_iterations_per_turn: u32,
    /// Cumulative token budget for the entire exploration.
    pub max_total_tokens: Option<usize>,
    /// Maximum estimated tokens before triggering compaction.
    pub max_context_tokens: usize,
    /// Fraction of `max_context_tokens` that triggers compaction (0.0–1.0).
    pub compaction_threshold: f32,
    /// Maximum compaction cycles before stopping.
    pub max_compactions: u32,
    /// Maximum agent turns before stopping (safety valve).
    pub max_turns: u32,
    /// Optional separate model for compaction (cheaper/faster).
    pub compaction_model: Option<String>,
    /// Optional custom compaction prompt.
    pub compaction_prompt: Option<String>,
}

impl Default for RlmConfig {
    fn default() -> Self {
        Self {
            model: String::new(), // Inherited from the backend
            max_iterations_per_turn: 1,
            max_total_tokens: None,
            max_context_tokens: 50_000,
            compaction_threshold: 0.7,
            max_compactions: 10,
            max_turns: 50,
            compaction_model: None,
            compaction_prompt: None,
        }
    }
}

/// Result of an RLM exploration run.
#[derive(Debug, Clone)]
pub struct ExplorationResult {
    /// The final summary/answer produced by the exploration.
    pub summary: String,
    /// Whether the exploration was truncated (budget or compaction limit hit).
    pub truncated: bool,
    /// Metadata about the exploration run.
    pub metadata: ExplorationMetadata,
}

/// Metadata from an RLM exploration run.
#[derive(Debug, Clone)]
pub struct ExplorationMetadata {
    /// Total LLM iterations across all compaction cycles.
    pub iterations_used: u32,
    /// Total input tokens consumed.
    pub input_tokens: u32,
    /// Total output tokens generated.
    pub output_tokens: u32,
    /// Number of compaction cycles performed.
    pub compactions_performed: u32,
    /// Model used for exploration.
    pub model_used: String,
}

impl ExplorationMetadata {
    /// Total tokens used (input + output).
    pub fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
}
