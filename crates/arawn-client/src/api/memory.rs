//! Memory API.

use crate::client::ArawnClient;
use crate::error::Result;
use crate::types::{MemorySearchResponse, StoreMemoryRequest, StoreMemoryResponse};

/// Query parameters for memory search.
#[derive(Debug, Default, serde::Serialize)]
pub struct MemorySearchQuery {
    /// Search query text.
    pub q: String,
    /// Maximum results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    /// Filter by session ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// Memory API client.
pub struct MemoryApi {
    client: ArawnClient,
}

impl MemoryApi {
    pub(crate) fn new(client: ArawnClient) -> Self {
        Self { client }
    }

    /// Search memories.
    pub async fn search(&self, query: &str) -> Result<MemorySearchResponse> {
        self.search_with_options(MemorySearchQuery {
            q: query.to_string(),
            ..Default::default()
        })
        .await
    }

    /// Search memories with options.
    pub async fn search_with_options(
        &self,
        query: MemorySearchQuery,
    ) -> Result<MemorySearchResponse> {
        self.client.get_with_query("memory/search", &query).await
    }

    /// Search memories in a specific session.
    pub async fn search_in_session(
        &self,
        query: &str,
        session_id: &str,
    ) -> Result<MemorySearchResponse> {
        self.search_with_options(MemorySearchQuery {
            q: query.to_string(),
            session_id: Some(session_id.to_string()),
            ..Default::default()
        })
        .await
    }

    /// Store a memory directly.
    pub async fn store(&self, request: StoreMemoryRequest) -> Result<StoreMemoryResponse> {
        self.client.post("memory", &request).await
    }

    /// Store a simple fact.
    pub async fn store_fact(&self, content: impl Into<String>) -> Result<StoreMemoryResponse> {
        self.store(StoreMemoryRequest {
            content: content.into(),
            content_type: "fact".to_string(),
            session_id: None,
            metadata: Default::default(),
            confidence: 0.8,
        })
        .await
    }

    /// Delete a memory by ID.
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("memory/{}", id)).await
    }
}
