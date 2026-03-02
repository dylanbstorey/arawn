//! Wasmtime sandbox for executing Rust scripts compiled to `wasm32-wasip1`.
//!
//! # Flow
//!
//! 1. Agent generates Rust source code
//! 2. `ScriptExecutor::compile()` invokes `rustc --target wasm32-wasip1`
//! 3. WASM binary is cached by SHA-256 of source content
//! 4. `ScriptExecutor::execute()` runs the module in Wasmtime with scoped WASI capabilities
//! 5. Context flows in via WASI stdin (JSON), output captured from WASI stdout (JSON)

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use tokio::sync::RwLock;
use tracing::{debug, warn};
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::p1::WasiP1Ctx;
use wasmtime_wasi::p2::pipe::{MemoryInputPipe, MemoryOutputPipe};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtxBuilder};

use crate::catalog::RuntimeCatalog;
use crate::definition::Capabilities;
use crate::error::PipelineError;
use crate::protocol::{RuntimeInput, RuntimeOutput};

/// Manages compilation and sandboxed execution of Rust scripts as WASM modules.
pub struct ScriptExecutor {
    /// Wasmtime engine (shared across all executions).
    engine: Engine,
    /// Directory for storing compiled `.wasm` files and temp sources.
    cache_dir: PathBuf,
    /// In-memory cache: SHA-256 hex → precompiled Module.
    module_cache: Arc<RwLock<HashMap<String, Module>>>,
    /// Default execution timeout.
    default_timeout: Duration,
}

/// Result of compiling a Rust source file to WASM.
#[derive(Debug)]
pub struct CompileResult {
    /// SHA-256 hex digest of the source content.
    pub source_hash: String,
    /// Path to the compiled `.wasm` file.
    pub wasm_path: PathBuf,
    /// Whether this was served from cache.
    pub cached: bool,
    /// Compilation time (zero if cached).
    pub compile_time: Duration,
}

/// Result of executing a WASM module.
#[derive(Debug)]
pub struct ScriptOutput {
    /// JSON output captured from stdout.
    pub stdout: String,
    /// Stderr output (diagnostics).
    pub stderr: String,
    /// Exit code (0 = success).
    pub exit_code: i32,
    /// Execution wall-clock time.
    pub elapsed: Duration,
}

/// Configuration for a single script execution.
#[derive(Debug, Clone)]
pub struct ScriptConfig {
    /// WASI capability grants.
    pub capabilities: Capabilities,
    /// Execution timeout override.
    pub timeout: Option<Duration>,
    /// Maximum memory in bytes (default: 64 MiB).
    pub max_memory_bytes: Option<usize>,
}

impl Default for ScriptConfig {
    fn default() -> Self {
        Self {
            capabilities: Capabilities {
                filesystem: vec![],
                network: false,
            },
            timeout: None,
            max_memory_bytes: Some(64 * 1024 * 1024),
        }
    }
}

impl ScriptExecutor {
    /// Create a new executor with the given cache directory and default timeout.
    pub fn new(cache_dir: PathBuf, default_timeout: Duration) -> Result<Self, PipelineError> {
        let mut config = Config::new();
        config.consume_fuel(true);

        let engine = Engine::new(&config).map_err(|e| {
            PipelineError::ScriptFailed(format!("Failed to create Wasmtime engine: {e}"))
        })?;

        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| PipelineError::ScriptFailed(format!("Failed to create cache dir: {e}")))?;

