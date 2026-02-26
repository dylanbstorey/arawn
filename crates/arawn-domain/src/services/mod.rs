//! Domain services.
//!
//! This module contains the core domain services that orchestrate
//! Arawn's functionality.

pub mod chat;
pub mod mcp;
pub mod memory;

use std::sync::Arc;

use arawn_agent::{Agent, SessionIndexer};
use arawn_memory::MemoryStore;
use arawn_workstream::{DirectoryManager, WorkstreamManager};
use tracing::info;

pub use mcp::SharedMcpManager;
pub use memory::MemoryService;

/// Domain services facade.
///
/// Provides unified access to all domain services. This is the main entry point
/// for transport layers to interact with Arawn's core functionality.
#[derive(Clone)]
pub struct DomainServices {
    /// Chat service for conversation orchestration.
    chat: chat::ChatService,
    /// MCP service for tool discovery and invocation.
    mcp: mcp::McpService,
    /// Memory service for notes and memory search.
    memory: MemoryService,
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
        memory_store: Option<Arc<MemoryStore>>,
    ) -> Self {
        info!("Initializing domain services");

        let chat = chat::ChatService::new(
            agent.clone(),
            workstreams.clone(),
            directory_manager.clone(),
            indexer.clone(),
        );

        let mcp = mcp::McpService::new(mcp_manager);
        let memory = MemoryService::new(memory_store);

        Self { chat, mcp, memory }
    }

    /// Get the chat service.
    pub fn chat(&self) -> &chat::ChatService {
        &self.chat
    }

    /// Get the MCP service.
    pub fn mcp(&self) -> &mcp::McpService {
        &self.mcp
    }

    /// Get the memory service.
    pub fn memory(&self) -> &MemoryService {
        &self.memory
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
        let services = DomainServices::new(agent, None, None, None, None, None);

        // Services should be created successfully
        // Verify we can access the agent through the chat service
        let _agent = services.chat().agent();
    }

    #[test]
    fn test_domain_services_memory_disabled() {
        let agent = create_test_agent();
        let services = DomainServices::new(agent, None, None, None, None, None);
        assert!(!services.memory().is_enabled());
        assert!(services.memory().store().is_none());
    }

    #[test]
    fn test_domain_services_memory_enabled() {
        let agent = create_test_agent();
        let store = Arc::new(arawn_memory::MemoryStore::open_in_memory().unwrap());
        let services = DomainServices::new(agent, None, None, None, None, Some(store));
        assert!(services.memory().is_enabled());
        assert!(services.memory().store().is_some());
    }
}
