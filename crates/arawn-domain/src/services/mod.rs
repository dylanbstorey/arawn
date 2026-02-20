//! Domain services.
//!
//! This module contains the core domain services that orchestrate
//! Arawn's functionality.

pub mod chat;
pub mod memory;
pub mod mcp;

use std::sync::Arc;

use arawn_agent::{Agent, SessionIndexer};
use arawn_workstream::{DirectoryManager, WorkstreamManager};
use tracing::info;

pub use mcp::SharedMcpManager;

/// Configuration for domain services.
#[derive(Debug, Clone)]
pub struct DomainConfig {
    /// Maximum sessions to keep in cache.
    pub max_cached_sessions: usize,
    /// Session TTL in seconds.
    pub session_ttl_secs: u64,
    /// Enable memory operations.
    pub memory_enabled: bool,
    /// Enable MCP servers.
    pub mcp_enabled: bool,
}

impl Default for DomainConfig {
    fn default() -> Self {
        Self {
            max_cached_sessions: 100,
            session_ttl_secs: 3600,
            memory_enabled: true,
            mcp_enabled: true,
        }
    }
}

/// Domain services facade.
///
/// Provides unified access to all domain services. This is the main entry point
/// for transport layers to interact with Arawn's core functionality.
#[derive(Clone)]
pub struct DomainServices {
    /// Chat service for conversation orchestration.
    chat: chat::ChatService,
    /// Memory service for storing and searching memories.
    memory: memory::MemoryService,
    /// MCP service for tool discovery and invocation.
    mcp: mcp::McpService,
}

impl DomainServices {
    /// Create new domain services with the given components.
    ///
    /// This is the primary constructor used when setting up the application.
    pub fn new(
        agent: Arc<Agent>,
        workstreams: Option<Arc<WorkstreamManager>>,
        directory_manager: Option<Arc<DirectoryManager>>,
        indexer: Option<Arc<SessionIndexer>>,
        mcp_manager: Option<SharedMcpManager>,
    ) -> Self {
        info!("Initializing domain services");

        let chat = chat::ChatService::new(
            agent.clone(),
            workstreams.clone(),
            directory_manager.clone(),
            indexer.clone(),
        );

        let memory = memory::MemoryService::new(agent.clone());

        let mcp = mcp::McpService::new(mcp_manager);

        Self { chat, memory, mcp }
    }

    /// Get the chat service.
    pub fn chat(&self) -> &chat::ChatService {
        &self.chat
    }

    /// Get the memory service.
    pub fn memory(&self) -> &memory::MemoryService {
        &self.memory
    }

    /// Get the MCP service.
    pub fn mcp(&self) -> &mcp::McpService {
        &self.mcp
    }

    /// Get the underlying agent.
    pub fn agent(&self) -> &Arc<Agent> {
        self.chat.agent()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arawn_agent::ToolRegistry;
    use arawn_llm::MockBackend;

    fn create_test_agent() -> Arc<Agent> {
        let backend = MockBackend::with_text("Test response");
        Arc::new(
            Agent::builder()
                .with_backend(backend)
                .with_tools(ToolRegistry::new())
                .build()
                .expect("failed to create test agent"),
        )
    }

    #[test]
    fn test_domain_services_creation() {
        let agent = create_test_agent();
        let services = DomainServices::new(agent, None, None, None, None);

        // Services should be created successfully
        // Verify we can access the agent through the chat service
        let _agent = services.chat().agent();
    }
}