        Ok(Self {
            engine,
            cache_dir,
            module_cache: Arc::new(RwLock::new(HashMap::new())),
            default_timeout,
        })
    }

    /// Compile Rust source code to a WASM module targeting `wasm32-wasip1`.
    ///
    /// Returns a `CompileResult` with the hash and path. Uses a SHA-256 cache
    /// to skip recompilation for identical source content.
    pub async fn compile(&self, source: &str) -> Result<CompileResult, PipelineError> {
        let hash = sha256_hex(source);

        // Check disk cache
        let wasm_path = self.cache_dir.join(format!("{hash}.wasm"));
        if wasm_path.exists() {
            if !self.module_cache.read().await.contains_key(&hash) {
                let module = Module::from_file(&self.engine, &wasm_path).map_err(|e| {
                    PipelineError::ScriptFailed(format!("Failed to load cached WASM: {e}"))
                })?;
                self.module_cache.write().await.insert(hash.clone(), module);
            }
            debug!(hash = %hash, "WASM cache hit");
            return Ok(CompileResult {
                source_hash: hash,
                wasm_path,
                cached: true,
                compile_time: Duration::ZERO,
            });
        }

        // Check wasm32-wasip1 target is available
        Self::check_wasm_target().await?;

        // Write source to temp file
        let src_path = self.cache_dir.join(format!("{hash}.rs"));
        tokio::fs::write(&src_path, source).await.map_err(|e| {
            PipelineError::CompilationFailed(format!("Failed to write source: {e}"))
        })?;

        // Compile with rustc
        let start = Instant::now();
        let output = tokio::process::Command::new("rustc")
            .arg("--target")
            .arg("wasm32-wasip1")
            .arg("--edition")
            .arg("2021")
            .arg("-O")
            .arg("-o")
            .arg(&wasm_path)
            .arg(&src_path)
            .output()
            .await
            .map_err(|e| {
                PipelineError::CompilationFailed(format!("Failed to invoke rustc: {e}"))
            })?;
        let compile_time = start.elapsed();

        if !output.status.success() {
            // Keep source file for debugging on failure
            let _ = tokio::fs::remove_file(&src_path).await;
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(PipelineError::CompilationFailed(stderr));
        }

        // Load module into memory cache
        let module = Module::from_file(&self.engine, &wasm_path).map_err(|e| {
            PipelineError::ScriptFailed(format!("Failed to load compiled WASM: {e}"))
        })?;
        self.module_cache.write().await.insert(hash.clone(), module);

        debug!(hash = %hash, elapsed = ?compile_time, "Compiled Rust to WASM");

        Ok(CompileResult {
            source_hash: hash,
            wasm_path,
            cached: false,
            compile_time,
        })
    }

    /// Compile an entire Cargo crate to `wasm32-wasip1` and return the `.wasm` path.
    ///
    /// Runs `cargo build --target wasm32-wasip1 --release` on the given crate directory.
    /// The compiled `.wasm` is left in the crate's `target/` directory.
    pub async fn compile_crate(&self, crate_dir: &Path) -> Result<PathBuf, PipelineError> {
        let manifest = crate_dir.join("Cargo.toml");
        if !manifest.exists() {
            return Err(PipelineError::CompilationFailed(format!(
                "No Cargo.toml found in {}",
                crate_dir.display()
            )));
        }

        Self::check_wasm_target().await?;

        let start = Instant::now();
        let output = tokio::process::Command::new("cargo")
            .arg("build")
            .arg("--target")
            .arg("wasm32-wasip1")
            .arg("--release")
            .arg("--manifest-path")
            .arg(&manifest)
            .output()
            .await
            .map_err(|e| {
                PipelineError::CompilationFailed(format!("Failed to invoke cargo: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PipelineError::CompilationFailed(format!(
                "cargo build failed for {}: {}",
                crate_dir.display(),
                stderr
            )));
        }

        let compile_time = start.elapsed();

        // Find the .wasm file in target/wasm32-wasip1/release/
        let release_dir = crate_dir.join("target/wasm32-wasip1/release");
        let wasm_path = std::fs::read_dir(&release_dir)
            .map_err(|e| {
                PipelineError::CompilationFailed(format!(
                    "Cannot read release dir {}: {e}",
                    release_dir.display()
                ))
            })?
            .filter_map(|entry| entry.ok())
            .map(|e| e.path())
            .find(|p| p.extension().is_some_and(|ext| ext == "wasm"))
            .ok_or_else(|| {
                PipelineError::CompilationFailed(format!(
                    "No .wasm file found in {}",
                    release_dir.display()
                ))
            })?;

        debug!(
            crate_dir = %crate_dir.display(),
            wasm = %wasm_path.display(),
            elapsed = ?compile_time,
            "Compiled crate to WASM"
        );

        Ok(wasm_path)
    }

    /// Execute a previously compiled WASM module with the given context and capabilities.
    ///
    /// Context is passed as JSON via stdin. Output is captured from stdout.
    pub async fn execute(
        &self,
        source_hash: &str,
        context_json: &str,
        config: &ScriptConfig,
    ) -> Result<ScriptOutput, PipelineError> {
        let module = {
            let cache = self.module_cache.read().await;
            cache.get(source_hash).cloned().ok_or_else(|| {
                PipelineError::ScriptFailed(format!(
                    "No compiled module found for hash {source_hash}. Call compile() first."
                ))
            })?
        };

        let timeout = config.timeout.unwrap_or(self.default_timeout);
        let engine = self.engine.clone();
        let context_json = context_json.to_string();
        let capabilities = config.capabilities.clone();

        // Run in blocking task since Wasmtime execution is synchronous
        tokio::task::spawn_blocking(move || {
            Self::execute_sync(&engine, &module, &context_json, &capabilities, timeout)
        })
        .await
        .map_err(|e| PipelineError::ScriptFailed(format!("Task join error: {e}")))?
    }

    /// Compile and execute in one call.
    pub async fn compile_and_execute(
        &self,
        source: &str,
        context_json: &str,
        config: &ScriptConfig,
    ) -> Result<(CompileResult, ScriptOutput), PipelineError> {
        let compile_result = self.compile(source).await?;
        let output = self
            .execute(&compile_result.source_hash, context_json, config)
            .await?;
        Ok((compile_result, output))
    }

    /// Clear the in-memory module cache.
    pub async fn clear_cache(&self) {
        self.module_cache.write().await.clear();
    }

    /// Execute a named runtime from the catalog with the given input.
    ///
    /// Looks up the runtime in the catalog, loads and caches the `.wasm` module,
    /// serializes `RuntimeInput` to stdin, and parses stdout as `RuntimeOutput`.
    pub async fn execute_runtime(
        &self,
        name: &str,
        input: &RuntimeInput,
        catalog: &RuntimeCatalog,
    ) -> Result<RuntimeOutput, PipelineError> {
        // Resolve runtime to .wasm path
        let wasm_path = catalog.resolve_path(name).ok_or_else(|| {
            PipelineError::ScriptFailed(format!("Unknown runtime '{name}' — not found in catalog"))
        })?;

        if !wasm_path.exists() {
            return Err(PipelineError::ScriptFailed(format!(
                "Runtime '{name}' .wasm file not found at {}",
                wasm_path.display()
            )));
        }

        // Load module (cache by runtime name)
        let cache_key = format!("runtime:{name}");
        let module = {
            let cache = self.module_cache.read().await;
            cache.get(&cache_key).cloned()
        };

        let module = match module {
            Some(m) => m,
            None => {
                let m = Module::from_file(&self.engine, &wasm_path).map_err(|e| {
                    PipelineError::ScriptFailed(format!(
                        "Failed to load runtime '{name}' from {}: {e}",
                        wasm_path.display()
                    ))
                })?;
                self.module_cache.write().await.insert(cache_key, m.clone());
                m
            }
        };

        // Serialize input
        let input_json = serde_json::to_string(input).map_err(|e| {
            PipelineError::ScriptFailed(format!("Failed to serialize RuntimeInput: {e}"))
        })?;

        let timeout = self.default_timeout;
        let engine = self.engine.clone();

        // Execute
        let script_output = tokio::task::spawn_blocking(move || {
            let caps = Capabilities {
                filesystem: vec![],
                network: false,
            };
            Self::execute_sync(&engine, &module, &input_json, &caps, timeout)
        })
        .await
        .map_err(|e| PipelineError::ScriptFailed(format!("Task join error: {e}")))??;

        if script_output.exit_code != 0 {
            return Err(PipelineError::ScriptFailed(format!(
                "Runtime '{name}' exited with code {}. stderr: {}",
                script_output.exit_code, script_output.stderr
            )));
        }

        // Parse stdout as RuntimeOutput
        let output: RuntimeOutput = serde_json::from_str(&script_output.stdout).map_err(|e| {
            PipelineError::ScriptFailed(format!(
                "Runtime '{name}' produced invalid output JSON: {e}. stdout: {}",
                script_output.stdout
            ))
        })?;

        Ok(output)
    }

    /// Check if the `wasm32-wasip1` target is installed.
    async fn check_wasm_target() -> Result<(), PipelineError> {
        let output = tokio::process::Command::new("rustup")
            .args(["target", "list", "--installed"])
            .output()
            .await
            .map_err(|e| {
                PipelineError::CompilationFailed(format!("Failed to invoke rustup: {e}"))
            })?;

        let installed = String::from_utf8_lossy(&output.stdout);
        if !installed.contains("wasm32-wasip1") {
            return Err(PipelineError::CompilationFailed(
                "The wasm32-wasip1 target is not installed. \
                 Install it with: rustup target add wasm32-wasip1"
                    .to_string(),
            ));
        }
        Ok(())
    }

    /// Synchronous WASM execution with Wasmtime + WASI Preview 1.
    fn execute_sync(
        engine: &Engine,
        module: &Module,
        context_json: &str,
        capabilities: &Capabilities,
        timeout: Duration,
    ) -> Result<ScriptOutput, PipelineError> {
        let start = Instant::now();

        // Build WASI context using wasmtime-wasi's builder + memory pipes
        let stdin_pipe = MemoryInputPipe::new(context_json.as_bytes().to_vec());
        let stdout_pipe = MemoryOutputPipe::new(1024 * 1024); // 1 MiB stdout buffer
        let stderr_pipe = MemoryOutputPipe::new(256 * 1024); // 256 KiB stderr buffer

        let mut wasi_builder = WasiCtxBuilder::new();
        wasi_builder
            .stdin(stdin_pipe)
            .stdout(stdout_pipe.clone())
            .stderr(stderr_pipe.clone());

        // Grant filesystem capabilities
        for dir_path in &capabilities.filesystem {
            let path = Path::new(dir_path);
            if path.is_dir()
                && let Err(e) =
                    wasi_builder.preopened_dir(path, dir_path, DirPerms::all(), FilePerms::all())
            {
                warn!(path = %dir_path, error = %e, "Could not open preopened directory, skipping");
            }
        }

        // Build WasiP1Ctx (WASI Preview 1 context for wasm32-wasip1 modules)
        let wasi_ctx = wasi_builder.build_p1();

        let mut store = Store::new(engine, wasi_ctx);

        // Set fuel limit based on timeout (roughly 33M instructions/sec)
        let fuel_per_sec = 33_000_000u64;
        let fuel = fuel_per_sec * timeout.as_secs().max(1);
        store
            .set_fuel(fuel)
            .map_err(|e| PipelineError::ScriptFailed(format!("Failed to set fuel: {e}")))?;

        // Link WASI Preview 1 imports
        let mut linker = Linker::new(engine);
        wasmtime_wasi::p1::add_to_linker_sync(&mut linker, |ctx: &mut WasiP1Ctx| ctx)
            .map_err(|e| PipelineError::ScriptFailed(format!("Failed to link WASI: {e}")))?;

        // Instantiate and run _start (WASI entry point)
        let instance = linker.instantiate(&mut store, module).map_err(|e| {
            PipelineError::ScriptFailed(format!("Failed to instantiate module: {e}"))
        })?;

        let start_fn = instance
            .get_typed_func::<(), ()>(&mut store, "_start")
            .map_err(|e| PipelineError::ScriptFailed(format!("No _start entry point: {e}")))?;

        let exit_code = match start_fn.call(&mut store, ()) {
            Ok(()) => 0,
            Err(e) => {
                // Check for WASI proc_exit
                if let Some(exit) = e.downcast_ref::<wasmtime_wasi::I32Exit>() {
                    exit.0
                } else {
                    // Check for fuel exhaustion (timeout)
                    let msg = e.to_string();
                    if msg.contains("fuel") {
                        return Err(PipelineError::ScriptFailed(
                            "Script execution timed out (fuel exhausted)".to_string(),
                        ));
                    }
                    return Err(PipelineError::ScriptFailed(format!(
                        "WASM execution error: {e}"
                    )));
                }
            }
        };

        let elapsed = start.elapsed();

        let stdout = String::from_utf8(stdout_pipe.contents().to_vec())
            .unwrap_or_else(|_| String::from("<non-utf8 output>"));
        let stderr = String::from_utf8(stderr_pipe.contents().to_vec())
            .unwrap_or_else(|_| String::from("<non-utf8 stderr>"));

        Ok(ScriptOutput {
            stdout,
            stderr,
            exit_code,
            elapsed,
        })
    }
}

