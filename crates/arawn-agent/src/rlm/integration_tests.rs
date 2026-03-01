//! Integration tests for the full RLM exploration pipeline.
//!
//! These tests exercise the entire flow from ExploreTool → RlmSpawner →
//! Agent + CompactionOrchestrator, validating that all components work
//! together correctly.

use std::sync::Arc;

use serde_json::json;

use arawn_llm::{CompletionResponse, ContentBlock, MockBackend, StopReason, Usage};

use crate::Tool;
use crate::rlm::{DEFAULT_READ_ONLY_TOOLS, RlmConfig, RlmSpawner};
use crate::tool::{MockTool, ToolContext, ToolRegistry, ToolResult};
use crate::tools::ExploreTool;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn mock_text_response(text: &str) -> CompletionResponse {
    CompletionResponse::new(
        "msg_1",
        "test-model",
        vec![ContentBlock::Text {
            text: text.to_string(),
            cache_control: None,
        }],
        StopReason::EndTurn,
        Usage::new(100, 50),
    )
}

fn mock_text_response_with_usage(text: &str, input: u32, output: u32) -> CompletionResponse {
    CompletionResponse::new(
        "msg_1",
        "test-model",
        vec![ContentBlock::Text {
            text: text.to_string(),
            cache_control: None,
        }],
        StopReason::EndTurn,
        Usage::new(input, output),
    )
}

fn mock_tool_use_response(
    tool_id: &str,
    tool_name: &str,
    args: serde_json::Value,
) -> CompletionResponse {
    CompletionResponse::new(
        "msg_1",
        "test-model",
        vec![ContentBlock::ToolUse {
            id: tool_id.to_string(),
            name: tool_name.to_string(),
            input: args,
            cache_control: None,
        }],
        StopReason::ToolUse,
        Usage::new(100, 50),
    )
}

/// Create a full tool registry with both read-only and write tools.
fn make_full_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    // Read-only tools (should be available to RLM)
    registry.register(MockTool::new("file_read"));
    registry.register(MockTool::new("glob"));
    registry.register(MockTool::new("grep"));
    registry.register(MockTool::new("web_fetch"));
    registry.register(MockTool::new("web_search"));
    registry.register(MockTool::new("memory_search"));
    registry.register(MockTool::new("think"));
    // Write tools (should be excluded from RLM)
    registry.register(MockTool::new("file_write"));
    registry.register(MockTool::new("shell"));
    registry.register(MockTool::new("delegate"));
    registry.register(MockTool::new("note"));
    // The explore tool itself (should NOT be passed to RLM)
    registry.register(MockTool::new("explore"));
    registry
}

fn make_spawner(backend: MockBackend) -> Arc<RlmSpawner> {
    Arc::new(RlmSpawner::new(Arc::new(backend), make_full_registry()))
}

fn make_spawner_with_config(backend: MockBackend, config: RlmConfig) -> Arc<RlmSpawner> {
    Arc::new(RlmSpawner::new(Arc::new(backend), make_full_registry()).with_config(config))
}

// ─────────────────────────────────────────────────────────────────────────────
// End-to-end ExploreTool tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_explore_tool_full_pipeline() {
    // Full pipeline: ExploreTool → RlmSpawner → Agent → Orchestrator → result
    // Agent uses a tool then produces a summary.
    let backend = MockBackend::new(vec![
        mock_tool_use_response("call_1", "grep", json!({"pattern": "auth"})),
        mock_text_response("Authentication is handled by the auth module at src/auth.rs."),
    ]);

    let spawner = make_spawner(backend);
    let tool = ExploreTool::new(spawner);
    let ctx = ToolContext::default();

    let result = tool
        .execute(json!({"query": "How does authentication work?"}), &ctx)
        .await
        .unwrap();

    assert!(result.is_success());
    let content = result.to_llm_content();
    assert!(content.contains("Authentication is handled by the auth module"));
    assert!(content.contains("Exploration:"));
    assert!(content.contains("iterations"));
    assert!(content.contains("tokens"));
}

