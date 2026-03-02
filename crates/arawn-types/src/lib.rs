//! Shared types for the Arawn agent system.

pub mod config;
pub mod delegation;
pub mod fs_gate;
pub mod hooks;
pub mod secret_resolver;

pub use delegation::{
    DelegationOutcome, SharedSubagentSpawner, SubagentInfo, SubagentResult, SubagentSpawner,
};
pub use fs_gate::{
    FsGate, FsGateError, FsGateResolver, GATED_TOOLS, SandboxOutput, SharedFsGate, is_gated_tool,
};
pub use hooks::{
    HookAction, HookDef, HookDispatch, HookEvent, HookMatcherGroup, HookOutcome, HookType,
    HooksConfig, SharedHookDispatcher,
};
pub use secret_resolver::{
    SecretResolver, SharedSecretResolver, contains_secret_handle, extract_secret_name,
    resolve_handles_in_json, resolve_handles_in_string,
};

pub use config::{
    AgentConfigProvider, ConfigProvider, HasAgentConfig, HasRateLimitConfig, HasSessionConfig,
    HasToolConfig, SessionConfigProvider, ToolConfigProvider, defaults as config_defaults,
};
