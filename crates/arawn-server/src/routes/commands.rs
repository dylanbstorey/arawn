//! Command infrastructure and REST API endpoints.
//!
//! Commands are server-side operations that can be invoked via the API.
//! The `/` syntax is purely client-side presentation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    Extension, Json,
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use arawn_agent::{CompactionResult, CompactorConfig, SessionCompactor, SessionId};
use uuid::Uuid;

use crate::auth::Identity;
use crate::error::ServerError;
use crate::state::AppState;

// ─────────────────────────────────────────────────────────────────────────────
// Command Types
// ─────────────────────────────────────────────────────────────────────────────

/// Result type for command execution.
pub type CommandResult<T> = std::result::Result<T, CommandError>;

/// Error type for command execution.
#[derive(Debug, Clone, Serialize)]
pub struct CommandError {
    /// Error code.
    pub code: String,
    /// Error message.
    pub message: String,
}

impl CommandError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self {
            code: "not_found".to_string(),
            message: msg.into(),
        }
    }

    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self {
            code: "invalid_params".to_string(),
            message: msg.into(),
        }
    }

    pub fn execution_failed(msg: impl Into<String>) -> Self {
        Self {
            code: "execution_failed".to_string(),
            message: msg.into(),
        }
    }
}

impl From<CommandError> for ServerError {
    fn from(e: CommandError) -> Self {
        match e.code.as_str() {
            "not_found" => ServerError::NotFound(e.message),
            "invalid_params" => ServerError::BadRequest(e.message),
            _ => ServerError::Internal(e.message),
        }
    }
}

/// Command handler trait.
#[async_trait]
pub trait CommandHandler: Send + Sync {
    /// The command name (e.g., "compact").
    fn name(&self) -> &str;

    /// Short description of what the command does.
    fn description(&self) -> &str;

    /// Execute the command with the given parameters.
    async fn execute(
        &self,
        state: &AppState,
        params: serde_json::Value,
    ) -> CommandResult<CommandOutput>;
}

/// Output from command execution.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CommandOutput {
    /// Simple text result.
    Text { message: String },
    /// Structured JSON result.
    Json { data: serde_json::Value },
    /// Progress update (for streaming).
    Progress { percent: u8, message: String },
    /// Command completed successfully.
    Completed { result: serde_json::Value },
    /// Command failed.
    Error { error: CommandError },
}

// ─────────────────────────────────────────────────────────────────────────────
// Command Registry
// ─────────────────────────────────────────────────────────────────────────────

/// Registry for command handlers.
#[derive(Default)]
pub struct CommandRegistry {
    handlers: HashMap<String, Arc<dyn CommandHandler>>,
}

impl CommandRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Create a registry with default commands.
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(CompactCommand::default());
        registry
    }

    /// Register a command handler.
    pub fn register<H: CommandHandler + 'static>(&mut self, handler: H) {
        self.handlers
            .insert(handler.name().to_string(), Arc::new(handler));
    }

    /// Get a command handler by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn CommandHandler>> {
        self.handlers.get(name).cloned()
    }

    /// List all registered commands.
    pub fn list(&self) -> Vec<CommandInfo> {
        self.handlers
            .values()
            .map(|h| CommandInfo {
                name: h.name().to_string(),
                description: h.description().to_string(),
            })
            .collect()
    }
}

/// Thread-safe command registry.
pub type SharedCommandRegistry = Arc<RwLock<CommandRegistry>>;

// ─────────────────────────────────────────────────────────────────────────────
// API Types
// ─────────────────────────────────────────────────────────────────────────────

/// Command info for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInfo {
    /// Command name.
    pub name: String,
    /// Command description.
    pub description: String,
}

/// Response for listing commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCommandsResponse {
    /// Available commands.
    pub commands: Vec<CommandInfo>,
}

/// Request to execute the compact command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactRequest {
    /// Session ID to compact.
    pub session_id: String,
    /// Force compaction even if below threshold.
    #[serde(default)]
    pub force: bool,
}

