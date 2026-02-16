//! Application state shared across handlers.

use std::collections::HashMap;
use std::sync::Arc;

use arawn_agent::{Agent, Session, SessionId, SessionIndexer};
use arawn_mcp::McpManager;
use arawn_types::{HasSessionConfig, SharedHookDispatcher};
use arawn_workstream::WorkstreamManager;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::config::ServerConfig;
use crate::ratelimit::{SharedRateLimiter, create_rate_limiter};
use crate::session_cache::SessionCache;

/// Thread-safe MCP manager.
pub type SharedMcpManager = Arc<RwLock<McpManager>>;

/// Task status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Task is queued but not started.
    Pending,
    /// Task is currently running.
    Running,
    /// Task completed successfully.
    Completed,
    /// Task failed.
    Failed,
    /// Task was cancelled.
    Cancelled,
}

/// A tracked task/operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedTask {
    /// Task ID.
    pub id: String,
    /// Task type/name.
    pub task_type: String,
    /// Current status.
    pub status: TaskStatus,
    /// Progress percentage (0-100).
    pub progress: Option<u8>,
    /// Status message.
    pub message: Option<String>,
    /// Associated session ID.
    pub session_id: Option<String>,
    /// When the task was created.
    pub created_at: DateTime<Utc>,
    /// When the task started running.
    pub started_at: Option<DateTime<Utc>>,
    /// When the task completed.
    pub completed_at: Option<DateTime<Utc>>,
    /// Error message if failed.
    pub error: Option<String>,
}

impl TrackedTask {
    /// Create a new pending task.
    pub fn new(id: impl Into<String>, task_type: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            task_type: task_type.into(),
            status: TaskStatus::Pending,
            progress: None,
            message: None,
            session_id: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
        }
    }

    /// Set the session ID.
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Mark the task as running.
    pub fn start(&mut self) {
        self.status = TaskStatus::Running;
        self.started_at = Some(Utc::now());
    }

    /// Update progress.
    pub fn update_progress(&mut self, progress: u8, message: Option<String>) {
        self.progress = Some(progress.min(100));
        self.message = message;
    }

    /// Mark the task as completed.
    pub fn complete(&mut self, message: Option<String>) {
        self.status = TaskStatus::Completed;
        self.progress = Some(100);
        self.message = message;
        self.completed_at = Some(Utc::now());
    }

    /// Mark the task as failed.
    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = TaskStatus::Failed;
        self.error = Some(error.into());
        self.completed_at = Some(Utc::now());
    }

    /// Mark the task as cancelled.
    pub fn cancel(&mut self) {
        self.status = TaskStatus::Cancelled;
        self.completed_at = Some(Utc::now());
    }
}

/// In-memory task store.
pub type TaskStore = Arc<RwLock<HashMap<String, TrackedTask>>>;

/// Application state shared across all handlers.
#[derive(Clone)]
pub struct AppState {
    /// The agent instance.
    pub agent: Arc<Agent>,

    /// Server configuration.
    pub config: Arc<ServerConfig>,

    /// Per-IP rate limiter (created from config.api_rpm).
    pub rate_limiter: SharedRateLimiter,

    /// Session cache - loads from workstream on cache miss, persists back on save.
    pub session_cache: SessionCache,

    /// Workstream manager (optional — None if workstreams not configured).
    pub workstreams: Option<Arc<WorkstreamManager>>,

    /// Session indexer (optional — None when indexing disabled).
    pub indexer: Option<Arc<SessionIndexer>>,

    /// Hook dispatcher for session lifecycle events (optional).
    pub hook_dispatcher: Option<SharedHookDispatcher>,

    /// MCP manager for Model Context Protocol servers (optional — None if MCP disabled).
    pub mcp_manager: Option<SharedMcpManager>,

    /// Task store for tracking long-running operations.
    pub tasks: TaskStore,
}