/// Compute SHA-256 hex digest of a string.
fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_executor() -> (ScriptExecutor, TempDir) {
        let tmp = TempDir::new().unwrap();
        let executor =
            ScriptExecutor::new(tmp.path().join("cache"), Duration::from_secs(30)).unwrap();
        (executor, tmp)
    }

    #[test]
    fn test_sha256_deterministic() {
        let hash1 = sha256_hex("hello world");
        let hash2 = sha256_hex("hello world");
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_sha256_different_inputs() {
        let hash1 = sha256_hex("hello");
        let hash2 = sha256_hex("world");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_executor_creation() {
        let (executor, _tmp) = test_executor();
        assert!(executor.cache_dir.exists());
    }

    #[test]
    fn test_default_script_config() {
        let config = ScriptConfig::default();
        assert!(config.capabilities.filesystem.is_empty());
        assert!(!config.capabilities.network);
        assert_eq!(config.max_memory_bytes, Some(64 * 1024 * 1024));
        assert!(config.timeout.is_none());
    }

    #[tokio::test]
    async fn test_compile_simple_rust() {
        let (executor, _tmp) = test_executor();
        let source = r#"fn main() { println!("hello"); }"#;
        let result = executor.compile(source).await;

        match result {
            Ok(r) => {
                assert!(!r.cached);
                assert!(r.wasm_path.exists());
                assert!(!r.source_hash.is_empty());
            }
            Err(PipelineError::CompilationFailed(msg)) if msg.contains("wasm32-wasip1") => {
                eprintln!("Skipping: wasm32-wasip1 target not installed");
            }
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }

    #[tokio::test]
    async fn test_compile_cache_hit() {
        let (executor, _tmp) = test_executor();
        let source = r#"fn main() { println!("cached"); }"#;

        let first = executor.compile(source).await;
        match first {
            Ok(r1) => {
                assert!(!r1.cached);
                let r2 = executor.compile(source).await.unwrap();
                assert!(r2.cached);
                assert_eq!(r1.source_hash, r2.source_hash);
                assert_eq!(r2.compile_time, Duration::ZERO);
            }
            Err(PipelineError::CompilationFailed(msg)) if msg.contains("wasm32-wasip1") => {
                eprintln!("Skipping: wasm32-wasip1 target not installed");
            }
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }

    #[tokio::test]
    async fn test_compile_error_returned() {
        let (executor, _tmp) = test_executor();
        let bad_source = r#"fn main() { let x: i32 = "not an int"; }"#;
        let result = executor.compile(bad_source).await;

        match result {
            Err(PipelineError::CompilationFailed(msg)) => {
                if !msg.contains("wasm32-wasip1") {
                    assert!(
                        msg.contains("mismatched types") || msg.contains("expected"),
                        "Expected type error, got: {msg}"
                    );
                }
            }
            Ok(_) => panic!("Expected compilation failure"),
            Err(e) => panic!("Unexpected error type: {e}"),
        }
    }

    #[tokio::test]
    async fn test_execute_simple_script() {
        let (executor, _tmp) = test_executor();
        let source = r#"fn main() { print!("{}", "{\"result\":\"hello\"}"); }"#;

        let compile = executor.compile(source).await;
        match compile {
            Ok(cr) => {
                let config = ScriptConfig::default();
                let output = executor
                    .execute(&cr.source_hash, "{}", &config)
                    .await
                    .unwrap();
                assert_eq!(output.exit_code, 0);
                assert!(output.stdout.contains("result"));
            }
            Err(PipelineError::CompilationFailed(msg)) if msg.contains("wasm32-wasip1") => {
                eprintln!("Skipping: wasm32-wasip1 target not installed");
            }
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }

    #[tokio::test]
    async fn test_execute_reads_stdin_context() {
        let (executor, _tmp) = test_executor();
        let source = r#"
use std::io::Read;
fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    print!("got:{}", input);
}
"#;
        let compile = executor.compile(source).await;
        match compile {
            Ok(cr) => {
                let config = ScriptConfig::default();
                let output = executor
                    .execute(&cr.source_hash, r#"{"key":"value"}"#, &config)
                    .await
                    .unwrap();
                assert_eq!(output.exit_code, 0);
                assert!(
                    output.stdout.contains(r#"got:{"key":"value"}"#),
                    "stdout was: {}",
                    output.stdout
                );
            }
            Err(PipelineError::CompilationFailed(msg)) if msg.contains("wasm32-wasip1") => {
                eprintln!("Skipping: wasm32-wasip1 target not installed");
            }
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }

    #[tokio::test]
    async fn test_execute_nonexistent_hash() {
        let (executor, _tmp) = test_executor();
        let config = ScriptConfig::default();
        let result = executor.execute("nonexistent_hash", "{}", &config).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No compiled module")
        );
    }

    #[tokio::test]
    async fn test_execute_exit_code() {
        let (executor, _tmp) = test_executor();
        let source = r#"fn main() { std::process::exit(42); }"#;

        let compile = executor.compile(source).await;
        match compile {
            Ok(cr) => {
                let config = ScriptConfig::default();
                let output = executor
                    .execute(&cr.source_hash, "{}", &config)
                    .await
                    .unwrap();
                assert_eq!(output.exit_code, 42);
            }
            Err(PipelineError::CompilationFailed(msg)) if msg.contains("wasm32-wasip1") => {
                eprintln!("Skipping: wasm32-wasip1 target not installed");
            }
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }

    #[tokio::test]
    async fn test_execute_runtime_unknown_name() {
        let (executor, tmp) = test_executor();
        let catalog = RuntimeCatalog::load(&tmp.path().join("runtimes")).unwrap();
        let input = RuntimeInput {
            config: serde_json::json!({}),
            context: serde_json::json!({}),
        };
        let result = executor
            .execute_runtime("nonexistent", &input, &catalog)
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown runtime"));
    }

    #[tokio::test]
    async fn test_execute_runtime_missing_wasm() {
        let (executor, tmp) = test_executor();
        let mut catalog = RuntimeCatalog::load(&tmp.path().join("runtimes")).unwrap();
        catalog
            .add(
                "ghost",
                crate::catalog::CatalogEntry {
                    description: "Missing file".into(),
                    path: "builtin/ghost.wasm".into(),
                    category: crate::catalog::RuntimeCategory::Builtin,
                },
            )
            .unwrap();
        let input = RuntimeInput {
            config: serde_json::json!({}),
            context: serde_json::json!({}),
        };
        let result = executor.execute_runtime("ghost", &input, &catalog).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found at"));
    }

    #[tokio::test]
    async fn test_execute_runtime_passthrough() {
        // Compile a minimal passthrough runtime inline
        let (executor, tmp) = test_executor();

        let source = r#"
use std::io::Read;
fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    // Parse as JSON, extract config and context, echo back as RuntimeOutput
    let v: serde_json::Value = serde_json::from_str(&input).unwrap();
    let out = serde_json::json!({
        "status": "ok",
        "output": {
            "config": v["config"],
            "context": v["context"],
        }
    });
    print!("{}", out);
}
"#;
        // Need serde_json — compile with rustc won't have it.
        // Instead, do a raw string echo approach:
        let simple_source = r#"
use std::io::Read;
fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    // Just wrap the input in a RuntimeOutput envelope
    print!("{{\"status\":\"ok\",\"output\":{}}}", input);
}
"#;
        let compile = executor.compile(simple_source).await;
        let cr = match compile {
            Ok(cr) => cr,
            Err(PipelineError::CompilationFailed(msg)) if msg.contains("wasm32-wasip1") => {
                eprintln!("Skipping: wasm32-wasip1 target not installed");
                return;
            }
            Err(e) => panic!("Unexpected error: {e}"),
        };

        // Copy the compiled wasm into a catalog
        let runtimes_dir = tmp.path().join("runtimes");
        let builtin_dir = runtimes_dir.join("builtin");
        std::fs::create_dir_all(&builtin_dir).unwrap();
        let dest = builtin_dir.join("passthrough.wasm");
        std::fs::copy(&cr.wasm_path, &dest).unwrap();

        let mut catalog = RuntimeCatalog::load(&runtimes_dir).unwrap();
        catalog
            .add(
                "passthrough",
                crate::catalog::CatalogEntry {
                    description: "Test passthrough".into(),
                    path: "builtin/passthrough.wasm".into(),
                    category: crate::catalog::RuntimeCategory::Builtin,
                },
            )
            .unwrap();

        let input = RuntimeInput {
            config: serde_json::json!({"key": "val"}),
            context: serde_json::json!({"prev": 42}),
        };
        let output = executor
            .execute_runtime("passthrough", &input, &catalog)
            .await
            .unwrap();
        assert!(output.is_ok());
        let out_val = output.output.unwrap();
        assert_eq!(out_val["config"]["key"], "val");
        assert_eq!(out_val["context"]["prev"], 42);
    }

    #[tokio::test]
    async fn test_execute_runtime_caches_module() {
        let (executor, tmp) = test_executor();
        let simple_source = r#"fn main() { print!("{{\"status\":\"ok\"}}"); }"#;
        let compile = executor.compile(simple_source).await;
        let cr = match compile {
            Ok(cr) => cr,
            Err(PipelineError::CompilationFailed(msg)) if msg.contains("wasm32-wasip1") => {
                eprintln!("Skipping: wasm32-wasip1 target not installed");
                return;
            }
            Err(e) => panic!("Unexpected error: {e}"),
        };

        let runtimes_dir = tmp.path().join("runtimes");
        let builtin_dir = runtimes_dir.join("builtin");
        std::fs::create_dir_all(&builtin_dir).unwrap();
        std::fs::copy(&cr.wasm_path, builtin_dir.join("test.wasm")).unwrap();

        let mut catalog = RuntimeCatalog::load(&runtimes_dir).unwrap();
        catalog
            .add(
                "test_rt",
                crate::catalog::CatalogEntry {
                    description: "".into(),
                    path: "builtin/test.wasm".into(),
                    category: crate::catalog::RuntimeCategory::Builtin,
                },
            )
            .unwrap();

        let input = RuntimeInput {
            config: serde_json::json!({}),
            context: serde_json::json!({}),
        };

        // First call loads module
        let _out1 = executor
            .execute_runtime("test_rt", &input, &catalog)
            .await
            .unwrap();
        // Verify cached
        assert!(
            executor
                .module_cache
                .read()
                .await
                .contains_key("runtime:test_rt")
        );
        // Second call uses cache
        let _out2 = executor
            .execute_runtime("test_rt", &input, &catalog)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let (executor, _tmp) = test_executor();
        let source = r#"fn main() {}"#;

        let compile = executor.compile(source).await;
        match compile {
            Ok(cr) => {
                // Module should be in cache
                assert!(
                    executor
                        .module_cache
                        .read()
                        .await
                        .contains_key(&cr.source_hash)
                );
                executor.clear_cache().await;
                assert!(executor.module_cache.read().await.is_empty());
                // But disk cache still exists, so re-compile should hit disk
                let r2 = executor.compile(source).await.unwrap();
                assert!(r2.cached);
            }
            Err(PipelineError::CompilationFailed(msg)) if msg.contains("wasm32-wasip1") => {
                eprintln!("Skipping: wasm32-wasip1 target not installed");
            }
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }
}
