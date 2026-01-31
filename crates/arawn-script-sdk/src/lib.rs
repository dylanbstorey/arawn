//! Arawn Script SDK — utilities for agent-generated Rust scripts.
//!
//! This crate is pre-compiled for `wasm32-wasip1` and linked into sandbox
//! scripts so they have access to JSON handling, text utilities, and a
//! standard execution harness without pulling in dependencies at compile time.
//!
//! # Usage
//!
//! Agent scripts implement a `run` function and use the [`entry`] macro to
//! wire up stdin/stdout I/O:
//!
//! ```rust,no_run
//! use arawn_script_sdk::prelude::*;
//!
//! fn run(ctx: Context) -> ScriptResult<Value> {
//!     let name = ctx.get_str("name").unwrap_or("world");
//!     Ok(json!({ "greeting": format!("Hello, {}!", name) }))
//! }
//!
//! arawn_script_sdk::entry!(run);
//! ```

pub mod context;
pub mod error;
pub mod text;

/// Re-exports for convenient `use arawn_script_sdk::prelude::*`.
pub mod prelude {
    pub use crate::context::Context;
    pub use crate::error::{ScriptError, ScriptResult};
    pub use crate::text;
    pub use serde_json::{json, Value};
}

pub use context::Context;
pub use error::{ScriptError, ScriptResult};

/// Entry-point macro that generates a `main()` function.
///
/// Reads JSON context from stdin, calls the provided function,
/// and writes the result as JSON to stdout. Errors are written
/// as structured JSON to stdout with a non-zero exit code.
///
/// # Example
///
/// ```rust,no_run
/// use arawn_script_sdk::prelude::*;
///
/// fn run(ctx: Context) -> ScriptResult<Value> {
///     Ok(json!({ "status": "ok" }))
/// }
///
/// arawn_script_sdk::entry!(run);
/// ```
#[macro_export]
macro_rules! entry {
    ($func:ident) => {
        fn main() {
            let result = $crate::run_harness($func);
            if let Err(code) = result {
                std::process::exit(code);
            }
        }
    };
}

/// Internal harness called by the `entry!` macro. Not intended for direct use.
///
/// Returns `Ok(())` on success, `Err(exit_code)` on failure.
pub fn run_harness(f: fn(Context) -> ScriptResult<serde_json::Value>) -> Result<(), i32> {
    use std::io::Read;

    // Read stdin (JSON context)
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        let err = serde_json::json!({
            "error": true,
            "message": format!("Failed to read stdin: {e}"),
        });
        print!("{}", err);
        return Err(1);
    }

    // Parse context
    let ctx = match Context::from_json(&input) {
        Ok(c) => c,
        Err(e) => {
            let err = serde_json::json!({
                "error": true,
                "message": format!("Failed to parse context: {e}"),
            });
            print!("{}", err);
            return Err(1);
        }
    };

    // Run user function
    match f(ctx) {
        Ok(value) => {
            print!("{}", value);
            Ok(())
        }
        Err(e) => {
            let err = serde_json::json!({
                "error": true,
                "message": e.to_string(),
            });
            print!("{}", err);
            Err(1)
        }
    }
}
