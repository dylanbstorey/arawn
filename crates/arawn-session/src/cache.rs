//! Session cache with LRU eviction and TTL support.

use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Instant;

use lru::LruCache;
use tokio::sync::RwLock;
use tracing::{debug, trace};

use crate::config::CacheConfig;
use crate::error::{Error, Result};
use crate::persistence::{NoPersistence, PersistenceHook, SessionData};
use crate::ttl::TtlTracker;

/// Entry stored in the cache.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Session data.
    pub data: SessionData,

    /// When this entry was inserted into cache.
    pub cached_at: Instant,

    /// Whether this entry has unsaved changes.
    pub dirty: bool,
}

impl CacheEntry {
    /// Create a new cache entry.
    pub fn new(data: SessionData) -> Self {
        Self {
            data,
            cached_at: Instant::now(),
            dirty: false,
        }
    }

    /// Mark the entry as dirty (has unsaved changes).
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Mark the entry as clean (saved).
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
}

/// Inner state protected by RwLock.
struct CacheInner<P: PersistenceHook> {
    /// LRU cache of sessions.
    lru: LruCache<String, CacheEntry>,

    /// TTL tracker for expiration.
    ttl: TtlTracker,

    /// Persistence backend.
    persistence: P,
}

/// Session cache with LRU eviction and optional TTL.
///
/// This cache provides:
/// - LRU eviction when max capacity is reached
/// - Optional TTL-based expiration
/// - Persistence hooks for loading/saving sessions
/// - Thread-safe access via RwLock
pub struct SessionCache<P: PersistenceHook = NoPersistence> {
    inner: Arc<RwLock<CacheInner<P>>>,
    config: CacheConfig,
}

impl SessionCache<NoPersistence> {
    /// Create a new session cache with no persistence backend.
    pub fn new(config: CacheConfig) -> Self {
        Self::with_persistence(config, NoPersistence)
    }
}

impl<P: PersistenceHook> SessionCache<P> {
    /// Create a new session cache with a persistence backend.
    pub fn with_persistence(config: CacheConfig, persistence: P) -> Self {
        let cap =
            NonZeroUsize::new(config.max_sessions).unwrap_or_else(|| NonZeroUsize::new(1).unwrap());

        let inner = CacheInner {
            lru: LruCache::new(cap),
            ttl: TtlTracker::new(config.ttl),
            persistence,
        };

        Self {
            inner: Arc::new(RwLock::new(inner)),
            config,
        }
    }

    /// Get the cache configuration.
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    /// Get the current number of cached sessions.
    pub async fn len(&self) -> usize {
        self.inner.read().await.lru.len()
    }

    /// Check if the cache is empty.
    pub async fn is_empty(&self) -> bool {
        self.inner.read().await.lru.is_empty()
    }

    /// Get a session from cache or load from persistence.
    ///
    /// This marks the session as recently used in the LRU cache
    /// and resets its TTL timer.
    pub async fn get_or_load(&self, session_id: &str, context_id: &str) -> Result<SessionData> {
        // First check cache
        {
            let mut inner = self.inner.write().await;

            // Check if expired
            if inner.ttl.is_expired(session_id) {
                debug!(session_id = %session_id, "Session expired, removing from cache");
                if let Some(entry) = inner.lru.pop(session_id) {
                    let _ = inner
                        .persistence
                        .on_evict(session_id, &entry.data.context_id);
                }
                inner.ttl.remove(session_id);
            } else if let Some(entry) = inner.lru.get(session_id) {
                trace!(session_id = %session_id, "Session found in cache");
                let data = entry.data.clone();
                inner.ttl.touch(session_id);
                return Ok(data);
            }
        }

        // Cache miss - try to load from persistence
        debug!(session_id = %session_id, context_id = %context_id, "Session cache miss, loading from persistence");

        let inner = self.inner.read().await;
        match inner.persistence.load(session_id, context_id)? {
            Some(data) => {
                drop(inner);

                // Insert into cache
                let mut inner = self.inner.write().await;
                let entry = CacheEntry::new(data.clone());
                inner.lru.put(session_id.to_string(), entry);
                inner.ttl.touch(session_id);

                debug!(
                    session_id = %session_id,
                    cache_size = inner.lru.len(),
                    "Session loaded from persistence"
                );

                Ok(data)
            }
            None => Err(Error::NotFound(session_id.to_string())),
        }
    }

