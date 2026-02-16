//! Shared types for the Arawn agent system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod config;
pub mod delegation;
pub mod error;
pub mod hooks;
pub mod memory;
pub mod message;
pub mod task;

pub use delegation::{
    DelegationOutcome, SharedSubagentSpawner, SubagentInfo, SubagentResult, SubagentSpawner,
};
pub use error::{Error, Result};
pub use hooks::{
    HookAction, HookDef, HookDispatch, HookEvent, HookMatcherGroup, HookOutcome, HookType,
    HooksConfig, SharedHookDispatcher,
};

pub use config::{
    defaults as config_defaults, AgentConfigProvider, ConfigProvider, HasAgentConfig,
    HasRateLimitConfig, HasSessionConfig, HasToolConfig, SessionConfigProvider, ToolConfigProvider,
};

/// Unique identifier for sessions, tasks, memories, etc.
pub type Id = Uuid;

/// Generate a new unique identifier.
pub fn new_id() -> Id {
    Uuid::new_v4()
}

/// Timestamp type used throughout the system.
pub type Timestamp = DateTime<Utc>;

/// Get the current timestamp.
pub fn now() -> Timestamp {
    Utc::now()
}

/// Configuration for the Arawn system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server bind address.
    pub bind_address: String,
    /// Server port.
    pub port: u16,
    /// Path to the SQLite database.
    pub db_path: String,
    /// Default LLM provider.
    pub default_provider: String,
    /// Enable memory context in conversations.
    pub memory_enabled: bool,
    /// Maximum memories to include in context.
    pub memory_limit: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".into(),
            port: 8420,
            db_path: "~/.arawn/arawn.db".into(),
            default_provider: "anthropic".into(),
            memory_enabled: true,
            memory_limit: 10,
        }
    }
}
