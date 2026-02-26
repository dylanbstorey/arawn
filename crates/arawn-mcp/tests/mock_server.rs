//! Mock MCP server for integration testing.
//!
//! This is a simple MCP server that responds to initialize, tools/list, and tools/call.
//!
//! Usage:
//!   mock-mcp-server [--delay-ms N] [--crash-on TOOL] [--slow-tool TOOL:MS]
//!
//! Options:
//!   --delay-ms N       Add N ms delay to all responses
//!   --crash-on TOOL    Exit with code 1 when TOOL is called
//!   --slow-tool T:MS   Add MS delay when tool T is called

#![allow(dead_code)]

use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// JSON-RPC request structure.
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

/// JSON-RPC response structure.
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

/// Server configuration parsed from command line.
struct ServerConfig {
    delay_ms: u64,
    crash_on: Option<String>,
    slow_tools: Vec<(String, u64)>,
}

impl ServerConfig {
    fn from_args() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut config = Self {
            delay_ms: 0,
            crash_on: None,
            slow_tools: Vec::new(),
        };

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--delay-ms" => {
                    if i + 1 < args.len() {
                        config.delay_ms = args[i + 1].parse().unwrap_or(0);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--crash-on" => {
                    if i + 1 < args.len() {
                        config.crash_on = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--slow-tool" => {
                    if i + 1 < args.len() {
                        if let Some((tool, ms)) = args[i + 1].split_once(':') {
                            if let Ok(ms) = ms.parse() {
                                config.slow_tools.push((tool.to_string(), ms));
                            }
                        }
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                _ => {
                    i += 1;
                }
            }
        }

        config
    }

    fn get_tool_delay(&self, tool_name: &str) -> u64 {
        for (tool, ms) in &self.slow_tools {
            if tool == tool_name {
                return *ms;
            }
        }
        0
    }
}

fn main() {
    let config = ServerConfig::from_args();
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());

    loop {
        // Read Content-Length header
        let mut header_line = String::new();
        let mut content_length: Option<usize> = None;

        loop {
            header_line.clear();
            if reader.read_line(&mut header_line).unwrap() == 0 {
                return; // EOF
            }

            let trimmed = header_line.trim();
            if trimmed.is_empty() {
                break;
            }

            if let Some(len_str) = trimmed.strip_prefix("Content-Length:") {
                content_length = Some(len_str.trim().parse().unwrap());
            }
        }

        let content_length = match content_length {
            Some(len) => len,
            None => continue,
        };

        // Read JSON body
        let mut body = vec![0u8; content_length];
        reader.read_exact(&mut body).unwrap();

        let body_str = String::from_utf8(body).unwrap();

        // Try to parse as request (might be notification)
        let request: JsonRpcRequest = match serde_json::from_str(&body_str) {
            Ok(req) => req,
            Err(_) => continue, // Skip notifications
        };

        // Apply global delay
        if config.delay_ms > 0 {
            thread::sleep(Duration::from_millis(config.delay_ms));
        }

        let response = handle_request(&request, &config);

        // Send response
        let response_json = serde_json::to_string(&response).unwrap();
        write!(
            stdout,
            "Content-Length: {}\r\n\r\n{}",
            response_json.len(),
            response_json
        )
        .unwrap();
        stdout.flush().unwrap();
    }
}

fn handle_request(request: &JsonRpcRequest, config: &ServerConfig) -> JsonRpcResponse {
    let result = match request.method.as_str() {
        "initialize" => Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "mock-mcp-server",
                "version": "1.0.0"
            }
        })),
        "tools/list" => Some(json!({
            "tools": [
                {
                    "name": "echo",
                    "description": "Echo back the input",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "message": { "type": "string" }
                        },
                        "required": ["message"]
                    }
                },
                {
                    "name": "add",
                    "description": "Add two numbers",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "a": { "type": "number" },
                            "b": { "type": "number" }
                        },
                        "required": ["a", "b"]
                    }
                },
                {
                    "name": "slow",
                    "description": "A slow tool for testing timeouts",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "delay_ms": { "type": "number" }
                        }
                    }
                },
                {
                    "name": "crash",
                    "description": "Crashes the server (for testing)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                }
            ]
        })),
        "tools/call" => {
            let params = request.params.as_ref().unwrap();
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));

            // Check if we should crash on this tool
            if let Some(ref crash_tool) = config.crash_on {
                if crash_tool == tool_name {
                    std::process::exit(1);
                }
            }

            // Apply tool-specific delay
            let tool_delay = config.get_tool_delay(tool_name);
            if tool_delay > 0 {
                thread::sleep(Duration::from_millis(tool_delay));
            }

            match tool_name {
                "echo" => {
                    let message = args.get("message").and_then(|v| v.as_str()).unwrap_or("");
                    Some(json!({
                        "content": [
                            { "type": "text", "text": message }
                        ]
                    }))
                }
                "add" => {
                    let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    Some(json!({
                        "content": [
                            { "type": "text", "text": format!("{}", a + b) }
                        ]
                    }))
                }
                "slow" => {
                    let delay = args
                        .get("delay_ms")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(1000);
                    thread::sleep(Duration::from_millis(delay));
                    Some(json!({
                        "content": [
                            { "type": "text", "text": format!("Slept for {} ms", delay) }
                        ]
                    }))
                }
                "crash" => {
                    // Exit immediately
                    std::process::exit(1);
                }
                _ => Some(json!({
                    "content": [
                        { "type": "text", "text": format!("Unknown tool: {}", tool_name) }
                    ],
                    "isError": true
                })),
            }
        }
        _ => None,
    };

    let error = if result.is_none() {
        Some(json!({
            "code": -32601,
            "message": format!("Method not found: {}", request.method)
        }))
    } else {
        None
    };

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result,
        error,
    }
}