    /// Insert a session into the cache.
    ///
    /// If the cache is at capacity, the least recently used session
    /// will be evicted (with on_evict callback).
    pub async fn insert(&self, data: SessionData) -> Result<()> {
        let session_id = data.id.clone();
        let context_id = data.context_id.clone();

        let mut inner = self.inner.write().await;

        // Check if we need to evict
        if inner.lru.len() >= self.config.max_sessions {
            // LRU will handle eviction, but we want to call on_evict
            if let Some((evicted_id, evicted_entry)) = inner.lru.peek_lru() {
                let evicted_id = evicted_id.clone();
                let evicted_context = evicted_entry.data.context_id.clone();
                debug!(
                    session_id = %evicted_id,
                    "Evicting LRU session to make room"
                );
                // The actual eviction happens in put(), but we call on_evict first
                let _ = inner.persistence.on_evict(&evicted_id, &evicted_context);
                inner.ttl.remove(&evicted_id);
            }
        }

        let entry = CacheEntry::new(data);
        inner.lru.put(session_id.clone(), entry);
        inner.ttl.touch(&session_id);

        trace!(
            session_id = %session_id,
            context_id = %context_id,
            cache_size = inner.lru.len(),
            "Session inserted into cache"
        );

        Ok(())
    }

    /// Update a session in the cache and optionally persist.
    pub async fn update(&self, data: SessionData, persist: bool) -> Result<()> {
        let session_id = data.id.clone();

        let mut inner = self.inner.write().await;

        if let Some(entry) = inner.lru.get_mut(&session_id) {
            entry.data = data.clone();
            entry.dirty = !persist;
            inner.ttl.touch(&session_id);

            if persist {
                inner.persistence.save(&data)?;
            }

            Ok(())
        } else {
            // Not in cache - insert it
            drop(inner);
            self.insert(data.clone()).await?;

            if persist {
                let inner = self.inner.read().await;
                inner.persistence.save(&data)?;
            }

            Ok(())
        }
    }

    /// Save a session to persistence.
    pub async fn save(&self, session_id: &str) -> Result<()> {
        let mut inner = self.inner.write().await;

        if let Some(entry) = inner.lru.get_mut(session_id) {
            let data = entry.data.clone();
            entry.mark_clean();
            inner.persistence.save(&data)?;
            Ok(())
        } else {
            Err(Error::NotFound(session_id.to_string()))
        }
    }

    /// Check if a session exists in cache (without loading).
    pub async fn contains(&self, session_id: &str) -> bool {
        let inner = self.inner.read().await;
        inner.lru.contains(session_id) && !inner.ttl.is_expired(session_id)
    }

    /// Peek at a session without updating LRU order or TTL.
    pub async fn peek(&self, session_id: &str) -> Option<SessionData> {
        let inner = self.inner.read().await;
        if inner.ttl.is_expired(session_id) {
            None
        } else {
            inner.lru.peek(session_id).map(|e| e.data.clone())
        }
    }

    /// Remove a session from cache and persistence.
    pub async fn remove(&self, session_id: &str, context_id: &str) -> Result<Option<SessionData>> {
        let mut inner = self.inner.write().await;

        inner.ttl.remove(session_id);
        let entry = inner.lru.pop(session_id);

        if entry.is_some() {
            inner.persistence.delete(session_id, context_id)?;
        }

        Ok(entry.map(|e| e.data))
    }

    /// Invalidate a session (remove from cache only, don't delete from persistence).
    pub async fn invalidate(&self, session_id: &str) {
        let mut inner = self.inner.write().await;
        inner.ttl.remove(session_id);
        if let Some(entry) = inner.lru.pop(session_id) {
            debug!(session_id = %session_id, "Session invalidated from cache");
            let _ = inner
                .persistence
                .on_evict(session_id, &entry.data.context_id);
        }
    }

    /// Clean up expired sessions.
    ///
    /// This is called automatically if `enable_cleanup_task` is true,
    /// but can also be called manually.
    pub async fn cleanup_expired(&self) -> usize {
        let mut inner = self.inner.write().await;
        let expired = inner.ttl.drain_expired();
        let count = expired.len();

        for session_id in expired {
            if let Some(entry) = inner.lru.pop(&session_id) {
                debug!(session_id = %session_id, "Cleaning up expired session");
                let _ = inner
                    .persistence
                    .on_evict(&session_id, &entry.data.context_id);
            }
        }

        if count > 0 {
            debug!(count = count, "Cleaned up expired sessions");
        }

        count
    }

    /// List all cached session IDs with their context IDs.
    pub async fn list_cached(&self) -> Vec<(String, String)> {
        let inner = self.inner.read().await;
        inner
            .lru
            .iter()
            .filter(|(id, _)| !inner.ttl.is_expired(id))
            .map(|(id, entry)| (id.clone(), entry.data.context_id.clone()))
            .collect()
    }

    /// Get cache statistics.
    pub async fn stats(&self) -> CacheStats {
        let inner = self.inner.read().await;
        CacheStats {
            size: inner.lru.len(),
            capacity: self.config.max_sessions,
            ttl_tracked: inner.ttl.len(),
        }
    }
}

