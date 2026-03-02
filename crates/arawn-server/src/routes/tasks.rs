//! Task management endpoints.
//!
//! Provides endpoints for listing, viewing, and cancelling long-running tasks.

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::Identity;
use crate::error::ServerError;
use crate::state::{AppState, TaskStatus, TrackedTask};

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/// Query params for listing tasks.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ListTasksQuery {
    /// Filter by status.
    #[serde(default)]
    pub status: Option<String>,
    /// Filter by session ID.
    #[serde(default)]
    pub session_id: Option<String>,
    /// Maximum number of tasks to return.
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

/// Summary info for a task.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaskSummary {
    /// Task ID.
    pub id: String,
    /// Task type.
    pub task_type: String,
    /// Current status.
    #[schema(value_type = String)]
    pub status: TaskStatus,
    /// Progress percentage (0-100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<u8>,
    /// Creation time.
    pub created_at: String,
}

/// Full task details.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaskDetail {
    /// Task ID.
    pub id: String,
    /// Task type.
    pub task_type: String,
    /// Current status.
    #[schema(value_type = String)]
    pub status: TaskStatus,
    /// Progress percentage (0-100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<u8>,
    /// Status message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Associated session ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Creation time.
    pub created_at: String,
    /// Start time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    /// Completion time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    /// Error message if failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Response for listing tasks.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListTasksResponse {
    /// List of tasks.
    pub tasks: Vec<TaskSummary>,
    /// Total count.
    pub total: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn task_to_summary(task: &TrackedTask) -> TaskSummary {
    TaskSummary {
        id: task.id.clone(),
        task_type: task.task_type.clone(),
        status: task.status.clone(),
        progress: task.progress,
        created_at: task.created_at.to_rfc3339(),
    }
}

fn task_to_detail(task: &TrackedTask) -> TaskDetail {
    TaskDetail {
        id: task.id.clone(),
        task_type: task.task_type.clone(),
        status: task.status.clone(),
        progress: task.progress,
        message: task.message.clone(),
        session_id: task.session_id.clone(),
        created_at: task.created_at.to_rfc3339(),
        started_at: task.started_at.map(|dt| dt.to_rfc3339()),
        completed_at: task.completed_at.map(|dt| dt.to_rfc3339()),
        error: task.error.clone(),
    }
}

fn parse_status(s: &str) -> Option<TaskStatus> {
    match s.to_lowercase().as_str() {
        "pending" => Some(TaskStatus::Pending),
        "running" => Some(TaskStatus::Running),
        "completed" => Some(TaskStatus::Completed),
        "failed" => Some(TaskStatus::Failed),
        "cancelled" => Some(TaskStatus::Cancelled),
        _ => None,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// GET /api/v1/tasks - List tasks.
#[utoipa::path(
    get,
    path = "/api/v1/tasks",
    params(
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("session_id" = Option<String>, Query, description = "Filter by session ID"),
        ("limit" = Option<usize>, Query, description = "Maximum tasks to return (default: 50)"),
    ),
    responses(
        (status = 200, description = "List of tasks", body = ListTasksResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = [])),
    tag = "tasks"
)]
pub async fn list_tasks_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Query(query): Query<ListTasksQuery>,
) -> Result<Json<ListTasksResponse>, ServerError> {
    let tasks = state.tasks().read().await;

    let status_filter = query.status.as_ref().and_then(|s| parse_status(s));

    let mut summaries: Vec<TaskSummary> = tasks
        .values()
        .filter(|t| {
            // Filter by status if specified
            if let Some(ref status) = status_filter
                && &t.status != status
            {
                return false;
            }
            // Filter by session_id if specified
            if let Some(ref sid) = query.session_id
                && t.session_id.as_ref() != Some(sid)
            {
                return false;
            }
            true
        })
        .map(task_to_summary)
        .collect();

    // Sort by created_at descending (most recent first)
    summaries.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // Total is the count of all matching tasks (before limit)
    let total = summaries.len();

    // Apply limit after counting total
    summaries.truncate(query.limit);

    Ok(Json(ListTasksResponse {
        tasks: summaries,
        total,
    }))
}

/// GET /api/v1/tasks/:id - Get task details.
#[utoipa::path(
    get,
    path = "/api/v1/tasks/{id}",
    params(
        ("id" = String, Path, description = "Task ID"),
    ),
    responses(
        (status = 200, description = "Task details", body = TaskDetail),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Task not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "tasks"
)]
pub async fn get_task_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskDetail>, ServerError> {
    let tasks = state.tasks().read().await;

    let task = tasks
        .get(&task_id)
        .ok_or_else(|| ServerError::NotFound(format!("Task {} not found", task_id)))?;

    Ok(Json(task_to_detail(task)))
}

