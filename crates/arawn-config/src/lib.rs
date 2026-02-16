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
pub mod resolver;
pub mod secrets;
pub mod types;

pub use client::{
    client_config_path, load_client_config, load_client_config_from, save_client_config,
    save_client_config_to, AuthConfig, ClientConfig, ClientDefaults, Context,
};
pub use discovery::{
    load_config, load_config_file, load_config_with_options, save_config, xdg_config_dir,
    xdg_config_path, LoadedConfig,
};
pub use error::{ConfigError, Result};
pub use resolver::{resolve_all_profiles, resolve_for_agent, ResolvedFrom, ResolvedLlm};
pub use types::*;