impl<P: PersistenceHook> Clone for SessionCache<P> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            config: self.config.clone(),
        }
    }
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Current number of cached sessions.
    pub size: usize,

    /// Maximum capacity.
    pub capacity: usize,

    /// Number of sessions being tracked for TTL.
    pub ttl_tracked: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_insert_and_get() {
        let config = CacheConfig::new().with_max_sessions(10);
        let cache = SessionCache::new(config);

        let data = SessionData::new("session-1", "ctx-1", vec![1, 2, 3]);
        cache.insert(data.clone()).await.unwrap();

        let retrieved = cache.get_or_load("session-1", "ctx-1").await.unwrap();
        assert_eq!(retrieved.id, "session-1");
        assert_eq!(retrieved.context_id, "ctx-1");
        assert_eq!(retrieved.state, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_not_found() {
        let config = CacheConfig::new();
        let cache = SessionCache::new(config);

        let result = cache.get_or_load("nonexistent", "ctx-1").await;
        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let config = CacheConfig::new().with_max_sessions(3);
        let cache = SessionCache::new(config);

        // Insert 3 sessions
        for i in 1..=3 {
            let data = SessionData::new(format!("session-{}", i), "ctx-1", vec![]);
            cache.insert(data).await.unwrap();
        }

        assert_eq!(cache.len().await, 3);

        // Insert a 4th - should evict session-1
        let data = SessionData::new("session-4", "ctx-1", vec![]);
        cache.insert(data).await.unwrap();

        assert_eq!(cache.len().await, 3);
        assert!(!cache.contains("session-1").await);
        assert!(cache.contains("session-2").await);
        assert!(cache.contains("session-3").await);
        assert!(cache.contains("session-4").await);
    }

    #[tokio::test]
    async fn test_lru_access_updates_order() {
        let config = CacheConfig::new().with_max_sessions(3);
        let cache = SessionCache::new(config);

        // Insert 3 sessions
        for i in 1..=3 {
            let data = SessionData::new(format!("session-{}", i), "ctx-1", vec![]);
            cache.insert(data).await.unwrap();
        }

        // Access session-1 to make it recently used
        let _ = cache.get_or_load("session-1", "ctx-1").await;

        // Insert a 4th - should evict session-2 (now LRU)
        let data = SessionData::new("session-4", "ctx-1", vec![]);
        cache.insert(data).await.unwrap();

        assert!(cache.contains("session-1").await); // Recently accessed
        assert!(!cache.contains("session-2").await); // Evicted
        assert!(cache.contains("session-3").await);
        assert!(cache.contains("session-4").await);
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        use std::time::Duration;
        use tokio::time::sleep;

        let config = CacheConfig::new()
            .with_max_sessions(10)
            .with_ttl(Duration::from_millis(50));
        let cache = SessionCache::new(config);

        let data = SessionData::new("session-1", "ctx-1", vec![]);
        cache.insert(data).await.unwrap();

        assert!(cache.contains("session-1").await);

        // Wait for expiration
        sleep(Duration::from_millis(100)).await;

        // Should be expired now
        assert!(!cache.contains("session-1").await);
    }

    #[tokio::test]
    async fn test_touch_resets_ttl() {
        use std::time::Duration;
        use tokio::time::sleep;

        let config = CacheConfig::new()
            .with_max_sessions(10)
            .with_ttl(Duration::from_millis(100));
        let cache = SessionCache::new(config);

        let data = SessionData::new("session-1", "ctx-1", vec![]);
        cache.insert(data).await.unwrap();

        // Wait a bit
        sleep(Duration::from_millis(60)).await;

        // Access to reset TTL
        let _ = cache.get_or_load("session-1", "ctx-1").await;

        // Wait a bit more (total > 100ms without reset)
        sleep(Duration::from_millis(60)).await;

        // Should still be valid because we accessed it
        assert!(cache.contains("session-1").await);
    }

    #[tokio::test]
    async fn test_invalidate() {
        let config = CacheConfig::new();
        let cache = SessionCache::new(config);

        let data = SessionData::new("session-1", "ctx-1", vec![]);
        cache.insert(data).await.unwrap();

        assert!(cache.contains("session-1").await);

        cache.invalidate("session-1").await;

        assert!(!cache.contains("session-1").await);
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        use std::time::Duration;
        use tokio::time::sleep;

        let config = CacheConfig::new()
            .with_max_sessions(10)
            .with_ttl(Duration::from_millis(50));
        let cache = SessionCache::new(config);

        // Insert multiple sessions
        for i in 1..=3 {
            let data = SessionData::new(format!("session-{}", i), "ctx-1", vec![]);
            cache.insert(data).await.unwrap();
        }

        assert_eq!(cache.len().await, 3);

        // Wait for expiration
        sleep(Duration::from_millis(100)).await;

        // Cleanup
        let cleaned = cache.cleanup_expired().await;
        assert_eq!(cleaned, 3);
        assert_eq!(cache.len().await, 0);
    }

    #[tokio::test]
    async fn test_stats() {
        let config = CacheConfig::new().with_max_sessions(100);
        let cache = SessionCache::new(config);

        for i in 1..=5 {
            let data = SessionData::new(format!("session-{}", i), "ctx-1", vec![]);
            cache.insert(data).await.unwrap();
        }

        let stats = cache.stats().await;
        assert_eq!(stats.size, 5);
        assert_eq!(stats.capacity, 100);
    }
}