/// Response from compact command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactResponse {
    /// Whether compaction was performed.
    pub compacted: bool,
    /// Number of turns compacted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turns_compacted: Option<usize>,
    /// Tokens before compaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_before: Option<usize>,
    /// Tokens after compaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_after: Option<usize>,
    /// Message describing the result.
    pub message: String,
}

impl From<CompactionResult> for CompactResponse {
    fn from(result: CompactionResult) -> Self {
        Self {
            compacted: true,
            turns_compacted: Some(result.turns_compacted),
            tokens_before: Some(result.tokens_before),
            tokens_after: Some(result.tokens_after),
            message: format!(
                "Compacted {} turns, freed ~{} tokens",
                result.turns_compacted,
                result.tokens_freed()
            ),
        }
    }
}

/// SSE event for compact progress.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CompactEvent {
    /// Starting compaction.
    Started { turns_to_compact: usize },
    /// Generating summary.
    Summarizing,
    /// Compaction completed.
    Completed { result: CompactResponse },
    /// Compaction was cancelled.
    Cancelled,
    /// Error occurred.
    Error { message: String },
}

// ─────────────────────────────────────────────────────────────────────────────
// Compact Command
// ─────────────────────────────────────────────────────────────────────────────

/// The compact command handler.
pub struct CompactCommand {
    config: CompactorConfig,
}

impl Default for CompactCommand {
    fn default() -> Self {
        Self {
            config: CompactorConfig::default(),
        }
    }
}