/// DELETE /api/v1/tasks/:id - Cancel a running task.
#[utoipa::path(
    delete,
    path = "/api/v1/tasks/{id}",
    params(
        ("id" = String, Path, description = "Task ID"),
    ),
    responses(
        (status = 204, description = "Task cancelled"),
        (status = 400, description = "Cannot cancel task in current state"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Task not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "tasks"
)]
pub async fn cancel_task_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Path(task_id): Path<String>,
) -> Result<StatusCode, ServerError> {
    let mut tasks = state.tasks().write().await;

    let task = tasks
        .get_mut(&task_id)
        .ok_or_else(|| ServerError::NotFound(format!("Task {} not found", task_id)))?;

    // Can only cancel pending or running tasks
    match task.status {
        TaskStatus::Pending | TaskStatus::Running => {
            task.cancel();
            Ok(StatusCode::NO_CONTENT)
        }
        _ => Err(ServerError::BadRequest(format!(
            "Cannot cancel task in {} state",
            serde_json::to_string(&task.status).unwrap_or_default()
        ))),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::auth_middleware;
    use crate::config::ServerConfig;
    use arawn_agent::{Agent, ToolRegistry};
    use arawn_llm::MockBackend;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
    };
    use tower::ServiceExt;

    fn create_test_state() -> AppState {
        let backend = MockBackend::with_text("Test");
        let agent = Agent::builder()
            .with_backend(backend)
            .with_tools(ToolRegistry::new())
            .build()
            .unwrap();

        AppState::new(agent, ServerConfig::new(Some("test-token".to_string())))
    }

    fn create_test_router(state: AppState) -> Router {
        Router::new()
            .route("/tasks", get(list_tasks_handler))
            .route(
                "/tasks/{id}",
                get(get_task_handler).delete(cancel_task_handler),
            )
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_list_tasks_empty() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/tasks")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: ListTasksResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.total, 0);
        assert!(result.tasks.is_empty());
    }

    #[tokio::test]
    async fn test_list_tasks_with_data() {
        let state = create_test_state();

        // Add some tasks
        {
            let mut tasks = state.tasks().write().await;
            tasks.insert("task-1".to_string(), TrackedTask::new("task-1", "indexing"));
            let mut running = TrackedTask::new("task-2", "processing");
            running.start();
            tasks.insert("task-2".to_string(), running);
        }

        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/tasks")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: ListTasksResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.total, 2);
    }

    #[tokio::test]
    async fn test_get_task() {
        let state = create_test_state();

        // Add a task
        {
            let mut tasks = state.tasks().write().await;
            tasks.insert("task-1".to_string(), TrackedTask::new("task-1", "indexing"));
        }

        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/tasks/task-1")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: TaskDetail = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.id, "task-1");
        assert_eq!(result.task_type, "indexing");
    }

    #[tokio::test]
    async fn test_get_task_not_found() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/tasks/nonexistent")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_cancel_task() {
        let state = create_test_state();

        // Add a running task
        {
            let mut tasks = state.tasks().write().await;
            let mut task = TrackedTask::new("task-1", "processing");
            task.start();
            tasks.insert("task-1".to_string(), task);
        }

        let app = create_test_router(state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/tasks/task-1")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // Verify cancelled
        let tasks = state.tasks().read().await;
        assert_eq!(tasks.get("task-1").unwrap().status, TaskStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_cancel_completed_task_fails() {
        let state = create_test_state();

        // Add a completed task
        {
            let mut tasks = state.tasks().write().await;
            let mut task = TrackedTask::new("task-1", "processing");
            task.complete(None);
            tasks.insert("task-1".to_string(), task);
        }

        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/tasks/task-1")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_list_tasks_filter_by_status() {
        let state = create_test_state();

        // Add tasks with different statuses
        {
            let mut tasks = state.tasks().write().await;
            tasks.insert(
                "task-1".to_string(),
                TrackedTask::new("task-1", "pending-task"),
            );
            let mut running = TrackedTask::new("task-2", "running-task");
            running.start();
            tasks.insert("task-2".to_string(), running);
        }

        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/tasks?status=running")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: ListTasksResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.tasks[0].task_type, "running-task");
    }
}
