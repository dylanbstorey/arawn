//! Request and response types for the Arawn API.
//!
//! These types mirror the server's API contract.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// Sessions
// ─────────────────────────────────────────────────────────────────────────────

/// Request to create a new session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    /// Optional title for the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Optional metadata to attach to the session.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Request to update a session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateSessionRequest {
    /// New title for the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Metadata to merge into the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Move session to a different workstream.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workstream_id: Option<String>,
}

/// Summary info for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    /// Session ID.
    pub id: String,
    /// Session title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Number of turns in the session.
    pub turn_count: usize,
    /// Creation time (ISO 8601).
    pub created_at: String,
    /// Last update time (ISO 8601).
    pub updated_at: String,
}

/// Full session details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDetail {
    /// Session ID.
    pub id: String,
    /// All turns in the session.
    pub turns: Vec<TurnInfo>,
    /// Creation time.
    pub created_at: String,
    /// Last update time.
    pub updated_at: String,
    /// Session metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Turn info within a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnInfo {
    /// Turn ID.
    pub id: String,
    /// User message.
    pub user_message: String,
    /// Assistant response (if complete).
    pub assistant_response: Option<String>,
    /// Number of tool calls.
    pub tool_call_count: usize,
    /// When the turn started.
    pub started_at: String,
    /// When the turn completed.
    pub completed_at: Option<String>,
}

/// Message info for conversation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageInfo {
    /// Role of the message sender.
    pub role: String,
    /// Content of the message.
    pub content: String,
    /// Timestamp of the message.
    pub timestamp: String,
}

/// Response containing session messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessagesResponse {
    /// Session ID.
    pub session_id: String,
    /// List of messages in the session.
    pub messages: Vec<MessageInfo>,
    /// Total message count.
    pub count: usize,
}

/// Response for list sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSessionsResponse {
    /// List of sessions.
    pub sessions: Vec<SessionSummary>,
    /// Total count.
    pub total: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// Workstreams
// ─────────────────────────────────────────────────────────────────────────────

/// Request to create a workstream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkstreamRequest {
    /// Workstream title.
    pub title: String,
    /// Default model for the workstream.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    /// Tags for categorization.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// Request to update a workstream.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateWorkstreamRequest {
    /// New title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// New summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// New default model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    /// New tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Workstream details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workstream {
    /// Workstream ID.
    pub id: String,
    /// Title.
    pub title: String,
    /// Summary.
    #[serde(default)]
    pub summary: Option<String>,
    /// State (active, archived).
    pub state: String,
    /// Default model.
    #[serde(default)]
    pub default_model: Option<String>,
    /// Whether this is the scratch workstream.
    pub is_scratch: bool,
    /// Creation time.
    pub created_at: String,
    /// Last update time.
    pub updated_at: String,
    /// Tags.
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

/// Response for list workstreams.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWorkstreamsResponse {
    /// List of workstreams.
    pub workstreams: Vec<Workstream>,
}

/// Request to send a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    /// Message role (user, assistant, system, agent_push).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Message content.
    pub content: String,
    /// Optional metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
}

/// Workstream message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkstreamMessage {
    /// Message ID.
    pub id: String,
    /// Workstream ID.
    pub workstream_id: String,
    /// Session ID.
    #[serde(default)]
    pub session_id: Option<String>,
    /// Role.
    pub role: String,
    /// Content.
    pub content: String,
    /// Timestamp.
    pub timestamp: String,
    /// Metadata.
    #[serde(default)]
    pub metadata: Option<String>,
}

/// Response for list messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMessagesResponse {
    /// List of messages.
    pub messages: Vec<WorkstreamMessage>,
}

/// Workstream session info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkstreamSession {
    /// Session ID.
    pub id: String,
    /// Workstream ID.
    pub workstream_id: String,
    /// Start time.
    pub started_at: String,
    /// End time.
    #[serde(default)]
    pub ended_at: Option<String>,
    /// Whether the session is active.
    pub is_active: bool,
}

/// Response for list workstream sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWorkstreamSessionsResponse {
    /// List of sessions.
    pub sessions: Vec<WorkstreamSession>,
}

/// Request to promote scratch workstream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromoteRequest {
    /// New title for the promoted workstream.
    pub title: String,
    /// Tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Default model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Chat
// ─────────────────────────────────────────────────────────────────────────────

/// Chat request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// User message.
    pub message: String,
    /// Session ID (optional - creates new session if not provided).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Model override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// System prompt override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    /// Temperature (0.0 - 1.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Max tokens for response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

impl ChatRequest {
    /// Create a new chat request with just a message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            session_id: None,
            model: None,
            system_prompt: None,
            temperature: None,
            max_tokens: None,
        }
    }

    /// Set the session ID.
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set the model.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
}