impl CompactCommand {
    /// Create with custom config.
    pub fn with_config(config: CompactorConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl CommandHandler for CompactCommand {
    fn name(&self) -> &str {
        "compact"
    }

    fn description(&self) -> &str {
        "Compact session history by summarizing older turns"
    }

    async fn execute(
        &self,
        state: &AppState,
        params: serde_json::Value,
    ) -> CommandResult<CommandOutput> {
        let request: CompactRequest = serde_json::from_value(params)
            .map_err(|e| CommandError::invalid_params(format!("Invalid parameters: {}", e)))?;

        let uuid: Uuid = request
            .session_id
            .parse()
            .map_err(|_| CommandError::invalid_params("Invalid session_id format"))?;
        let session_id = SessionId::from_uuid(uuid);

        // Get session from cache
        let session = state
            .session_cache
            .get(&session_id)
            .await
            .ok_or_else(|| CommandError::not_found(format!("Session {} not found", session_id)))?;

        // Get LLM backend for compaction
        let backend = state.agent.backend();

        // Create compactor
        let compactor = SessionCompactor::new(backend, self.config.clone());

        // Check if compaction needed (unless forced)
        if !request.force && !compactor.needs_compaction(&session, 3) {
            return Ok(CommandOutput::Completed {
                result: serde_json::to_value(CompactResponse {
                    compacted: false,
                    turns_compacted: None,
                    tokens_before: None,
                    tokens_after: None,
                    message: "Session does not need compaction".to_string(),
                })
                .unwrap(),
            });
        }

        // Perform compaction
        match compactor.compact(&session).await {
            Ok(Some(result)) => {
                // Note: In a full implementation, we'd apply the compaction to the session
                // by replacing old turns with a summary turn. For now, just return the result.
                let response: CompactResponse = result.into();
                Ok(CommandOutput::Completed {
                    result: serde_json::to_value(response).unwrap(),
                })
            }
            Ok(None) => Ok(CommandOutput::Completed {
                result: serde_json::to_value(CompactResponse {
                    compacted: false,
                    turns_compacted: None,
                    tokens_before: None,
                    tokens_after: None,
                    message: "No turns to compact".to_string(),
                })
                .unwrap(),
            }),
            Err(e) => Err(CommandError::execution_failed(format!(
                "Compaction failed: {}",
                e
            ))),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// GET /api/v1/commands - List available commands.
pub async fn list_commands_handler(
    State(_state): State<AppState>,
    Extension(_identity): Extension<Identity>,
) -> Result<Json<ListCommandsResponse>, ServerError> {
    // Create a registry with defaults (in a real app, this would be in AppState)
    let registry = CommandRegistry::with_defaults();
    let commands = registry.list();

    Ok(Json(ListCommandsResponse { commands }))
}

/// POST /api/v1/commands/compact - Execute compact command.
pub async fn compact_command_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Json(request): Json<CompactRequest>,
) -> Result<Json<CompactResponse>, ServerError> {
    let command = CompactCommand::default();

    let params = serde_json::to_value(&request).map_err(|e| ServerError::Serialization(e))?;

    match command.execute(&state, params).await {
        Ok(CommandOutput::Completed { result }) => {
            let response: CompactResponse =
                serde_json::from_value(result).map_err(|e| ServerError::Serialization(e))?;
            Ok(Json(response))
        }
        Ok(_) => Err(ServerError::Internal(
            "Unexpected command output".to_string(),
        )),
        Err(e) => Err(e.into()),
    }
}

/// POST /api/v1/commands/compact/stream - Execute compact command with SSE.
pub async fn compact_command_stream_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Json(request): Json<CompactRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>>, ServerError> {
    let uuid: Uuid = request
        .session_id
        .parse()
        .map_err(|_| ServerError::BadRequest("Invalid session_id format".to_string()))?;
    let session_id = SessionId::from_uuid(uuid);

    // Get session from cache
    let session = state
        .session_cache
        .get(&session_id)
        .await
        .ok_or_else(|| ServerError::NotFound(format!("Session {} not found", session_id)))?;

    // Get LLM backend for compaction
    let backend = state.agent.backend();

    // Create compactor
    let config = CompactorConfig::default();
    let compactor = SessionCompactor::new(backend, config.clone());

    // Build the list of events
    let mut events: Vec<CompactEvent> = Vec::new();

    // Check if compaction needed
    if !request.force && !compactor.needs_compaction(&session, 3) {
        events.push(CompactEvent::Completed {
            result: CompactResponse {
                compacted: false,
                turns_compacted: None,
                tokens_before: None,
                tokens_after: None,
                message: "Session does not need compaction".to_string(),
            },
        });
    } else {
        // Add progress events
        events.push(CompactEvent::Started {
            turns_to_compact: session
                .turn_count()
                .saturating_sub(config.preserve_recent),
        });
        events.push(CompactEvent::Summarizing);

        // Perform compaction
        let final_event = match compactor.compact(&session).await {
            Ok(Some(result)) => CompactEvent::Completed {
                result: result.into(),
            },
            Ok(None) => CompactEvent::Completed {
                result: CompactResponse {
                    compacted: false,
                    turns_compacted: None,
                    tokens_before: None,
                    tokens_after: None,
                    message: "No turns to compact".to_string(),
                },
            },
            Err(e) => CompactEvent::Error {
                message: e.to_string(),
            },
        };
        events.push(final_event);
    }

    let stream = stream::iter(events.into_iter().map(|event| {
        Ok::<_, std::convert::Infallible>(
            Event::default().data(serde_json::to_string(&event).unwrap()),
        )
    }));

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use arawn_agent::{Agent, ToolRegistry};
    use arawn_llm::MockBackend;
    use crate::config::ServerConfig;

    fn create_test_state() -> AppState {
        let backend = MockBackend::with_text("Test summary of older conversation.");
        let agent = Agent::builder()
            .with_backend(backend)
            .with_tools(ToolRegistry::new())
            .build()
            .unwrap();
        AppState::new(agent, ServerConfig::new(Some("test-token".to_string())))
    }

    #[test]
    fn test_command_registry_new() {
        let registry = CommandRegistry::new();
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_command_registry_with_defaults() {
        let registry = CommandRegistry::with_defaults();
        let commands = registry.list();
        assert!(!commands.is_empty());
        assert!(commands.iter().any(|c| c.name == "compact"));
    }

    #[test]
    fn test_command_registry_register_and_lookup() {
        let mut registry = CommandRegistry::new();
        registry.register(CompactCommand::default());

        let handler = registry.get("compact");
        assert!(handler.is_some());
        assert_eq!(handler.unwrap().name(), "compact");
    }

    #[test]
    fn test_command_registry_list() {
        let mut registry = CommandRegistry::new();
        registry.register(CompactCommand::default());

        let commands = registry.list();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].name, "compact");
        assert!(!commands[0].description.is_empty());
    }

    #[test]
    fn test_command_registry_get_nonexistent() {
        let registry = CommandRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_compact_command_metadata() {
        let command = CompactCommand::default();
        assert_eq!(command.name(), "compact");
        assert!(!command.description().is_empty());
    }

    #[tokio::test]
    async fn test_compact_command_invalid_session_id() {
        let state = create_test_state();
        let command = CompactCommand::default();

        let params = serde_json::json!({
            "session_id": "not-a-uuid"
        });

        let result = command.execute(&state, params).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "invalid_params");
    }

    #[tokio::test]
    async fn test_compact_command_session_not_found() {
        let state = create_test_state();
        let command = CompactCommand::default();

        let params = serde_json::json!({
            "session_id": "00000000-0000-0000-0000-000000000001"
        });

        let result = command.execute(&state, params).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "not_found");
    }

    #[tokio::test]
    async fn test_compact_command_no_compaction_needed() {
        let state = create_test_state();
        let session_id = state.get_or_create_session(None).await;

        // Add only 2 turns - not enough to compact (default preserve 3)
        state
            .session_cache
            .with_session_mut(&session_id, |session| {
                let turn = session.start_turn("Hello");
                turn.complete("Hi!");
                let turn = session.start_turn("How are you?");
                turn.complete("Great!");
            })
            .await;

        let command = CompactCommand::default();
        let params = serde_json::json!({
            "session_id": session_id.to_string()
        });

        let result = command.execute(&state, params).await.unwrap();
        match result {
            CommandOutput::Completed { result } => {
                let response: CompactResponse = serde_json::from_value(result).unwrap();
                assert!(!response.compacted);
                assert!(response.message.contains("does not need"));
            }
            _ => panic!("Expected Completed output"),
        }
    }

    #[tokio::test]
    async fn test_compact_command_force() {
        let state = create_test_state();
        let session_id = state.get_or_create_session(None).await;

        // Add 6 turns - enough to compact 3
        state
            .session_cache
            .with_session_mut(&session_id, |session| {
                for i in 0..6 {
                    let turn = session.start_turn(format!("Message {}", i));
                    turn.complete(format!("Response {}", i));
                }
            })
            .await;

        let command = CompactCommand::default();
        let params = serde_json::json!({
            "session_id": session_id.to_string(),
            "force": true
        });

        let result = command.execute(&state, params).await.unwrap();
        match result {
            CommandOutput::Completed { result } => {
                let response: CompactResponse = serde_json::from_value(result).unwrap();
                assert!(response.compacted);
                assert_eq!(response.turns_compacted, Some(3));
            }
            _ => panic!("Expected Completed output"),
        }
    }

    #[test]
    fn test_compact_response_from_result() {
        let result = CompactionResult {
            turns_compacted: 5,
            tokens_before: 1000,
            tokens_after: 200,
            summary: "Summary".to_string(),
        };

        let response: CompactResponse = result.into();
        assert!(response.compacted);
        assert_eq!(response.turns_compacted, Some(5));
        assert_eq!(response.tokens_before, Some(1000));
        assert_eq!(response.tokens_after, Some(200));
        assert!(response.message.contains("5 turns"));
        assert!(response.message.contains("800 tokens"));
    }

    #[test]
    fn test_command_error_types() {
        let err = CommandError::not_found("Session missing");
        assert_eq!(err.code, "not_found");

        let err = CommandError::invalid_params("Bad param");
        assert_eq!(err.code, "invalid_params");

        let err = CommandError::execution_failed("Failed");
        assert_eq!(err.code, "execution_failed");
    }

    #[test]
    fn test_command_error_to_server_error() {
        let err: ServerError = CommandError::not_found("Missing").into();
        assert!(matches!(err, ServerError::NotFound(_)));

        let err: ServerError = CommandError::invalid_params("Bad").into();
        assert!(matches!(err, ServerError::BadRequest(_)));

        let err: ServerError = CommandError::execution_failed("Failed").into();
        assert!(matches!(err, ServerError::Internal(_)));
    }
}
