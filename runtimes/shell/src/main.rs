//! Shell runtime — executes a command with arguments and returns output.
//!
//! Config fields:
//! - `command` (string, required): Command to execute
//! - `args` (array of strings, optional): Command arguments
//! - `stdin` (string, optional): Input to pass to the command's stdin
//!
//! Returns stdout, stderr, and exit code.
//!
//! Note: Under WASI preview 1, process spawning is not natively supported.
//! This runtime uses `std::process::Command` which requires the host to
//! provide command execution capability via WASI extensions or a compatible
//! runtime like Wasmtime with command support.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Read;

#[derive(Deserialize)]
struct RuntimeInput {
    #[serde(default)]
    config: Value,
    #[serde(default)]
    context: Value,
}

#[derive(Serialize)]
struct RuntimeOutput {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn emit_error(msg: &str) {
    let out = RuntimeOutput {
        status: "error".into(),
        output: None,
        error: Some(msg.to_string()),
    };
    match serde_json::to_string(&out) {
        Ok(json) => print!("{json}"),
        Err(_) => print!(r#"{{"status":"error","error":"serialization failed"}}"#),
    }
    std::process::exit(1);
}

fn emit_output(out: &RuntimeOutput) {
    match serde_json::to_string(out) {
        Ok(json) => print!("{json}"),
        Err(e) => emit_error(&format!("Failed to serialize output: {e}")),
    }
}

/// Core processing logic, separated for testability.
fn process(input: &str) -> RuntimeOutput {
    let ri: RuntimeInput = match serde_json::from_str(input) {
        Ok(ri) => ri,
        Err(e) => {
            return RuntimeOutput {
                status: "error".into(),
                output: None,
                error: Some(format!("Invalid input JSON: {e}")),
            };
        }
    };

    let command = match ri.config.get("command").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => {
            return RuntimeOutput {
                status: "error".into(),
                output: None,
                error: Some("Missing required config field 'command'".into()),
            };
        }
    };

    let args: Vec<String> = ri
        .config
        .get("args")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let stdin_input = ri.config.get("stdin").and_then(|v| v.as_str()).map(String::from);

    let mut cmd = std::process::Command::new(&command);
    cmd.args(&args);

    if stdin_input.is_some() {
        cmd.stdin(std::process::Stdio::piped());
    }
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return RuntimeOutput {
                status: "error".into(),
                output: None,
                error: Some(format!("Failed to spawn command '{}': {}", command, e)),
            };
        }
    };

    if let Some(ref input_data) = stdin_input {
        if let Some(mut child_stdin) = child.stdin.take() {
            use std::io::Write;
            if let Err(e) = child_stdin.write_all(input_data.as_bytes()) {
                return RuntimeOutput {
                    status: "error".into(),
                    output: None,
                    error: Some(format!("Failed to write to command stdin: {e}")),
                };
            }
        }
    }

    match child.wait_with_output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            let exit_code = output.status.code().unwrap_or(-1);

            RuntimeOutput {
                status: if output.status.success() { "ok" } else { "error" }.into(),
                output: Some(serde_json::json!({
                    "command": command,
                    "args": args,
                    "stdout": stdout,
                    "stderr": stderr,
                    "exit_code": exit_code,
                })),
                error: if output.status.success() {
                    None
                } else {
                    Some(format!(
                        "Command '{}' exited with code {}{}",
                        command,
                        exit_code,
                        if stderr.is_empty() { String::new() } else { format!(": {stderr}") }
                    ))
                },
            }
        }
        Err(e) => RuntimeOutput {
            status: "error".into(),
            output: None,
            error: Some(format!("Failed to wait for command '{}': {e}", command)),
        },
    }
}

fn main() {
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        emit_error(&format!("Failed to read stdin: {e}"));
        return;
    }

    let result = process(&input);
    if result.status == "error" {
        // Still emit the output structure (it may contain stdout/stderr) then exit
        emit_output(&result);
        std::process::exit(1);
    } else {
        emit_output(&result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_shell_echo() {
        let input = json!({
            "config": {"command": "echo", "args": ["hello"]},
            "context": {}
        }).to_string();
        let result = process(&input);
        assert_eq!(result.status, "ok");
        let out = result.output.unwrap();
        assert_eq!(out["stdout"].as_str().unwrap().trim(), "hello");
        assert_eq!(out["exit_code"], 0);
    }

    #[test]
    fn test_shell_failed_command() {
        let input = json!({
            "config": {"command": "false"},
            "context": {}
        }).to_string();
        let result = process(&input);
        assert_eq!(result.status, "error");
        assert!(result.error.is_some());
        let out = result.output.unwrap();
        assert_ne!(out["exit_code"], 0);
    }

    #[test]
    fn test_shell_nonexistent_command() {
        let input = json!({
            "config": {"command": "nonexistent_command_xyz_123"},
            "context": {}
        }).to_string();
        let result = process(&input);
        assert_eq!(result.status, "error");
        assert!(result.error.unwrap().contains("Failed to spawn command"));
    }

    #[test]
    fn test_shell_missing_command() {
        let input = json!({"config": {}, "context": {}}).to_string();
        let result = process(&input);
        assert_eq!(result.status, "error");
        assert!(result.error.unwrap().contains("Missing required config field 'command'"));
    }

    #[test]
    fn test_shell_stdin_pipe() {
        let input = json!({
            "config": {"command": "cat", "stdin": "piped data"},
            "context": {}
        }).to_string();
        let result = process(&input);
        assert_eq!(result.status, "ok");
        assert_eq!(result.output.unwrap()["stdout"], "piped data");
    }

    #[test]
    fn test_shell_args_array() {
        let input = json!({
            "config": {"command": "echo", "args": ["-n", "no newline"]},
            "context": {}
        }).to_string();
        let result = process(&input);
        assert_eq!(result.status, "ok");
        assert_eq!(result.output.unwrap()["stdout"], "no newline");
    }

    #[test]
    fn test_shell_invalid_json() {
        let result = process("not json");
        assert_eq!(result.status, "error");
        assert!(result.error.unwrap().contains("Invalid input JSON"));
    }

    #[test]
    fn test_shell_stderr_capture() {
        let input = json!({
            "config": {"command": "sh", "args": ["-c", "echo err >&2; exit 1"]},
            "context": {}
        }).to_string();
        let result = process(&input);
        assert_eq!(result.status, "error");
        let out = result.output.unwrap();
        assert!(out["stderr"].as_str().unwrap().contains("err"));
        assert_eq!(out["exit_code"], 1);
    }

    #[test]
    fn test_shell_empty_args() {
        let input = json!({
            "config": {"command": "echo", "args": []},
            "context": {}
        }).to_string();
        let result = process(&input);
        assert_eq!(result.status, "ok");
    }

    #[test]
    fn test_shell_non_string_args_filtered() {
        let input = json!({
            "config": {"command": "echo", "args": ["hello", 42, "world"]},
            "context": {}
        }).to_string();
        let result = process(&input);
        assert_eq!(result.status, "ok");
        // Non-string args (42) are filtered out
        let out = result.output.unwrap();
        let args = out["args"].as_array().unwrap();
        assert_eq!(args.len(), 2);
    }
}
