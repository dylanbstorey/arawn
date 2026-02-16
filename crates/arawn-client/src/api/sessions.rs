//! Sessions API.

use crate::client::ArawnClient;
use crate::error::Result;
use crate::types::{
    CreateSessionRequest, ListSessionsResponse, SessionDetail, SessionMessagesResponse,
    UpdateSessionRequest,
};

/// Sessions API client.
pub struct SessionsApi {
    client: ArawnClient,
}

impl SessionsApi {
    pub(crate) fn new(client: ArawnClient) -> Self {
        Self { client }
    }

    /// List all sessions.
    pub async fn list(&self) -> Result<ListSessionsResponse> {
        self.client.get("sessions").await
    }

    /// Get a session by ID.
    pub async fn get(&self, id: &str) -> Result<SessionDetail> {
        self.client.get(&format!("sessions/{}", id)).await
    }

    /// Create a new session.
    pub async fn create(&self, request: CreateSessionRequest) -> Result<SessionDetail> {
        self.client.post("sessions", &request).await
    }

    /// Update a session.
    pub async fn update(&self, id: &str, request: UpdateSessionRequest) -> Result<SessionDetail> {
        self.client.patch(&format!("sessions/{}", id), &request).await
    }

    /// Delete a session.
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("sessions/{}", id)).await
    }

    /// Get messages for a session.
    pub async fn messages(&self, id: &str) -> Result<SessionMessagesResponse> {
        self.client.get(&format!("sessions/{}/messages", id)).await
    }
}
