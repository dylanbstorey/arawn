//! Shared types for the Arawn agent system.

pub mod config;
pub mod delegation;
pub mod hooks;

pub use delegation::{
    DelegationOutcome, SharedSubagentSpawner, SubagentInfo, SubagentResult, SubagentSpawner,
};
pub use hooks::{
    HookAction, HookDef, HookDispatch, HookEvent, HookMatcherGroup, HookOutcome, HookType,
    HooksConfig, SharedHookDispatcher,
};

pub use config::{
    AgentConfigProvider, ConfigProvider, HasAgentConfig, HasRateLimitConfig, HasSessionConfig,
    HasToolConfig, SessionConfigProvider, ToolConfigProvider, defaults as config_defaults,
};
