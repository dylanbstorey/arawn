//! Tasks API.

use crate::client::ArawnClient;
use crate::error::Result;
use crate::types::{ListTasksResponse, TaskDetail};

/// Query parameters for listing tasks.
#[derive(Debug, Default, serde::Serialize)]
pub struct ListTasksQuery {
    /// Filter by status (pending, running, completed, failed, cancelled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Filter by session ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Maximum number of tasks to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// Tasks API client.
pub struct TasksApi {
    client: ArawnClient,
}

impl TasksApi {
    pub(crate) fn new(client: ArawnClient) -> Self {
        Self { client }
    }

    /// List all tasks.
    pub async fn list(&self) -> Result<ListTasksResponse> {
        self.client.get("tasks").await
    }

    /// List tasks with query parameters.
    pub async fn list_with_query(&self, query: ListTasksQuery) -> Result<ListTasksResponse> {
        self.client.get_with_query("tasks", &query).await
    }

    /// List running tasks.
    pub async fn list_running(&self) -> Result<ListTasksResponse> {
        self.list_with_query(ListTasksQuery {
            status: Some("running".to_string()),
            ..Default::default()
        })
        .await
    }

    /// List tasks for a session.
    pub async fn list_for_session(&self, session_id: &str) -> Result<ListTasksResponse> {
        self.list_with_query(ListTasksQuery {
            session_id: Some(session_id.to_string()),
            ..Default::default()
        })
        .await
    }

    /// Get a task by ID.
    pub async fn get(&self, id: &str) -> Result<TaskDetail> {
        self.client.get(&format!("tasks/{}", id)).await
    }

    /// Cancel a task.
    pub async fn cancel(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("tasks/{}", id)).await
    }
}
