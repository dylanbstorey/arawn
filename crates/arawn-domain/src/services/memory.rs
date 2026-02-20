//! Memory service for storing and searching memories.
//!
//! The memory service provides a unified interface for memory operations.
//! Note: Memory operations are currently stubbed - the actual memory store
//! is managed internally by the agent during turns.

use std::sync::Arc;

use arawn_agent::Agent;
use tracing::debug;

use crate::error::{DomainError, Result};

/// Result from a memory search.
#[derive(Debug, Clone)]
pub struct MemorySearchResult {
    /// The memory content.
    pub content: String,
    /// Relevance score (0.0 to 1.0).
    pub score: f32,
    /// Memory metadata.
    pub metadata: Option<String>,
}

/// Memory service for storing and searching memories.
///
/// Note: This is a placeholder service. Memory operations are handled
/// internally by the agent during conversation turns. This service
/// exists to provide a future extension point for direct memory access.
#[derive(Clone)]
pub struct MemoryService {
    #[allow(dead_code)]
    agent: Arc<Agent>,
}

impl MemoryService {
    /// Create a new memory service.
    pub fn new(agent: Arc<Agent>) -> Self {
        Self { agent }
    }

    /// Check if memory is enabled.
    ///
    /// Note: Currently always returns false as direct memory access
    /// is not yet exposed by the agent.
    pub fn is_enabled(&self) -> bool {
        // Memory is managed internally by the agent
        // Direct access is not currently exposed
        false
    }

    /// Store a memory.
    pub async fn store(&self, content: &str, _metadata: Option<&str>) -> Result<String> {
        if !self.is_enabled() {
            return Err(DomainError::Memory("Memory not enabled".to_string()));
        }

        debug!(content_len = content.len(), "Storing memory");

        let id = uuid::Uuid::new_v4().to_string();
        debug!(memory_id = %id, "Memory stored");

        Ok(id)
    }

    /// Search memories.
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<MemorySearchResult>> {
        if !self.is_enabled() {
            return Err(DomainError::Memory("Memory not enabled".to_string()));
        }

        debug!(query = query, limit = limit, "Searching memories");
        Ok(vec![])
    }

    /// Delete a memory.
    pub async fn delete(&self, id: &str) -> Result<bool> {
        if !self.is_enabled() {
            return Err(DomainError::Memory("Memory not enabled".to_string()));
        }

        debug!(memory_id = id, "Deleting memory");
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arawn_agent::ToolRegistry;
    use arawn_llm::MockBackend;

    fn create_test_agent() -> Arc<Agent> {
        let backend = MockBackend::with_text("Test");
        Arc::new(
            Agent::builder()
                .with_backend(backend)
                .with_tools(ToolRegistry::new())
                .build()
                .expect("failed to create test agent"),
        )
    }

    #[test]
    fn test_memory_service_creation() {
        let agent = create_test_agent();
        let memory = MemoryService::new(agent);

        // Memory is currently not enabled (stub implementation)
        assert!(!memory.is_enabled());
    }
}
