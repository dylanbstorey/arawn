//! Task types for long-running operations.

use serde::{Deserialize, Serialize};

use crate::{Id, Timestamp, new_id, now};

/// A long-running task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Id,
    pub name: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub created_at: Timestamp,
    pub started_at: Option<Timestamp>,
    pub completed_at: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<TaskResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub progress: Vec<TaskProgress>,
}

impl Task {
    /// Create a new pending task.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: new_id(),
            name: name.into(),
            description: None,
            status: TaskStatus::Pending,
            created_at: now(),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
            progress: Vec::new(),
        }
    }

    /// Mark the task as running.
    pub fn start(&mut self) {
        self.status = TaskStatus::Running;
        self.started_at = Some(now());
    }

    /// Mark the task as completed with a result.
    pub fn complete(&mut self, result: TaskResult) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(now());
        self.result = Some(result);
    }

    /// Mark the task as failed with an error.
    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = TaskStatus::Failed;
        self.completed_at = Some(now());
        self.error = Some(error.into());
    }

    /// Add a progress update.
    pub fn add_progress(&mut self, message: impl Into<String>) {
        self.progress.push(TaskProgress {
            timestamp: now(),
            message: message.into(),
        });
    }
}

/// Status of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Progress update for a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    pub timestamp: Timestamp,
    pub message: String,
}

/// Result of a completed task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub artifacts: Vec<Artifact>,
}

/// An artifact produced by a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub artifact_type: ArtifactType,
    pub content: String,
}

/// Type of artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    Text,
    Code,
    Json,
    Markdown,
    File,
}
