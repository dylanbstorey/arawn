//! Persistence hooks for session loading and saving.
//!
//! This module defines traits that allow the session cache to be decoupled
//! from specific storage backends. The [`PersistenceHook`] trait uses an
//! associated `Value` type so backends can store domain-specific types
//! (e.g., a rich `Session` object) without serialization overhead.

use crate::error::Result;

/// Data container for session state.
///
/// This is a generic container that can hold any session-related data.
/// It serves as the default `Value` type for [`NoPersistence`] and can
/// be used by backends that work with opaque byte blobs.
#[derive(Debug, Clone)]
pub struct SessionData {
    /// Unique identifier for the session.
    pub id: String,

    /// Context identifier (e.g., workstream ID).
    pub context_id: String,

    /// Serialized session state (application-specific format).
    pub state: Vec<u8>,

    /// When the session was created.
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,

    /// When the session was last updated.
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl SessionData {
    /// Create a new session data container.
    pub fn new(id: impl Into<String>, context_id: impl Into<String>, state: Vec<u8>) -> Self {
        Self {
            id: id.into(),
            context_id: context_id.into(),
            state,
            created_at: None,
            updated_at: None,
        }
    }

    /// Set creation timestamp.
    pub fn with_created_at(mut self, ts: chrono::DateTime<chrono::Utc>) -> Self {
        self.created_at = Some(ts);
        self
    }

    /// Set update timestamp.
    pub fn with_updated_at(mut self, ts: chrono::DateTime<chrono::Utc>) -> Self {
        self.updated_at = Some(ts);
        self
    }
}

/// Trait for persistence backends.
///
/// Implement this trait to connect the session cache to your storage backend.
/// The cache calls these methods on cache misses and when saving sessions.
///
/// The associated `Value` type determines what the cache stores in memory.
/// For example, a server might use `Value = Session` (a rich domain object),
/// while a simpler backend might use `Value = SessionData` (opaque bytes).
pub trait PersistenceHook: Send + Sync {
    /// The value type stored in the cache.
    type Value: Clone + Send + Sync + 'static;

    /// Load a session from storage.
    ///
    /// Called when a session is requested but not found in cache.
    /// Return `Ok(None)` if the session doesn't exist in storage.
    fn load(&self, session_id: &str, context_id: &str) -> Result<Option<Self::Value>>;

    /// Save a session to storage.
    ///
    /// Called when a session needs to be persisted (e.g., after updates).
    fn save(&self, session_id: &str, context_id: &str, value: &Self::Value) -> Result<()>;

    /// Delete a session from storage.
    ///
    /// Called when a session is explicitly removed.
    fn delete(&self, session_id: &str, context_id: &str) -> Result<()>;

    /// Called when a session is evicted from cache due to LRU or TTL.
    ///
    /// This is an opportunity to persist any unsaved state before eviction.
    /// Default implementation does nothing.
    fn on_evict(&self, _session_id: &str, _context_id: &str) -> Result<()> {
        Ok(())
    }
}

/// A no-op persistence hook for in-memory only caching.
#[derive(Debug, Clone, Default)]
pub struct NoPersistence;

impl PersistenceHook for NoPersistence {
    type Value = SessionData;

    fn load(&self, _session_id: &str, _context_id: &str) -> Result<Option<SessionData>> {
        Ok(None)
    }

    fn save(&self, _session_id: &str, _context_id: &str, _value: &SessionData) -> Result<()> {
        Ok(())
    }

    fn delete(&self, _session_id: &str, _context_id: &str) -> Result<()> {
        Ok(())
    }
}