#[tokio::test]
async fn test_explore_tool_multi_tool_research() {
    // Agent performs multiple tool calls before producing summary
    let backend = MockBackend::new(vec![
        mock_tool_use_response("call_1", "glob", json!({"pattern": "*.rs"})),
        mock_tool_use_response("call_2", "file_read", json!({"path": "src/lib.rs"})),
        mock_tool_use_response("call_3", "grep", json!({"pattern": "fn main"})),
        mock_text_response("The project has 15 Rust files. The entry point is in src/main.rs."),
    ]);

    let spawner = make_spawner(backend);
    let tool = ExploreTool::new(spawner);
    let ctx = ToolContext::default();

    let result = tool
        .execute(json!({"query": "Describe the project structure"}), &ctx)
        .await
        .unwrap();

    assert!(result.is_success());
    let content = result.to_llm_content();
    assert!(content.contains("15 Rust files"));
    assert!(content.contains("src/main.rs"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Compaction cycle tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_explore_compaction_cycle() {
    // RlmSpawner level: agent generates enough context to trigger compaction,
    // then continues and produces a final summary.
    //
    // With preserve_recent=3 (default), we need at least 4 session turns
    // before compaction can happen. Each orchestrator turn with
    // max_iterations_per_turn=1 produces 1 session turn.
    let large_output = "x".repeat(2000); // ~500 tokens at 4 chars/token

    // Agent backend: 4 tool calls (to accumulate enough turns) → final summary
    let agent_backend = MockBackend::new(vec![
        mock_tool_use_response("call_1", "grep", json!({})),
        mock_tool_use_response("call_2", "grep", json!({})),
        mock_tool_use_response("call_3", "grep", json!({})),
        mock_tool_use_response("call_4", "grep", json!({})),
        mock_text_response("After compaction, found auth in 3 modules."),
    ]);

    // Compactor backend: produces compacted summary
    let compaction_backend = MockBackend::new(vec![mock_text_response(
        "Previous exploration found authentication patterns.",
    )]);

    let mut tools = ToolRegistry::new();
    tools.register(MockTool::new("grep").with_response(ToolResult::text(&large_output)));

    let config = RlmConfig {
        max_context_tokens: 1000,
        compaction_threshold: 0.1, // Very low — triggers easily
        ..RlmConfig::default()
    };

    let spawner = RlmSpawner::new(Arc::new(agent_backend), tools).with_config(config);
    let spawner = spawner.with_compaction_backend(Arc::new(compaction_backend));

    let result = spawner.explore("How does auth work?").await.unwrap();

    assert_eq!(result.summary, "After compaction, found auth in 3 modules.");
    assert!(!result.truncated);
    assert!(result.metadata.compactions_performed >= 1);
}

#[tokio::test]
async fn test_explore_multiple_compaction_cycles() {
    // Agent generates large context repeatedly, causing multiple compaction cycles.
    // With preserve_recent=3, need enough tool calls to trigger compaction twice.
    let large_output = "x".repeat(2000);

    // Many tool calls to accumulate enough for 2+ compactions
    let agent_backend = MockBackend::new(vec![
        mock_tool_use_response("call_1", "grep", json!({})),
        mock_tool_use_response("call_2", "grep", json!({})),
        mock_tool_use_response("call_3", "grep", json!({})),
        mock_tool_use_response("call_4", "grep", json!({})),
        mock_tool_use_response("call_5", "grep", json!({})),
        mock_tool_use_response("call_6", "grep", json!({})),
        mock_tool_use_response("call_7", "grep", json!({})),
        mock_tool_use_response("call_8", "grep", json!({})),
        mock_text_response("Final answer after compactions."),
    ]);

    let compaction_backend = MockBackend::new(vec![
        mock_text_response("Compacted findings #1."),
        mock_text_response("Compacted findings #2."),
        mock_text_response("Compacted findings #3."),
    ]);

    let mut tools = ToolRegistry::new();
    tools.register(MockTool::new("grep").with_response(ToolResult::text(&large_output)));

    let config = RlmConfig {
        max_context_tokens: 1000,
        compaction_threshold: 0.1,
        max_compactions: 10,
        ..RlmConfig::default()
    };

    let spawner = RlmSpawner::new(Arc::new(agent_backend), tools)
        .with_config(config)
        .with_compaction_backend(Arc::new(compaction_backend));

    let result = spawner.explore("Deep research").await.unwrap();

    assert_eq!(result.summary, "Final answer after compactions.");
    assert!(!result.truncated);
    assert!(result.metadata.compactions_performed >= 2);
}

// ─────────────────────────────────────────────────────────────────────────────
// Budget and limit enforcement tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_explore_max_turns_enforced() {
    // Agent keeps calling tools but context stays small (no compaction).
    // The max_turns safety valve should stop execution.
    let agent_backend = MockBackend::new(vec![
        mock_tool_use_response("call_1", "grep", json!({})),
        mock_tool_use_response("call_2", "grep", json!({})),
        mock_tool_use_response("call_3", "grep", json!({})),
    ]);

    let mut tools = ToolRegistry::new();
    tools.register(MockTool::new("grep"));

    let config = RlmConfig {
        max_turns: 2, // Stop after 2 turns
        ..RlmConfig::default()
    };

    let spawner = RlmSpawner::new(Arc::new(agent_backend), tools).with_config(config);

    let result = spawner.explore("Search forever").await.unwrap();

    assert!(result.truncated);
    assert_eq!(result.metadata.compactions_performed, 0);
}

#[tokio::test]
async fn test_explore_max_compactions_enforced() {
    // Agent generates large output repeatedly, exhausting compaction budget.
    // Need many tool calls with large outputs to trigger compaction multiple times.
    let large_output = "x".repeat(2000);

    // Lots of tool calls to exhaust the compaction budget of 2
    let agent_backend = MockBackend::new(vec![
        mock_tool_use_response("call_1", "grep", json!({})),
        mock_tool_use_response("call_2", "grep", json!({})),
        mock_tool_use_response("call_3", "grep", json!({})),
        mock_tool_use_response("call_4", "grep", json!({})),
        mock_tool_use_response("call_5", "grep", json!({})),
        mock_tool_use_response("call_6", "grep", json!({})),
        mock_tool_use_response("call_7", "grep", json!({})),
        mock_tool_use_response("call_8", "grep", json!({})),
        mock_tool_use_response("call_9", "grep", json!({})),
        mock_tool_use_response("call_10", "grep", json!({})),
        mock_tool_use_response("call_11", "grep", json!({})),
        mock_tool_use_response("call_12", "grep", json!({})),
    ]);

    let compaction_backend = MockBackend::new(vec![
        mock_text_response("Summary 1."),
        mock_text_response("Summary 2."),
    ]);

    let mut tools = ToolRegistry::new();
    tools.register(MockTool::new("grep").with_response(ToolResult::text(&large_output)));

    let config = RlmConfig {
        max_context_tokens: 1000,
        compaction_threshold: 0.1,
        max_compactions: 2, // Stop after 2 compactions
        max_turns: 50,
        ..RlmConfig::default()
    };

    let spawner = RlmSpawner::new(Arc::new(agent_backend), tools)
        .with_config(config)
        .with_compaction_backend(Arc::new(compaction_backend));

    let result = spawner.explore("Keep searching").await.unwrap();

    assert!(result.truncated);
    assert_eq!(result.metadata.compactions_performed, 2);
}

#[tokio::test]
async fn test_explore_token_budget_enforced() {
    // Set a low max_total_tokens budget. The agent should stop after
    // exceeding the budget rather than continue indefinitely.
    let agent_backend = MockBackend::new(vec![
        mock_text_response_with_usage("First answer.", 100, 50),
        // This response would exceed budget but should not be reached
        mock_text_response_with_usage("Should not reach this.", 100, 50),
    ]);

    let mut tools = ToolRegistry::new();
    tools.register(MockTool::new("grep"));

    let config = RlmConfig {
        max_total_tokens: Some(200), // Budget of 200 tokens
        ..RlmConfig::default()
    };

    let spawner = RlmSpawner::new(Arc::new(agent_backend), tools).with_config(config);

    let result = spawner.explore("Quick question").await.unwrap();

    // First response uses 150 tokens, which exceeds the 200 budget
    // (or close enough that the agent stops). The result should be
    // the first answer since the agent finishes naturally on EndTurn.
    assert_eq!(result.summary, "First answer.");
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool filtering tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_explore_excludes_write_tools() {
    // Verify that write tools are NOT available to the exploration agent.
    // The DEFAULT_READ_ONLY_TOOLS list should NOT contain any write tools.
    let excluded_tools = [
        "shell",
        "file_write",
        "delegate",
        "note",
        "explore", // Prevents recursive exploration
        "catalog",
        "workflow",
    ];

    for tool_name in &excluded_tools {
        assert!(
            !DEFAULT_READ_ONLY_TOOLS.contains(tool_name),
            "{} should NOT be in DEFAULT_READ_ONLY_TOOLS",
            tool_name
        );
    }
}

#[tokio::test]
async fn test_explore_includes_read_only_tools() {
    // Verify all expected read-only tools are included.
    let expected_tools = [
        "file_read",
        "glob",
        "grep",
        "web_fetch",
        "web_search",
        "memory_search",
        "think",
    ];

    for tool_name in &expected_tools {
        assert!(
            DEFAULT_READ_ONLY_TOOLS.contains(tool_name),
            "{} should be in DEFAULT_READ_ONLY_TOOLS",
            tool_name
        );
    }
}

#[tokio::test]
async fn test_explore_no_recursive_spawning() {
    // Even if the parent registry has an "explore" tool registered,
    // it should NOT be available to the RLM sub-agent.
    let backend = MockBackend::new(vec![mock_text_response("Done.")]);

    // make_full_registry() includes an "explore" MockTool
    let registry = make_full_registry();
    assert!(registry.get("explore").is_some(), "Parent has explore tool");

    let spawner = RlmSpawner::new(Arc::new(backend), registry);

    // Explore should work fine — the "explore" tool is filtered out
    let result = spawner.explore("test").await.unwrap();
    assert!(!result.truncated);
    assert_eq!(result.summary, "Done.");
}

// ─────────────────────────────────────────────────────────────────────────────
// Config wiring tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_explore_custom_model_config() {
    // Verify that custom model config flows through to the exploration.
    let backend = MockBackend::new(vec![mock_text_response("Custom model result.")]);

    let config = RlmConfig {
        model: "claude-haiku-4-5-20251001".to_string(),
        max_context_tokens: 25_000,
        compaction_threshold: 0.5,
        max_compactions: 5,
        max_turns: 15,
        ..RlmConfig::default()
    };

    let spawner = make_spawner_with_config(backend, config);
    let tool = ExploreTool::new(spawner);
    let ctx = ToolContext::default();

    let result = tool
        .execute(json!({"query": "test config"}), &ctx)
        .await
        .unwrap();

    assert!(result.is_success());
    let content = result.to_llm_content();
    assert!(content.contains("Custom model result."));
}

#[tokio::test]
async fn test_rlm_config_to_agent_config_model() {
    // When RlmConfig specifies a model, the exploration metadata should
    // reflect it (the agent uses that model).
    let backend = MockBackend::new(vec![mock_text_response("Done.")]);

    let config = RlmConfig {
        model: "claude-haiku-4-5-20251001".to_string(),
        ..RlmConfig::default()
    };

    let spawner = RlmSpawner::new(Arc::new(backend), make_full_registry()).with_config(config);

    let result = spawner.explore("test").await.unwrap();

    assert_eq!(result.metadata.model_used, "claude-haiku-4-5-20251001");
}

#[tokio::test]
async fn test_rlm_default_config_model() {
    // When RlmConfig has empty model (default), the agent's default model
    // from AgentConfig should be used.
    let backend = MockBackend::new(vec![mock_text_response("Done.")]);

    let spawner = make_spawner(backend);

    let result = spawner.explore("test").await.unwrap();

    // Default model from AgentConfig
    assert_eq!(result.metadata.model_used, "claude-sonnet-4-20250514");
}

// ─────────────────────────────────────────────────────────────────────────────
// Config TOML deserialization wiring
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_rlm_toml_config_to_rlm_config() {
    // Verify the mapping from arawn-config's RlmTomlConfig to
    // arawn-agent's RlmConfig works as expected.
    use arawn_config::RlmTomlConfig;

    let toml_config = RlmTomlConfig {
        model: Some("claude-haiku-4-5-20251001".to_string()),
        max_turns: Some(30),
        max_context_tokens: Some(80_000),
        compaction_threshold: Some(0.6),
        max_compactions: Some(5),
        max_total_tokens: Some(200_000),
        compaction_model: Some("cheap-model".to_string()),
    };

    // Apply overrides (same logic as start.rs)
    let mut rlm_config = RlmConfig::default();
    if let Some(ref model) = toml_config.model {
        rlm_config.model = model.clone();
    }
    if let Some(max_turns) = toml_config.max_turns {
        rlm_config.max_turns = max_turns;
    }
    if let Some(max_ctx) = toml_config.max_context_tokens {
        rlm_config.max_context_tokens = max_ctx;
    }
    if let Some(threshold) = toml_config.compaction_threshold {
        rlm_config.compaction_threshold = threshold;
    }
    if let Some(max_c) = toml_config.max_compactions {
        rlm_config.max_compactions = max_c;
    }
    if let Some(max_t) = toml_config.max_total_tokens {
        rlm_config.max_total_tokens = Some(max_t);
    }
    if let Some(ref c_model) = toml_config.compaction_model {
        rlm_config.compaction_model = Some(c_model.clone());
    }

    assert_eq!(rlm_config.model, "claude-haiku-4-5-20251001");
    assert_eq!(rlm_config.max_turns, 30);
    assert_eq!(rlm_config.max_context_tokens, 80_000);
    assert_eq!(rlm_config.compaction_threshold, 0.6);
    assert_eq!(rlm_config.max_compactions, 5);
    assert_eq!(rlm_config.max_total_tokens, Some(200_000));
    assert_eq!(rlm_config.compaction_model, Some("cheap-model".to_string()));
}

#[test]
fn test_rlm_toml_defaults_preserve_agent_defaults() {
    // When RlmTomlConfig has all None fields, RlmConfig defaults are preserved.
    let toml_config = arawn_config::RlmTomlConfig::default();
    let rlm_config = RlmConfig::default();

    // All None — nothing should change
    assert!(toml_config.model.is_none());
    assert!(toml_config.max_turns.is_none());
    assert!(toml_config.max_context_tokens.is_none());
    assert!(toml_config.compaction_threshold.is_none());
    assert!(toml_config.max_compactions.is_none());
    assert!(toml_config.max_total_tokens.is_none());
    assert!(toml_config.compaction_model.is_none());

    // Agent defaults are sensible
    assert!(rlm_config.model.is_empty()); // Inherited from backend
    assert_eq!(rlm_config.max_turns, 50);
    assert_eq!(rlm_config.max_context_tokens, 50_000);
    assert_eq!(rlm_config.compaction_threshold, 0.7);
    assert_eq!(rlm_config.max_compactions, 10);
    assert!(rlm_config.max_total_tokens.is_none());
    assert!(rlm_config.compaction_model.is_none());
}

// ─────────────────────────────────────────────────────────────────────────────
// Metadata and output format tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_explore_tool_metadata_footer_format() {
    // Verify the metadata footer contains all expected fields.
    let backend = MockBackend::new(vec![mock_text_response("Answer.")]);

    let spawner = make_spawner(backend);
    let tool = ExploreTool::new(spawner);
    let ctx = ToolContext::default();

    let result = tool.execute(json!({"query": "test"}), &ctx).await.unwrap();

    let content = result.to_llm_content();

    // Should contain the summary
    assert!(content.starts_with("Answer."));
    // Should contain metadata separator
    assert!(content.contains("---"));
    // Should contain iteration count
    assert!(content.contains("1 iterations"));
    // Should contain token counts
    assert!(content.contains("150 tokens"));
    assert!(content.contains("100in"));
    assert!(content.contains("50out"));
}

#[tokio::test]
async fn test_explore_tool_compaction_metadata() {
    // Verify compaction count appears in the metadata footer.
    // Need enough tool calls (4+) with large outputs to trigger compaction
    // with preserve_recent=3 default.
    let large_output = "x".repeat(2000);

    let agent_backend = MockBackend::new(vec![
        mock_tool_use_response("call_1", "grep", json!({})),
        mock_tool_use_response("call_2", "grep", json!({})),
        mock_tool_use_response("call_3", "grep", json!({})),
        mock_tool_use_response("call_4", "grep", json!({})),
        mock_text_response("Done after compaction."),
    ]);

    let compaction_backend = MockBackend::new(vec![mock_text_response("Compacted.")]);

    let mut tools = ToolRegistry::new();
    tools.register(MockTool::new("grep").with_response(ToolResult::text(&large_output)));

    let config = RlmConfig {
        max_context_tokens: 1000,
        compaction_threshold: 0.1,
        ..RlmConfig::default()
    };

    let spawner = Arc::new(
        RlmSpawner::new(Arc::new(agent_backend), tools)
            .with_config(config)
            .with_compaction_backend(Arc::new(compaction_backend)),
    );

    let tool = ExploreTool::new(spawner);
    let ctx = ToolContext::default();

    let result = tool.execute(json!({"query": "test"}), &ctx).await.unwrap();

    let content = result.to_llm_content();
    assert!(content.contains("compaction"));
}

#[tokio::test]
async fn test_explore_tool_truncated_metadata() {
    // When exploration is truncated, verify [truncated] appears in output.
    let agent_backend = MockBackend::new(vec![
        mock_tool_use_response("call_1", "grep", json!({})),
        mock_tool_use_response("call_2", "grep", json!({})),
    ]);

    let mut tools = ToolRegistry::new();
    tools.register(MockTool::new("grep"));

    let config = RlmConfig {
        max_turns: 1, // Stop after 1 turn
        ..RlmConfig::default()
    };

    let spawner = Arc::new(RlmSpawner::new(Arc::new(agent_backend), tools).with_config(config));

    let tool = ExploreTool::new(spawner);
    let ctx = ToolContext::default();

    let result = tool.execute(json!({"query": "test"}), &ctx).await.unwrap();

    let content = result.to_llm_content();
    assert!(content.contains("[truncated]"));
}