impl AppState {
    /// Create a new application state.
    pub fn new(agent: Agent, config: ServerConfig) -> Self {
        // Create rate limiter with configured RPM
        let rate_limiter = create_rate_limiter(config.api_rpm);

        Self {
            agent: Arc::new(agent),
            config: Arc::new(config),
            rate_limiter,
            session_cache: SessionCache::new(None),
            workstreams: None,
            indexer: None,
            hook_dispatcher: None,
            mcp_manager: None,
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create application state with workstream support.
    pub fn with_workstreams(mut self, manager: WorkstreamManager) -> Self {
        let ws_arc = Arc::new(manager);
        self.session_cache = SessionCache::new(Some(ws_arc.clone()));
        self.workstreams = Some(ws_arc);
        self
    }

    /// Create application state with session indexer.
    pub fn with_indexer(mut self, indexer: SessionIndexer) -> Self {
        self.indexer = Some(Arc::new(indexer));
        self
    }

    /// Create application state with hook dispatcher for lifecycle events.
    pub fn with_hook_dispatcher(mut self, dispatcher: SharedHookDispatcher) -> Self {
        self.hook_dispatcher = Some(dispatcher);
        self
    }

    /// Create application state with MCP manager.
    pub fn with_mcp_manager(mut self, manager: McpManager) -> Self {
        self.mcp_manager = Some(Arc::new(RwLock::new(manager)));
        self
    }

    /// Configure session cache using a config provider.
    ///
    /// This applies session cache settings (max sessions, TTL, etc.) from any
    /// type implementing `HasSessionConfig`, enabling decoupled configuration.
    pub fn with_session_config<C: HasSessionConfig>(mut self, config: &C) -> Self {
        // Recreate session cache with config, preserving workstream manager
        self.session_cache = SessionCache::from_session_config(
            self.workstreams.clone(),
            config,
        );
        self
    }

    /// Get or create a session by ID.
    ///
    /// If session_id is None, creates a new session.
    /// Defaults to "scratch" workstream.
    pub async fn get_or_create_session(&self, session_id: Option<SessionId>) -> SessionId {
        self.get_or_create_session_in_workstream(session_id, "scratch")
            .await
    }

    /// Get or create a session in a specific workstream.
    ///
    /// Sessions are loaded from workstream storage on cache miss and persisted back.
    pub async fn get_or_create_session_in_workstream(
        &self,
        session_id: Option<SessionId>,
        workstream_id: &str,
    ) -> SessionId {
        let result = self
            .session_cache
            .get_or_create(session_id, workstream_id)
            .await;

        let (id, is_new) = match result {
            Ok((id, _, is_new)) => (id, is_new),
            Err(e) => {
                warn!("Session cache error: {}, creating new session", e);
                let (id, _) = self.session_cache.create_session(workstream_id).await;
                (id, true)
            }
        };

        // Fire SessionStart hook for new sessions
        if is_new {
            if let Some(ref dispatcher) = self.hook_dispatcher {
                let outcome = dispatcher.dispatch_session_start(&id.to_string()).await;
                debug!(session_id = %id, ?outcome, "SessionStart hook dispatched");
            }
        }

        id
    }

    /// Close a session: remove it from the cache and trigger background indexing.
    ///
    /// Returns `true` if the session existed and was removed.
    /// Indexing runs asynchronously and does not block the caller.
    pub async fn close_session(&self, session_id: SessionId) -> bool {
        let session = match self.session_cache.remove(&session_id).await {
            Some(s) => s,
            None => return false,
        };

        let turn_count = session.turn_count();

        // Fire SessionEnd hook
        if let Some(ref dispatcher) = self.hook_dispatcher {
            let outcome = dispatcher
                .dispatch_session_end(&session_id.to_string(), turn_count)
                .await;
            debug!(session_id = %session_id, turn_count, ?outcome, "SessionEnd hook dispatched");
        }

        // Spawn background indexing if indexer is configured and session has turns
        if let Some(indexer) = &self.indexer {
            if !session.is_empty() {
                let indexer = Arc::clone(indexer);
                let messages = session_to_messages(&session);
                let sid = session_id.to_string();

                tokio::spawn(async move {
                    let report = indexer
                        .index_session(&sid, &messages_as_refs(&messages))
                        .await;
                    info!(
                        session_id = %sid,
                        report = %report,
                        "Background session indexing complete"
                    );
                    if report.has_errors() {
                        warn!(
                            session_id = %sid,
                            errors = ?report.errors,
                            "Session indexing completed with errors"
                        );
                    }
                });
            }
        }

        true
    }

    /// Get session from cache (loading from workstream if needed).
    pub async fn get_session(&self, session_id: SessionId, workstream_id: &str) -> Option<Session> {
        match self.session_cache.get_or_load(session_id, workstream_id).await {
            Ok((session, _)) => Some(session),
            Err(_) => None,
        }
    }

    /// Update session in cache.
    pub async fn update_session(&self, session_id: SessionId, session: Session) {
        let _ = self.session_cache.update(session_id, session).await;
    }

    /// Invalidate a cached session (e.g., after workstream reassignment).
    pub async fn invalidate_session(&self, session_id: SessionId) {
        self.session_cache.invalidate(&session_id).await;
    }
}

/// Convert a session's turns into owned `(role, content)` pairs.
pub(crate) fn session_to_messages(session: &Session) -> Vec<(String, String)> {
    let mut messages = Vec::new();
    for turn in session.all_turns() {
        messages.push(("user".to_string(), turn.user_message.clone()));
        if let Some(ref response) = turn.assistant_response {
            messages.push(("assistant".to_string(), response.clone()));
        }
    }
    messages
}

/// Convert owned message pairs to borrowed slices for the indexer API.
pub(crate) fn messages_as_refs(messages: &[(String, String)]) -> Vec<(&str, &str)> {
    messages
        .iter()
        .map(|(r, c)| (r.as_str(), c.as_str()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use arawn_agent::{Agent, ToolRegistry};
    use arawn_llm::MockBackend;

    fn create_test_state() -> AppState {
        let backend = MockBackend::with_text("Test");
        let agent = Agent::builder()
            .with_backend(backend)
            .with_tools(ToolRegistry::new())
            .build()
            .unwrap();
        AppState::new(agent, ServerConfig::new(Some("test-token".to_string())))
    }

    #[test]
    fn test_session_to_messages_empty() {
        let session = Session::new();
        let messages = session_to_messages(&session);
        assert!(messages.is_empty());
    }

    #[test]
    fn test_session_to_messages_with_turns() {
        let mut session = Session::new();
        let turn = session.start_turn("Hello");
        turn.complete("Hi there!");
        let turn = session.start_turn("How are you?");
        turn.complete("I'm great!");

        let messages = session_to_messages(&session);
        assert_eq!(messages.len(), 4);
        assert_eq!(messages[0], ("user".to_string(), "Hello".to_string()));
        assert_eq!(
            messages[1],
            ("assistant".to_string(), "Hi there!".to_string())
        );
        assert_eq!(
            messages[2],
            ("user".to_string(), "How are you?".to_string())
        );
        assert_eq!(
            messages[3],
            ("assistant".to_string(), "I'm great!".to_string())
        );
    }

    #[test]
    fn test_session_to_messages_incomplete_turn() {
        let mut session = Session::new();
        session.start_turn("Hello");
        // No assistant response set

        let messages = session_to_messages(&session);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], ("user".to_string(), "Hello".to_string()));
    }

    #[test]
    fn test_messages_as_refs() {
        let owned = vec![
            ("user".to_string(), "Hello".to_string()),
            ("assistant".to_string(), "Hi".to_string()),
        ];
        let refs = messages_as_refs(&owned);
        assert_eq!(refs, vec![("user", "Hello"), ("assistant", "Hi")]);
    }

    #[tokio::test]
    async fn test_close_session_removes_session() {
        let state = create_test_state();
        let session_id = state.get_or_create_session(None).await;

        // Session exists in cache
        assert!(state.session_cache.contains(&session_id).await);

        // Close it
        assert!(state.close_session(session_id).await);

        // Session removed
        assert!(!state.session_cache.contains(&session_id).await);
    }

    #[tokio::test]
    async fn test_close_session_nonexistent_returns_false() {
        let state = create_test_state();
        let fake_id = SessionId::new();
        assert!(!state.close_session(fake_id).await);
    }

    #[tokio::test]
    async fn test_close_session_without_indexer() {
        let state = create_test_state();
        let session_id = state.get_or_create_session(None).await;

        // Add a turn so the session isn't empty
        state
            .session_cache
            .with_session_mut(&session_id, |session| {
                let turn = session.start_turn("Hello");
                turn.complete("Hi!");
            })
            .await;

        // Should succeed even without indexer
        assert!(state.close_session(session_id).await);
        assert!(!state.session_cache.contains(&session_id).await);
    }

    #[test]
    fn test_default_state_has_no_indexer() {
        let state = create_test_state();
        assert!(state.indexer.is_none());
    }
}
