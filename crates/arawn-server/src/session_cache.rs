//! Session cache that loads sessions from workstream storage.
//!
//! This module provides a caching layer that:
//! - Loads sessions from workstream JSONL on cache miss
//! - Caches sessions in memory for performance with LRU eviction
//! - Optionally expires sessions after a TTL period
//! - Persists turn data back to workstream storage
//!
//! This makes workstream sessions the single source of truth while
//! maintaining the in-memory performance needed for active sessions.
//!
//! The cache uses [`arawn_session::SessionCache`] with a
//! [`WorkstreamPersistence`] hook that loads from and saves to
//! workstream JSONL storage.

use std::sync::Arc;
use std::time::Duration;

use arawn_agent::{Session, SessionId, ToolCall, ToolResultRecord, Turn};
use arawn_session::{CacheConfig, PersistenceHook};
use arawn_types::HasSessionConfig;
use arawn_workstream::{ReconstructedSession, SessionLoader, WorkstreamManager};
use tracing::{debug, warn};

/// Default maximum number of sessions to cache.
/// With ~100KB average session size, this uses ~1GB of memory.
const DEFAULT_MAX_SESSIONS: usize = 10_000;

/// Default TTL for sessions (1 hour).
const DEFAULT_SESSION_TTL: Option<Duration> = Some(Duration::from_secs(3600));

