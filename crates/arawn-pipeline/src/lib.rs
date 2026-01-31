//! Workflow orchestration engine for Arawn, powered by Cloacina.
//!
//! This crate provides the `PipelineEngine` — Arawn's execution backbone for
//! resilient, async, and scheduled workflows. It wraps Cloacina's `DefaultRunner`
//! with Arawn-specific configuration and a dynamic workflow construction API.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │  PipelineEngine                                         │
//! │  - Wraps Cloacina DefaultRunner (SQLite backend)        │
//! │  - Dynamic workflow construction (no macros)            │
//! │  - Cron scheduling + push triggers                      │
//! │  - Agent-facing API for workflow CRUD                   │
//! └─────────────────────────────────────────────────────────┘
//! ```

pub mod catalog;
pub mod context;
pub mod definition;
pub mod engine;
pub mod error;
pub mod factory;
pub mod loader;
pub mod protocol;
pub mod sandbox;
pub mod task;

pub use catalog::{CatalogEntry, RuntimeCatalog, RuntimeCategory};
pub use context::{ContextResolver, resolve_params, resolve_template_string};
pub use definition::{
    ActionDefinition, ActionExecutorFactory, Capabilities, RuntimeConfig, ScheduleConfig,
    TaskDefinition, TriggerConfig, WorkflowDefinition, WorkflowFile,
};
pub use engine::{ExecutionResult, ExecutionStatus, PipelineConfig, PipelineEngine, ScheduleInfo};
pub use error::{PipelineError, Result};
pub use factory::build_executor_factory;
pub use loader::{WatcherHandle, WorkflowEvent, WorkflowLoader};
pub use protocol::{RuntimeInput, RuntimeOutput};
pub use sandbox::{CompileResult, ScriptConfig, ScriptExecutor, ScriptOutput};
pub use task::DynamicTask;