/// Chat response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Assistant response.
    pub response: String,
    /// Session ID.
    pub session_id: String,
    /// Turn ID.
    pub turn_id: String,
    /// Tool calls made during this turn.
    #[serde(default)]
    pub tool_calls: Vec<ToolCallInfo>,
    /// Model used.
    #[serde(default)]
    pub model: Option<String>,
    /// Token usage.
    #[serde(default)]
    pub usage: Option<TokenUsage>,
}

/// Tool call information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
    /// Tool name.
    pub name: String,
    /// Tool call ID.
    pub id: String,
    /// Whether the call succeeded.
    pub success: bool,
}

/// Token usage information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Prompt tokens.
    pub prompt_tokens: u32,
    /// Completion tokens.
    pub completion_tokens: u32,
    /// Total tokens.
    pub total_tokens: u32,
}

/// Streaming chat event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// Session started.
    SessionStart {
        session_id: String,
        turn_id: String,
    },
    /// Text content chunk.
    Content {
        text: String,
    },
    /// Tool call started.
    ToolStart {
        tool_name: String,
        tool_call_id: String,
    },
    /// Tool output chunk.
    ToolOutput {
        tool_call_id: String,
        content: String,
    },
    /// Tool call completed.
    ToolEnd {
        tool_call_id: String,
        success: bool,
    },
    /// Turn completed.
    Done {
        response: String,
        #[serde(default)]
        usage: Option<TokenUsage>,
    },
    /// Error occurred.
    Error {
        message: String,
    },
}

// ─────────────────────────────────────────────────────────────────────────────
// Config
// ─────────────────────────────────────────────────────────────────────────────

/// Server configuration response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    /// Server package version.
    pub version: String,
    /// API contract version, independent of package version.
    #[serde(default)]
    pub api_version: Option<String>,
    /// Feature flags.
    pub features: ConfigFeatures,
    /// Resource limits.
    pub limits: ConfigLimits,
    /// Bind address.
    pub bind_address: String,
    /// Whether authentication is required.
    pub auth_required: bool,
}

/// Server feature flags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFeatures {
    /// Whether workstreams are enabled.
    pub workstreams_enabled: bool,
    /// Whether memory is enabled.
    pub memory_enabled: bool,
    /// Whether MCP is enabled.
    pub mcp_enabled: bool,
    /// Whether rate limiting is enabled.
    pub rate_limiting: bool,
    /// Whether request logging is enabled.
    pub request_logging: bool,
}

/// Server limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigLimits {
    /// Maximum concurrent requests.
    #[serde(default)]
    pub max_concurrent_requests: Option<u32>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Agents
// ─────────────────────────────────────────────────────────────────────────────

/// Agent summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSummary {
    /// Agent ID.
    pub id: String,
    /// Agent name.
    pub name: String,
    /// Whether this is the default agent.
    pub is_default: bool,
    /// Number of tools available.
    pub tool_count: usize,
}

/// Agent details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDetail {
    /// Agent ID.
    pub id: String,
    /// Agent name.
    pub name: String,
    /// Whether this is the default agent.
    pub is_default: bool,
    /// Available tools.
    pub tools: Vec<AgentToolInfo>,
    /// Agent capabilities.
    pub capabilities: AgentCapabilities,
}

/// Tool info for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolInfo {
    /// Tool name.
    pub name: String,
    /// Tool description.
    pub description: String,
}

/// Agent capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    /// Whether streaming is supported.
    pub streaming: bool,
    /// Whether tool use is supported.
    pub tool_use: bool,
    /// Maximum context length.
    #[serde(default)]
    pub max_context_length: Option<usize>,
}

/// Response for list agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAgentsResponse {
    /// List of agents.
    pub agents: Vec<AgentSummary>,
    /// Total count.
    pub total: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// Notes
// ─────────────────────────────────────────────────────────────────────────────

/// A note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// Note ID.
    pub id: String,
    /// Note content.
    pub content: String,
    /// Tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Creation time.
    pub created_at: String,
}

/// Request to create a note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNoteRequest {
    /// Note content.
    pub content: String,
    /// Tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// Request to update a note.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateNoteRequest {
    /// New content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// New tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Response for list notes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListNotesResponse {
    /// List of notes.
    pub notes: Vec<Note>,
    /// Total count.
    pub total: usize,
}

/// Response for single note operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteResponse {
    /// The note.
    pub note: Note,
}

// ─────────────────────────────────────────────────────────────────────────────
// Memory
// ─────────────────────────────────────────────────────────────────────────────

