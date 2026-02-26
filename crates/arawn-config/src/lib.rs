//! Configuration system for the Arawn agent framework.
//!
//! Provides TOML-based configuration with:
//! - Named LLM configurations (`[llm]`, `[llm.claude]`, `[llm.fast]`, etc.)
//! - Cascading agent-to-LLM binding with fallback defaults
//! - Config file layering (XDG user config + project-local overrides)
//! - API key resolution (keyring → env var → config file)
//!
//! Client configuration (kubeconfig-style):
//! - Named connection contexts (server + auth)
//! - `current-context` for default selection
//! - Multiple auth methods (API key, OAuth, bearer)
//!
//! See ADR ARAWN-A-0001 for architectural decisions.

pub mod client;
pub mod discovery;
pub mod error;
pub mod paths;
pub mod resolver;
pub mod secrets;
pub mod types;

pub use client::{
    AuthConfig, ClientConfig, ClientDefaults, Context, client_config_path, load_client_config,
    load_client_config_from, save_client_config, save_client_config_to,
};
pub use discovery::{
    LoadedConfig, load_config, load_config_file, load_config_with_options, save_config,
    xdg_config_dir, xdg_config_path,
};
pub use error::{ConfigError, Result};
pub use paths::{
    CleanupConfig as PathCleanupConfig, MonitoringConfig, PathConfig, UsageThresholds,
};
pub use resolver::{ResolvedFrom, ResolvedLlm, resolve_all_profiles, resolve_for_agent};
pub use types::*;
