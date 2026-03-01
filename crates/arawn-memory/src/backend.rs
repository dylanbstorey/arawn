//! Memory backend trait for pluggable storage.
//!
//! This module defines the `MemoryBackend` trait that allows different storage
//! implementations (SQLite, Redis, mock, etc.) to be used interchangeably.
//!
//! # Example
//!
//! ```ignore
//! use arawn_memory::{MemoryBackend, Memory, MemoryId, ContentType};
//!
//! // Use the default SQLite backend
//! let backend = SqliteMemoryBackend::open("~/.arawn/memory.db")?;
//!
//! // Or use a mock for testing
//! let mock = MockMemoryBackend::new();
//!
//! // Both implement the same trait
//! fn process_memories(backend: &dyn MemoryBackend) {
//!     let memories = backend.list(None, 10, 0).unwrap();
//!     // ...
//! }
//! ```

use crate::error::Result;
use crate::types::{ContentType, Memory, MemoryId};

/// Trait for memory storage backends.
///
/// This trait defines the core operations for storing and retrieving memories.
/// Implementations can use different storage technologies (SQLite, Redis, etc.)
/// while providing a consistent interface.
///
/// # Thread Safety
///
/// All implementations must be `Send + Sync` to allow sharing across threads.
pub trait MemoryBackend: Send + Sync {
    /// Insert a new memory into storage.
    ///
    /// # Errors
    ///
    /// Returns an error if the memory could not be inserted (e.g., duplicate ID,
    /// storage unavailable).
    fn insert(&self, memory: &Memory) -> Result<()>;

    /// Get a memory by its unique ID.
    ///
    /// Returns `Ok(None)` if the memory does not exist.
    fn get(&self, id: MemoryId) -> Result<Option<Memory>>;

    /// Update an existing memory.
    ///
    /// # Errors
    ///
    /// Returns `NotFound` error if the memory does not exist.
    fn update(&self, memory: &Memory) -> Result<()>;

    /// Delete a memory by ID.
    ///
    /// Returns `true` if the memory existed and was deleted, `false` if not found.
    fn delete(&self, id: MemoryId) -> Result<bool>;

    /// List memories with optional filtering and pagination.
    ///
    /// # Arguments
    ///
    /// * `content_type` - Filter by content type (None for all types)
    /// * `limit` - Maximum number of results to return
    /// * `offset` - Number of results to skip (for pagination)
    ///
    /// Results are ordered by `created_at` descending (newest first).
    fn list(
        &self,
        content_type: Option<ContentType>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Memory>>;

    /// Count memories with optional filtering.
    fn count(&self, content_type: Option<ContentType>) -> Result<usize>;

    /// Record an access to a memory (updates accessed_at and access_count).
    ///
    /// # Errors
    ///
    /// Returns `NotFound` error if the memory does not exist.
    fn touch(&self, id: MemoryId) -> Result<()>;
}

/// Extension trait for advanced memory operations.
///
/// These operations are optional and may not be supported by all backends.
/// The default implementations return `Ok(Vec::new())` or `Ok(())`.
pub trait MemoryBackendExt: MemoryBackend {
    /// Find memories that contradict a given subject/predicate pair.
    ///
    /// Used for detecting when a new fact conflicts with existing knowledge.
    fn find_contradictions(&self, subject: &str, predicate: &str) -> Result<Vec<Memory>> {
        let _ = (subject, predicate);
        Ok(Vec::new())
    }

    /// Mark a memory as superseded by another.
    ///
    /// Sets the old memory's `superseded` flag to true and links it to the new memory.
    fn supersede(&self, old_id: MemoryId, new_id: MemoryId) -> Result<()> {
        let _ = (old_id, new_id);
        Ok(())
    }

    /// Reinforce a memory (increment reinforcement count).
    ///
    /// Called when a fact is confirmed by new evidence.
    fn reinforce(&self, id: MemoryId) -> Result<()> {
        let _ = id;
        Ok(())
    }

    /// Update the last_accessed timestamp without incrementing access_count.
    fn update_last_accessed(&self, id: MemoryId) -> Result<()> {
        let _ = id;
        Ok(())
    }
}

/// Mock memory backend for testing.
///
/// Stores memories in memory using a simple HashMap.
/// Useful for unit tests that don't need persistence.
#[cfg(test)]
#[derive(Debug, Default)]
pub struct MockMemoryBackend {
    memories: std::sync::Mutex<std::collections::HashMap<MemoryId, Memory>>,
}

#[cfg(test)]
impl MockMemoryBackend {
    /// Create a new empty mock backend.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the number of stored memories.
    pub fn len(&self) -> usize {
        self.memories.lock().unwrap().len()
    }

    /// Check if the backend is empty.
    pub fn is_empty(&self) -> bool {
        self.memories.lock().unwrap().is_empty()
    }

