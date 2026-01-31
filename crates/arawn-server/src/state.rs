//! Application state shared across handlers.

use std::collections::HashMap;
use std::sync::Arc;

use arawn_agent::{Agent, Session, SessionId, SessionIndexer};
use arawn_mcp::McpManager;
use arawn_types::SharedHookDispatcher;
use arawn_workstream::WorkstreamManager;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::config::ServerConfig;

/// In-memory session store.
pub type SessionStore = Arc<RwLock<HashMap<SessionId, Session>>>;

/// Thread-safe MCP manager.
pub type SharedMcpManager = Arc<RwLock<McpManager>>;

/// Application state shared across all handlers.
#[derive(Clone)]
pub struct AppState {
    /// The agent instance.
    pub agent: Arc<Agent>,

    /// Server configuration.
    pub config: Arc<ServerConfig>,

    /// In-memory session store.
    pub sessions: SessionStore,

    /// Workstream manager (optional — None if workstreams not configured).
    pub workstreams: Option<Arc<WorkstreamManager>>,

    /// Session indexer (optional — None when indexing disabled).
    pub indexer: Option<Arc<SessionIndexer>>,

    /// Hook dispatcher for session lifecycle events (optional).
    pub hook_dispatcher: Option<SharedHookDispatcher>,

    /// MCP manager for Model Context Protocol servers (optional — None if MCP disabled).
    pub mcp_manager: Option<SharedMcpManager>,
}

impl AppState {
    /// Create a new application state.
    pub fn new(agent: Agent, config: ServerConfig) -> Self {
        Self {
            agent: Arc::new(agent),
            config: Arc::new(config),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            workstreams: None,
            indexer: None,
            hook_dispatcher: None,
            mcp_manager: None,
        }
    }

    /// Create application state with workstream support.
    pub fn with_workstreams(mut self, manager: WorkstreamManager) -> Self {
        self.workstreams = Some(Arc::new(manager));
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

    /// Get or create a session by ID.
    ///
    /// If session_id is None, creates a new session.
    /// Returns the session ID and a mutable reference to the session.
    pub async fn get_or_create_session(&self, session_id: Option<SessionId>) -> SessionId {
        let (id, is_new) = {
            let mut sessions = self.sessions.write().await;

            match session_id {
                Some(id) if sessions.contains_key(&id) => (id, false),
                Some(id) => {
                    // Create session with the provided ID
                    sessions.insert(id, Session::with_id(id));
                    (id, true)
                }
                None => {
                    // Create new session
                    let session = Session::new();
                    let id = session.id;
                    sessions.insert(id, session);
                    (id, true)
                }
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

    /// Close a session: remove it from the store and trigger background indexing.
    ///
    /// Returns `true` if the session existed and was removed.
    /// Indexing runs asynchronously and does not block the caller.
    pub async fn close_session(&self, session_id: SessionId) -> bool {
        let session = {
            let mut sessions = self.sessions.write().await;
            sessions.remove(&session_id)
        };

        let session = match session {
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

        // Session exists
        assert!(state.sessions.read().await.contains_key(&session_id));

        // Close it
        assert!(state.close_session(session_id).await);

        // Session removed
        assert!(!state.sessions.read().await.contains_key(&session_id));
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
        {
            let mut sessions = state.sessions.write().await;
            let session = sessions.get_mut(&session_id).unwrap();
            let turn = session.start_turn("Hello");
            turn.complete("Hi!");
        }

        // Should succeed even without indexer
        assert!(state.close_session(session_id).await);
        assert!(!state.sessions.read().await.contains_key(&session_id));
    }

    #[test]
    fn test_default_state_has_no_indexer() {
        let state = create_test_state();
        assert!(state.indexer.is_none());
    }
}
