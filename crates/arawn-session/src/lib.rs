//! Session cache with LRU eviction and TTL support.
//!
//! This crate provides a generic caching layer for sessions with:
//! - LRU eviction to prevent unbounded memory growth
//! - Optional TTL for auto-expiring stale sessions
//! - Persistence hooks for loading/saving sessions
//!
//! # Example
//!
//! ```rust,ignore
//! use arawn_session::{SessionCache, CacheConfig};
//!
//! let config = CacheConfig::default()
//!     .with_max_sessions(1000)
//!     .with_ttl(Duration::from_secs(3600));
//!
//! let cache = SessionCache::new(config);
//! ```

mod cache;
mod config;
mod error;
mod persistence;
mod ttl;

pub use cache::{CacheEntry, CacheStats, SessionCache};
pub use config::CacheConfig;
pub use error::{Error, Result};
pub use persistence::{NoPersistence, PersistenceHook, SessionData};
pub use ttl::TtlTracker;