/// Error type for session cache operations.
#[derive(Debug, thiserror::Error)]
pub enum SessionCacheError {
    #[error("Session not found: {0}")]
    NotFound(String),
    #[error("Workstream not found: {0}")]
    WorkstreamNotFound(String),
    #[error("No workstream manager configured")]
    NoWorkstreamManager,
    #[error("Workstream error: {0}")]
    Workstream(#[from] arawn_workstream::WorkstreamError),
    #[error("Cache error: {0}")]
    Cache(#[from] arawn_session::Error),
}

pub type Result<T> = std::result::Result<T, SessionCacheError>;

/// Persistence hook that loads/saves sessions from workstream JSONL storage.
///
/// When the cache misses, this hook loads the session from the workstream's
/// JSONL message store. If no messages are found, it returns an empty session
/// (matching the previous behavior where a cache miss creates a fresh session).
pub struct WorkstreamPersistence {
    workstreams: Option<Arc<WorkstreamManager>>,
}

impl PersistenceHook for WorkstreamPersistence {
    type Value = Session;

    fn load(&self, session_id: &str, context_id: &str) -> arawn_session::Result<Option<Session>> {
        let Some(ref manager) = self.workstreams else {
            // No workstream manager — return an empty session
            let sid = parse_session_id(session_id)?;
            return Ok(Some(Session::with_id(sid)));
        };

        let loader = SessionLoader::new(manager.message_store());
        match loader
            .load_session(context_id, session_id)
            .map_err(|e| arawn_session::Error::Persistence(e.to_string()))?
        {
            Some(reconstructed) => {
                let sid = parse_session_id(session_id)?;
                let session = convert_reconstructed_to_session(&reconstructed, sid);
                Ok(Some(session))
            }
            None => {
                // No messages found — return an empty session
                let sid = parse_session_id(session_id)?;
                Ok(Some(Session::with_id(sid)))
            }
        }
    }

    fn save(
        &self,
        _session_id: &str,
        _context_id: &str,
        _value: &Session,
    ) -> arawn_session::Result<()> {
        // Turns are saved explicitly via save_turn(); no bulk save needed.
        Ok(())
    }

    fn delete(&self, _session_id: &str, _context_id: &str) -> arawn_session::Result<()> {
        // Session deletion from workstream storage is handled elsewhere.
        Ok(())
    }
}

/// Parse a session ID string into a `SessionId`.
fn parse_session_id(session_id: &str) -> arawn_session::Result<SessionId> {
    uuid::Uuid::parse_str(session_id)
        .map(SessionId::from_uuid)
        .map_err(|e| arawn_session::Error::Persistence(format!("Invalid session ID: {e}")))
}

/// Session cache that loads from and persists to workstream storage.
///
/// Uses LRU eviction to prevent unbounded memory growth. Least recently
/// used sessions are evicted when the cache reaches capacity. Sessions
/// can also expire based on TTL if configured.
///
/// Backed by [`arawn_session::SessionCache`] with a [`WorkstreamPersistence`]
/// hook for workstream JSONL storage.
#[derive(Clone)]
pub struct SessionCache {
    /// Generic cache with workstream persistence.
    inner: arawn_session::SessionCache<WorkstreamPersistence>,
    /// Workstream manager for turn persistence.
    workstreams: Option<Arc<WorkstreamManager>>,
}

impl SessionCache {
    /// Create a new session cache with default capacity and TTL.
    pub fn new(workstreams: Option<Arc<WorkstreamManager>>) -> Self {
        Self::with_config(workstreams, DEFAULT_MAX_SESSIONS, DEFAULT_SESSION_TTL)
    }

    /// Create a session cache from a configuration provider.
    ///
    /// This allows any type implementing `HasSessionConfig` to configure
    /// the cache, enabling decoupled configuration passing.
    pub fn from_session_config<C: HasSessionConfig>(
        workstreams: Option<Arc<WorkstreamManager>>,
        config: &C,
    ) -> Self {
        Self::with_config(workstreams, config.max_sessions(), config.session_ttl())
    }

    /// Create a new session cache with specified capacity.
    pub fn with_capacity(workstreams: Option<Arc<WorkstreamManager>>, max_sessions: usize) -> Self {
        Self::with_config(workstreams, max_sessions, DEFAULT_SESSION_TTL)
    }

    /// Create a new session cache with full configuration.
    pub fn with_config(
        workstreams: Option<Arc<WorkstreamManager>>,
        max_sessions: usize,
        ttl: Option<Duration>,
    ) -> Self {
        let mut cache_config = CacheConfig::new().with_max_sessions(max_sessions);
        if let Some(ttl) = ttl {
            cache_config = cache_config.with_ttl(ttl);
        }

        let persistence = WorkstreamPersistence {
            workstreams: workstreams.clone(),
        };

        Self {
            inner: arawn_session::SessionCache::with_persistence(cache_config, persistence),
            workstreams,
        }
    }

    /// Get the current number of cached sessions.
    pub async fn len(&self) -> usize {
        self.inner.len().await
    }

    /// Check if the cache is empty.
    pub async fn is_empty(&self) -> bool {
        self.inner.is_empty().await
    }

    /// Clean up expired sessions.
    ///
    /// Returns the number of sessions that were cleaned up.
    pub async fn cleanup_expired(&self) -> usize {
        self.inner.cleanup_expired().await
    }

    /// Get a session from cache or load from workstream.
    ///
    /// Returns the session and its associated workstream ID.
    /// This marks the session as recently used in the LRU cache and resets TTL.
    pub async fn get_or_load(
        &self,
        session_id: SessionId,
        workstream_id: &str,
    ) -> Result<(Session, String)> {
        let session_id_str = session_id.to_string();

        let session = self
            .inner
            .get_or_load(&session_id_str, workstream_id)
            .await?;

        Ok((session, workstream_id.to_string()))
    }

    /// Create a new session and add it to the cache.
    pub async fn create_session(&self, workstream_id: &str) -> (SessionId, Session) {
        let session = Session::new();
        let session_id = session.id;
        let session_id_str = session_id.to_string();

        let _ = self
            .inner
            .insert(&session_id_str, workstream_id, session.clone())
            .await;

        (session_id, session)
    }

    /// Get or create a session.
    ///
    /// If `session_id` is Some and exists in cache, returns it.
    /// If `session_id` is Some but not in cache, attempts to load from workstream.
    /// If `session_id` is None, creates a new session.
    pub async fn get_or_create(
        &self,
        session_id: Option<SessionId>,
        workstream_id: &str,
    ) -> Result<(SessionId, Session, bool)> {
        match session_id {
            Some(id) => {
                let session_id_str = id.to_string();

                // Check cache first (peek to avoid loading)
                if let Some(session) = self.inner.peek(&session_id_str).await {
                    // Touch the TTL by doing a get_or_load
                    let _ = self.inner.get_or_load(&session_id_str, workstream_id).await;
                    return Ok((id, session, false));
                }

                // Try to load from workstream
                let (session, _) = self.get_or_load(id, workstream_id).await?;
                Ok((id, session, false))
            }
            None => {
                let (id, session) = self.create_session(workstream_id).await;
                Ok((id, session, true))
            }
        }
    }

    /// Check if a session exists in cache (and is not expired).
    pub async fn contains(&self, session_id: &SessionId) -> bool {
        self.inner.contains(&session_id.to_string()).await
    }

    /// Get a session from cache only (no workstream loading).
    /// Does NOT update LRU order (peek operation).
    pub async fn get(&self, session_id: &SessionId) -> Option<Session> {
        self.inner.peek(&session_id.to_string()).await
    }

    /// Get the workstream ID for a cached session.
    /// Does NOT update LRU order (peek operation).
    pub async fn get_workstream_id(&self, session_id: &SessionId) -> Option<String> {
        self.inner.peek_context_id(&session_id.to_string()).await
    }

    /// Update a session in cache.
    pub async fn update(&self, session_id: SessionId, session: Session) -> Result<()> {
        let session_id_str = session_id.to_string();

        // Get the existing context_id
        if let Some(context_id) = self.inner.peek_context_id(&session_id_str).await {
            self.inner
                .update(&session_id_str, &context_id, session, false)
                .await?;
        } else {
            warn!(session_id = %session_id, "Updating session that was not in cache");
        }

        Ok(())
    }

    /// Save a completed turn to workstream storage.
    pub async fn save_turn(
        &self,
        session_id: SessionId,
        turn: &Turn,
        workstream_id: &str,
    ) -> Result<()> {
        if let Some(ref manager) = self.workstreams {
            let loader = SessionLoader::new(manager.message_store());
            let session_id_str = session_id.to_string();

            // Convert tool calls
            let tool_calls: Vec<_> = turn
                .tool_calls
                .iter()
                .map(|tc| (tc.id.clone(), tc.name.clone(), tc.arguments.clone()))
                .collect();

            // Convert tool results
            let tool_results: Vec<_> = turn
                .tool_results
                .iter()
                .map(|tr| (tr.tool_call_id.clone(), tr.success, tr.content.clone()))
                .collect();

            loader.save_turn(
                workstream_id,
                &session_id_str,
                &turn.user_message,
                &tool_calls,
                &tool_results,
                turn.assistant_response.as_deref(),
            )?;

            debug!(
                session_id = %session_id,
                workstream_id = %workstream_id,
                tool_call_count = tool_calls.len(),
                "Turn saved to workstream"
            );
        }

        Ok(())
    }

    /// Remove a session from cache.
    pub async fn remove(&self, session_id: &SessionId) -> Option<Session> {
        let session_id_str = session_id.to_string();

        // Get context_id before removal (needed for persistence hook)
        let context_id = self
            .inner
            .peek_context_id(&session_id_str)
            .await
            .unwrap_or_default();

        self.inner
            .remove(&session_id_str, &context_id)
            .await
            .ok()
            .flatten()
    }

    /// Invalidate a cached session (e.g., after reassignment).
    pub async fn invalidate(&self, session_id: &SessionId) {
        self.inner.invalidate(&session_id.to_string()).await;
    }

    /// List all cached sessions (excludes expired).
    pub async fn list_cached(&self) -> Vec<(SessionId, String)> {
        self.inner
            .list_cached()
            .await
            .into_iter()
            .filter_map(|(id_str, context_id)| {
                uuid::Uuid::parse_str(&id_str)
                    .ok()
                    .map(|uuid| (SessionId::from_uuid(uuid), context_id))
            })
            .collect()
    }

    /// Get all sessions (for backwards compatibility, excludes expired).
    pub async fn all_sessions(&self) -> std::collections::HashMap<SessionId, Session> {
        self.inner
            .for_each(|id_str, entry| {
                uuid::Uuid::parse_str(id_str)
                    .ok()
                    .map(|uuid| (SessionId::from_uuid(uuid), entry.value.clone()))
            })
            .await
            .into_iter()
            .flatten()
            .collect()
    }

    /// Direct access to cache for backwards compatibility during migration.
    pub async fn with_session<F, R>(&self, session_id: &SessionId, f: F) -> Option<R>
    where
        F: FnOnce(&Session) -> R,
    {
        self.inner.with_ref(&session_id.to_string(), f).await
    }

    /// Direct mutable access to cache for backwards compatibility during migration.
    pub async fn with_session_mut<F, R>(&self, session_id: &SessionId, f: F) -> Option<R>
    where
        F: FnOnce(&mut Session) -> R,
    {
        self.inner.with_mut(&session_id.to_string(), f).await
    }

    /// Insert a session directly into cache.
    pub async fn insert(&self, session_id: SessionId, session: Session, workstream_id: &str) {
        let session_id_str = session_id.to_string();
        let _ = self
            .inner
            .insert(&session_id_str, workstream_id, session)
            .await;
    }
}

/// Convert a reconstructed session from workstream to an agent Session.
fn convert_reconstructed_to_session(
    reconstructed: &ReconstructedSession,
    session_id: SessionId,
) -> Session {
    let mut session = Session::with_id(session_id);
    session.created_at = reconstructed.created_at;
    session.updated_at = reconstructed.updated_at;

    for rturn in &reconstructed.turns {
        let turn = session.start_turn(&rturn.user_message);
        turn.started_at = rturn.started_at;

        // Add tool calls
        for tc in &rturn.tool_calls {
            turn.add_tool_call(ToolCall {
                id: tc.id.clone(),
                name: tc.name.clone(),
                arguments: tc.arguments.clone(),
            });
        }

        // Add tool results
        for tr in &rturn.tool_results {
            turn.add_tool_result(ToolResultRecord {
                tool_call_id: tr.tool_call_id.clone(),
                success: tr.success,
                content: tr.content.clone(),
            });
        }

        // Set response if available
        if let Some(ref response) = rturn.assistant_response {
            turn.assistant_response = Some(response.clone());
            turn.completed_at = rturn.completed_at;
        }
    }

    session
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_session() {
        let cache = SessionCache::new(None);

        let (session_id, _session) = cache.create_session("ws-1").await;

        assert!(cache.contains(&session_id).await);
        assert_eq!(
            cache.get_workstream_id(&session_id).await,
            Some("ws-1".to_string())
        );
    }

    #[tokio::test]
    async fn test_get_nonexistent_creates_empty() {
        let cache = SessionCache::new(None);
        let session_id = SessionId::new();

        let result = cache.get_or_load(session_id, "ws-1").await;
        assert!(result.is_ok());

        let (session, ws_id) = result.unwrap();
        assert_eq!(session.id, session_id);
        assert!(session.is_empty());
        assert_eq!(ws_id, "ws-1");
    }

    #[tokio::test]
    async fn test_remove_session() {
        let cache = SessionCache::new(None);

        let (session_id, _) = cache.create_session("ws-1").await;
        assert!(cache.contains(&session_id).await);

        let removed = cache.remove(&session_id).await;
        assert!(removed.is_some());
        assert!(!cache.contains(&session_id).await);
    }

    #[tokio::test]
    async fn test_invalidate_session() {
        let cache = SessionCache::new(None);

        let (session_id, _) = cache.create_session("ws-1").await;
        assert!(cache.contains(&session_id).await);

        cache.invalidate(&session_id).await;
        assert!(!cache.contains(&session_id).await);
    }

    #[tokio::test]
    async fn test_update_session() {
        let cache = SessionCache::new(None);

        let (session_id, mut session) = cache.create_session("ws-1").await;
        session.start_turn("Hello").complete("Hi!");

        cache.update(session_id, session.clone()).await.unwrap();

        let cached = cache.get(&session_id).await.unwrap();
        assert_eq!(cached.turn_count(), 1);
    }

    #[tokio::test]
    async fn test_list_cached() {
        let cache = SessionCache::new(None);

        cache.create_session("ws-1").await;
        cache.create_session("ws-2").await;

        let list = cache.list_cached().await;
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        // Create a cache with capacity of 3
        let cache = SessionCache::with_capacity(None, 3);

        // Add 3 sessions
        let (id1, _) = cache.create_session("ws-1").await;
        let (id2, _) = cache.create_session("ws-2").await;
        let (id3, _) = cache.create_session("ws-3").await;

        assert_eq!(cache.len().await, 3);
        assert!(cache.contains(&id1).await);
        assert!(cache.contains(&id2).await);
        assert!(cache.contains(&id3).await);

        // Add a 4th session - should evict id1 (least recently used)
        let (id4, _) = cache.create_session("ws-4").await;

        assert_eq!(cache.len().await, 3);
        assert!(!cache.contains(&id1).await); // Evicted
        assert!(cache.contains(&id2).await);
        assert!(cache.contains(&id3).await);
        assert!(cache.contains(&id4).await);
    }

    #[tokio::test]
    async fn test_lru_access_updates_order() {
        // Create a cache with capacity of 3
        let cache = SessionCache::with_capacity(None, 3);

        // Add 3 sessions
        let (id1, _) = cache.create_session("ws-1").await;
        let (id2, _) = cache.create_session("ws-2").await;
        let (id3, _) = cache.create_session("ws-3").await;

        // Access id1 to make it recently used
        let _ = cache.get_or_load(id1, "ws-1").await;

        // Add a 4th session - should evict id2 (now least recently used)
        let (id4, _) = cache.create_session("ws-4").await;

        assert_eq!(cache.len().await, 3);
        assert!(cache.contains(&id1).await); // Recently accessed, kept
        assert!(!cache.contains(&id2).await); // Evicted
        assert!(cache.contains(&id3).await);
        assert!(cache.contains(&id4).await);
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        use tokio::time::sleep;

        // Create a cache with 50ms TTL
        let cache = SessionCache::with_config(None, 100, Some(Duration::from_millis(50)));

        let (session_id, _) = cache.create_session("ws-1").await;
        assert!(cache.contains(&session_id).await);

        // Wait for TTL to expire
        sleep(Duration::from_millis(100)).await;

        // Session should be expired
        assert!(!cache.contains(&session_id).await);
    }

    #[tokio::test]
    async fn test_ttl_access_resets_timer() {
        use tokio::time::sleep;

        // Create a cache with 100ms TTL
        let cache = SessionCache::with_config(None, 100, Some(Duration::from_millis(100)));

        let (session_id, _) = cache.create_session("ws-1").await;

        // Wait a bit
        sleep(Duration::from_millis(60)).await;

        // Access the session to reset TTL
        let _ = cache.get_or_load(session_id, "ws-1").await;

        // Wait a bit more (total > 100ms without reset)
        sleep(Duration::from_millis(60)).await;

        // Should still be valid because we accessed it
        assert!(cache.contains(&session_id).await);
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        use tokio::time::sleep;

        // Create a cache with 50ms TTL
        let cache = SessionCache::with_config(None, 100, Some(Duration::from_millis(50)));

        // Create multiple sessions
        cache.create_session("ws-1").await;
        cache.create_session("ws-2").await;
        cache.create_session("ws-3").await;

        assert_eq!(cache.len().await, 3);

        // Wait for TTL to expire
        sleep(Duration::from_millis(100)).await;

        // Clean up expired sessions
        let cleaned = cache.cleanup_expired().await;
        assert_eq!(cleaned, 3);
        assert_eq!(cache.len().await, 0);
    }
}