    /// Clear all stored memories.
    pub fn clear(&self) {
        self.memories.lock().unwrap().clear();
    }
}

#[cfg(test)]
impl MemoryBackend for MockMemoryBackend {
    fn insert(&self, memory: &Memory) -> Result<()> {
        let mut map = self.memories.lock().unwrap();
        map.insert(memory.id, memory.clone());
        Ok(())
    }

    fn get(&self, id: MemoryId) -> Result<Option<Memory>> {
        let map = self.memories.lock().unwrap();
        Ok(map.get(&id).cloned())
    }

    fn update(&self, memory: &Memory) -> Result<()> {
        let mut map = self.memories.lock().unwrap();
        if let std::collections::hash_map::Entry::Occupied(mut e) = map.entry(memory.id) {
            e.insert(memory.clone());
            Ok(())
        } else {
            Err(crate::error::MemoryError::NotFound(format!(
                "Memory {}",
                memory.id
            )))
        }
    }

    fn delete(&self, id: MemoryId) -> Result<bool> {
        let mut map = self.memories.lock().unwrap();
        Ok(map.remove(&id).is_some())
    }

    fn list(
        &self,
        content_type: Option<ContentType>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Memory>> {
        let map = self.memories.lock().unwrap();
        let mut memories: Vec<_> = map
            .values()
            .filter(|m| content_type.is_none() || Some(m.content_type) == content_type)
            .cloned()
            .collect();

        // Sort by created_at descending
        memories.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Apply pagination
        let paginated: Vec<_> = memories.into_iter().skip(offset).take(limit).collect();

        Ok(paginated)
    }

    fn count(&self, content_type: Option<ContentType>) -> Result<usize> {
        let map = self.memories.lock().unwrap();
        let count = map
            .values()
            .filter(|m| content_type.is_none() || Some(m.content_type) == content_type)
            .count();
        Ok(count)
    }

    fn touch(&self, id: MemoryId) -> Result<()> {
        let mut map = self.memories.lock().unwrap();
        if let Some(memory) = map.get_mut(&id) {
            memory.access_count += 1;
            memory.accessed_at = chrono::Utc::now();
            Ok(())
        } else {
            Err(crate::error::MemoryError::NotFound(format!(
                "Memory {}",
                id
            )))
        }
    }
}

#[cfg(test)]
impl MemoryBackendExt for MockMemoryBackend {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ContentType;

    #[test]
    fn test_mock_backend_insert_and_get() {
        let backend = MockMemoryBackend::new();

        let memory = Memory::new(ContentType::Note, "Test content");
        backend.insert(&memory).unwrap();

        let fetched = backend.get(memory.id).unwrap().unwrap();
        assert_eq!(fetched.content, "Test content");
    }

    #[test]
    fn test_mock_backend_update() {
        let backend = MockMemoryBackend::new();

        let mut memory = Memory::new(ContentType::Note, "Original");
        backend.insert(&memory).unwrap();

        memory.content = "Updated".to_string();
        backend.update(&memory).unwrap();

        let fetched = backend.get(memory.id).unwrap().unwrap();
        assert_eq!(fetched.content, "Updated");
    }

    #[test]
    fn test_mock_backend_delete() {
        let backend = MockMemoryBackend::new();

        let memory = Memory::new(ContentType::Note, "Test");
        backend.insert(&memory).unwrap();

        assert!(backend.delete(memory.id).unwrap());
        assert!(backend.get(memory.id).unwrap().is_none());
        assert!(!backend.delete(memory.id).unwrap());
    }

    #[test]
    fn test_mock_backend_list_and_count() {
        let backend = MockMemoryBackend::new();

        for i in 0..5 {
            let memory = Memory::new(ContentType::Note, format!("Note {}", i));
            backend.insert(&memory).unwrap();
        }
        for i in 0..3 {
            let memory = Memory::new(ContentType::Fact, format!("Fact {}", i));
            backend.insert(&memory).unwrap();
        }

        assert_eq!(backend.count(None).unwrap(), 8);
        assert_eq!(backend.count(Some(ContentType::Note)).unwrap(), 5);
        assert_eq!(backend.count(Some(ContentType::Fact)).unwrap(), 3);

        let all = backend.list(None, 100, 0).unwrap();
        assert_eq!(all.len(), 8);

        let notes = backend.list(Some(ContentType::Note), 100, 0).unwrap();
        assert_eq!(notes.len(), 5);

        let page = backend.list(None, 3, 2).unwrap();
        assert_eq!(page.len(), 3);
    }

    #[test]
    fn test_mock_backend_touch() {
        let backend = MockMemoryBackend::new();

        let memory = Memory::new(ContentType::Note, "Test");
        backend.insert(&memory).unwrap();

        backend.touch(memory.id).unwrap();
        backend.touch(memory.id).unwrap();

        let fetched = backend.get(memory.id).unwrap().unwrap();
        assert_eq!(fetched.access_count, 2);
    }
}
