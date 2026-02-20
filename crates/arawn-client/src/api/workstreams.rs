//! Workstreams API.

use crate::client::ArawnClient;
use crate::error::Result;
use crate::types::{
    CreateWorkstreamRequest, ListMessagesResponse, ListWorkstreamSessionsResponse,
    ListWorkstreamsResponse, PromoteRequest, SendMessageRequest, UpdateWorkstreamRequest,
    Workstream, WorkstreamMessage,
};

/// Query parameters for listing messages.
#[derive(Debug, Default, serde::Serialize)]
pub struct ListMessagesQuery {
    /// Filter messages since this timestamp (RFC 3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
}

/// Query parameters for listing workstreams.
#[derive(Debug, Default, serde::Serialize)]
pub struct ListWorkstreamsQuery {
    /// Include archived workstreams in the response.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub include_archived: bool,
}

/// Workstreams API client.
pub struct WorkstreamsApi {
    client: ArawnClient,
}

impl WorkstreamsApi {
    pub(crate) fn new(client: ArawnClient) -> Self {
        Self { client }
    }

    /// List all active workstreams.
    pub async fn list(&self) -> Result<ListWorkstreamsResponse> {
        self.client.get("workstreams").await
    }

    /// List all workstreams including archived.
    pub async fn list_all(&self) -> Result<ListWorkstreamsResponse> {
        let query = ListWorkstreamsQuery {
            include_archived: true,
        };
        self.client.get_with_query("workstreams", &query).await
    }

    /// Get a workstream by ID.
    pub async fn get(&self, id: &str) -> Result<Workstream> {
        self.client.get(&format!("workstreams/{}", id)).await
    }

    /// Create a new workstream.
    pub async fn create(&self, request: CreateWorkstreamRequest) -> Result<Workstream> {
        self.client.post("workstreams", &request).await
    }

    /// Update a workstream.
    pub async fn update(&self, id: &str, request: UpdateWorkstreamRequest) -> Result<Workstream> {
        self.client
            .patch(&format!("workstreams/{}", id), &request)
            .await
    }

    /// Delete (archive) a workstream.
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("workstreams/{}", id)).await
    }

    /// Send a message to a workstream.
    pub async fn send_message(
        &self,
        workstream_id: &str,
        request: SendMessageRequest,
    ) -> Result<WorkstreamMessage> {
        self.client
            .post(&format!("workstreams/{}/messages", workstream_id), &request)
            .await
    }

    /// List messages in a workstream.
    pub async fn messages(&self, workstream_id: &str) -> Result<ListMessagesResponse> {
        self.client
            .get(&format!("workstreams/{}/messages", workstream_id))
            .await
    }

    /// List messages since a timestamp.
    pub async fn messages_since(
        &self,
        workstream_id: &str,
        since: &str,
    ) -> Result<ListMessagesResponse> {
        let query = ListMessagesQuery {
            since: Some(since.to_string()),
        };
        self.client
            .get_with_query(&format!("workstreams/{}/messages", workstream_id), &query)
            .await
    }

    /// List sessions in a workstream.
    pub async fn sessions(&self, workstream_id: &str) -> Result<ListWorkstreamSessionsResponse> {
        self.client
            .get(&format!("workstreams/{}/sessions", workstream_id))
            .await
    }

    /// Promote the scratch workstream to a named workstream.
    pub async fn promote_scratch(&self, request: PromoteRequest) -> Result<Workstream> {
        self.client
            .post("workstreams/scratch/promote", &request)
            .await
    }
}
