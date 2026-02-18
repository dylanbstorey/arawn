//! OS-level sandboxing for shell command execution.
//!
//! This crate provides a high-level interface to the `sandbox-runtime` crate,
//! making shell command sandboxing easy and safe. Sandboxing is **required** -
//! commands cannot be executed without a working sandbox.
//!
//! # Security Model
//!
//! - **Write access**: Deny by default, allow only explicit paths
//! - **Read access**: Allow by default, deny sensitive paths
//! - **Network**: Controlled via proxy-based domain filtering
//!
//! # Platform Support
//!
//! | Platform | Backend | Requirements |
//! |----------|---------|--------------|
//! | macOS | sandbox-exec | Built-in (no extra deps) |
//! | Linux | bubblewrap + socat | `apt install bubblewrap socat` |
//!
//! # Example
//!
//! ```no_run
//! use arawn_sandbox::{SandboxManager, SandboxConfig};
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Check if sandbox is available
//!     let status = SandboxManager::check_availability();
//!     if !status.is_available() {
//!         eprintln!("Sandbox unavailable: {}", status);
//!         return Ok(());
//!     }
//!
//!     // Create sandbox manager
//!     let manager = SandboxManager::new().await?;
//!
//!     // Configure allowed paths
//!     let config = SandboxConfig::default()
//!         .with_write_paths(vec![PathBuf::from("/tmp/work")]);
//!
//!     // Execute sandboxed command
//!     let output = manager.execute("echo hello", &config).await?;
//!     println!("Output: {}", output.stdout);
//!
//!     Ok(())
//! }
//! ```

mod config;
mod error;
mod manager;
mod platform;

pub use config::SandboxConfig;
pub use error::{SandboxError, SandboxResult};
pub use manager::SandboxManager;
pub use platform::{Platform, SandboxStatus};
