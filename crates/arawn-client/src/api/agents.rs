//! Agents API.

use crate::client::ArawnClient;
use crate::error::Result;
use crate::types::{AgentDetail, ListAgentsResponse};

/// Agents API client.
pub struct AgentsApi {
    client: ArawnClient,
}

impl AgentsApi {
    pub(crate) fn new(client: ArawnClient) -> Self {
        Self { client }
    }

    /// List all available agents.
    pub async fn list(&self) -> Result<ListAgentsResponse> {
        self.client.get("agents").await
    }

    /// Get an agent by ID.
    pub async fn get(&self, id: &str) -> Result<AgentDetail> {
        self.client.get(&format!("agents/{}", id)).await
    }

    /// Get the main/default agent.
    pub async fn main(&self) -> Result<AgentDetail> {
        self.get("main").await
    }
}