/// Request to store a memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreMemoryRequest {
    /// Memory content.
    pub content: String,
    /// Content type (fact, summary, etc.).
    #[serde(default = "default_content_type")]
    pub content_type: String,
    /// Session ID to associate with.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Confidence score (0.0 - 1.0).
    #[serde(default = "default_confidence")]
    pub confidence: f32,
}

fn default_content_type() -> String {
    "fact".to_string()
}

fn default_confidence() -> f32 {
    0.8
}

/// Response after storing a memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreMemoryResponse {
    /// Memory ID.
    pub id: String,
    /// Content type.
    pub content_type: String,
    /// Confirmation message.
    pub message: String,
}

/// Memory search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    /// Result ID.
    pub id: String,
    /// Content type.
    pub content_type: String,
    /// Content text.
    pub content: String,
    /// Session ID.
    #[serde(default)]
    pub session_id: Option<String>,
    /// Relevance score.
    pub score: f32,
    /// Source.
    pub source: String,
    /// Citation metadata.
    #[serde(default)]
    pub citation: Option<serde_json::Value>,
}

/// Response for memory search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResponse {
    /// Search results.
    pub results: Vec<MemorySearchResult>,
    /// Query executed.
    pub query: String,
    /// Result count.
    pub count: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// Tasks
// ─────────────────────────────────────────────────────────────────────────────

/// Task status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Task is pending.
    Pending,
    /// Task is running.
    Running,
    /// Task completed.
    Completed,
    /// Task failed.
    Failed,
    /// Task was cancelled.
    Cancelled,
}

/// Task summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    /// Task ID.
    pub id: String,
    /// Task type.
    pub task_type: String,
    /// Current status.
    pub status: TaskStatus,
    /// Progress percentage.
    #[serde(default)]
    pub progress: Option<u8>,
    /// Creation time.
    pub created_at: String,
}

/// Task details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDetail {
    /// Task ID.
    pub id: String,
    /// Task type.
    pub task_type: String,
    /// Current status.
    pub status: TaskStatus,
    /// Progress percentage.
    #[serde(default)]
    pub progress: Option<u8>,
    /// Status message.
    #[serde(default)]
    pub message: Option<String>,
    /// Session ID.
    #[serde(default)]
    pub session_id: Option<String>,
    /// Creation time.
    pub created_at: String,
    /// Start time.
    #[serde(default)]
    pub started_at: Option<String>,
    /// Completion time.
    #[serde(default)]
    pub completed_at: Option<String>,
    /// Error message.
    #[serde(default)]
    pub error: Option<String>,
}

/// Response for list tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTasksResponse {
    /// List of tasks.
    pub tasks: Vec<TaskSummary>,
    /// Total count.
    pub total: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// MCP
// ─────────────────────────────────────────────────────────────────────────────

/// Request to add an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddServerRequest {
    /// Server name.
    pub name: String,
    /// Command to run (for stdio servers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Command arguments.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// Environment variables.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    /// HTTP URL (for HTTP servers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Whether to connect immediately.
    #[serde(default)]
    pub auto_connect: bool,
}

/// Response after adding a server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddServerResponse {
    /// Server name.
    pub name: String,
    /// Whether the server is connected.
    pub connected: bool,
    /// Available tools (if connected).
    #[serde(default)]
    pub tools: Vec<String>,
}

/// MCP server info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server name.
    pub name: String,
    /// Server type (stdio, http).
    pub server_type: String,
    /// Whether connected.
    pub connected: bool,
    /// Tool count (if connected).
    #[serde(default)]
    pub tool_count: Option<usize>,
}

/// Response for list servers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListServersResponse {
    /// List of servers.
    pub servers: Vec<ServerInfo>,
}

/// Tool info from MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolInfo {
    /// Tool name.
    pub name: String,
    /// Tool description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Response for list server tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResponse {
    /// Server name.
    pub server: String,
    /// List of tools.
    pub tools: Vec<McpToolInfo>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Health
// ─────────────────────────────────────────────────────────────────────────────

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Status (ok, degraded, unhealthy).
    pub status: String,
    /// Server version.
    #[serde(default)]
    pub version: Option<String>,
}

/// Detailed health response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedHealthResponse {
    /// Overall status.
    pub status: String,
    /// Server version.
    pub version: String,
    /// Uptime in seconds.
    pub uptime_seconds: u64,
    /// Component health.
    pub components: HealthComponents,
}

/// Health of individual components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthComponents {
    /// Agent status.
    pub agent: String,
    /// Memory status.
    #[serde(default)]
    pub memory: Option<String>,
    /// Workstreams status.
    #[serde(default)]
    pub workstreams: Option<String>,
    /// MCP status.
    #[serde(default)]
    pub mcp: Option<String>,
}
