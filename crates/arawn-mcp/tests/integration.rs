//! Integration tests for the MCP client.
//!
//! These tests use a mock MCP server to verify the full protocol flow.

use std::path::PathBuf;
use std::time::Duration;

use arawn_mcp::{HttpTransportConfig, McpClient, McpManager, McpServerConfig, McpTransport};
use serde_json::json;

/// Get the path to the mock MCP server binary.
fn mock_server_path() -> PathBuf {
    // The binary is built in target/debug or target/release
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // crates
    path.pop(); // arawn root
    path.push("target");
    path.push(if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    });
    path.push("mock-mcp-server");
    path
}

/// Check if the mock server binary exists.
fn mock_server_exists() -> bool {
    mock_server_path().exists()
}

#[test]
fn test_connect_and_initialize() {
    if !mock_server_exists() {
        eprintln!(
            "Skipping test: mock-mcp-server not built. Run `cargo build --package arawn-mcp` first."
        );
        return;
    }

    let config = McpServerConfig::new("test", mock_server_path().to_string_lossy().to_string());
    let mut client = McpClient::connect_stdio(config).expect("Failed to connect");

    let server_info = client.initialize().expect("Failed to initialize");
    assert_eq!(server_info.name, "mock-mcp-server");
    assert_eq!(server_info.version, "1.0.0");
    assert!(client.is_initialized());
}

#[test]
fn test_list_tools() {
    if !mock_server_exists() {
        eprintln!("Skipping test: mock-mcp-server not built");
        return;
    }

    let config = McpServerConfig::new("test", mock_server_path().to_string_lossy().to_string());
    let mut client = McpClient::connect_stdio(config).expect("Failed to connect");
    client.initialize().expect("Failed to initialize");

    let tools = client.list_tools().expect("Failed to list tools");
    assert_eq!(tools.len(), 4); // echo, add, slow, crash

    let echo_tool = tools
        .iter()
        .find(|t| t.name == "echo")
        .expect("echo tool not found");
    assert_eq!(
        echo_tool.description.as_deref(),
        Some("Echo back the input")
    );

    let add_tool = tools
        .iter()
        .find(|t| t.name == "add")
        .expect("add tool not found");
    assert_eq!(add_tool.description.as_deref(), Some("Add two numbers"));

    // Also verify the slow and crash tools are present
    assert!(
        tools.iter().any(|t| t.name == "slow"),
        "slow tool not found"
    );
    assert!(
        tools.iter().any(|t| t.name == "crash"),
        "crash tool not found"
    );
}

#[test]
fn test_call_echo_tool() {
    if !mock_server_exists() {
        eprintln!("Skipping test: mock-mcp-server not built");
        return;
    }

    let config = McpServerConfig::new("test", mock_server_path().to_string_lossy().to_string());
    let mut client = McpClient::connect_stdio(config).expect("Failed to connect");
    client.initialize().expect("Failed to initialize");

    let result = client
        .call_tool("echo", Some(json!({"message": "Hello, MCP!"})))
        .expect("Failed to call tool");

    assert!(!result.is_error());
    assert_eq!(result.text(), Some("Hello, MCP!".to_string()));
}

#[test]
fn test_call_add_tool() {
    if !mock_server_exists() {
        eprintln!("Skipping test: mock-mcp-server not built");
        return;
    }

    let config = McpServerConfig::new("test", mock_server_path().to_string_lossy().to_string());
    let mut client = McpClient::connect_stdio(config).expect("Failed to connect");
    client.initialize().expect("Failed to initialize");

    let result = client
        .call_tool("add", Some(json!({"a": 5, "b": 7})))
        .expect("Failed to call tool");

    assert!(!result.is_error());
    assert_eq!(result.text(), Some("12".to_string()));
}

#[test]
fn test_call_unknown_tool() {
    if !mock_server_exists() {
        eprintln!("Skipping test: mock-mcp-server not built");
        return;
    }

    let config = McpServerConfig::new("test", mock_server_path().to_string_lossy().to_string());
    let mut client = McpClient::connect_stdio(config).expect("Failed to connect");
    client.initialize().expect("Failed to initialize");

    let result = client
        .call_tool("nonexistent", Some(json!({})))
        .expect("Failed to call tool");

    assert!(result.is_error());
    assert!(result.text().unwrap_or_default().contains("Unknown tool"));
}

#[test]
fn test_call_before_initialize_fails() {
    if !mock_server_exists() {
        eprintln!("Skipping test: mock-mcp-server not built");
        return;
    }

    let config = McpServerConfig::new("test", mock_server_path().to_string_lossy().to_string());
    let client = McpClient::connect_stdio(config).expect("Failed to connect");

    // Calling tools before initialize should fail
    let result = client.list_tools();
    assert!(result.is_err());

    let result = client.call_tool("echo", Some(json!({"message": "test"})));
    assert!(result.is_err());
}

#[test]
fn test_shutdown() {
    if !mock_server_exists() {
        eprintln!("Skipping test: mock-mcp-server not built");
        return;
    }

    let config = McpServerConfig::new("test", mock_server_path().to_string_lossy().to_string());
    let mut client = McpClient::connect_stdio(config).expect("Failed to connect");
    client.initialize().expect("Failed to initialize");

    // Should be connected
    assert!(client.is_connected());

    // Shutdown should succeed
    client.shutdown().expect("Failed to shutdown");
}

// ─────────────────────────────────────────────────────────────────────────────
// Server crash recovery tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_server_crash_detection() {
    if !mock_server_exists() {
        eprintln!("Skipping test: mock-mcp-server not built");
        return;
    }

    // Start server with --crash-on flag
    let server_path = mock_server_path().to_string_lossy().to_string();
    let config = McpServerConfig::new("crash-test", &server_path)
        .with_arg("--crash-on")
        .with_arg("crash");

    let mut client = McpClient::connect_stdio(config).expect("Failed to connect");
    client.initialize().expect("Failed to initialize");

    // This should cause the server to crash
    let result = client.call_tool("crash", Some(json!({})));

    // The call should fail because the server crashed
    assert!(result.is_err(), "Expected error after server crash");
}

#[test]
fn test_connection_closed_detection() {
    if !mock_server_exists() {
        eprintln!("Skipping test: mock-mcp-server not built");
        return;
    }

    let config = McpServerConfig::new("test", mock_server_path().to_string_lossy().to_string());
    let mut client = McpClient::connect_stdio(config).expect("Failed to connect");
    client.initialize().expect("Failed to initialize");

    // Shutdown the client (kills the server)
    client.shutdown().expect("Failed to shutdown");

    // Subsequent calls should fail
    let result = client.list_tools();
    assert!(result.is_err(), "Expected error after shutdown");
}

// ─────────────────────────────────────────────────────────────────────────────
// Multiple concurrent MCP servers tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_multiple_servers() {
    if !mock_server_exists() {
        eprintln!("Skipping test: mock-mcp-server not built");
        return;
    }

    let server_path = mock_server_path().to_string_lossy().to_string();

    // Create manager with multiple server configurations
    let mut manager = McpManager::new();
    manager.add_server(McpServerConfig::new("server1", &server_path));
    manager.add_server(McpServerConfig::new("server2", &server_path));
    manager.add_server(McpServerConfig::new("server3", &server_path));

    // Connect all servers
    let connected = manager.connect_all().expect("Failed to connect servers");
    assert_eq!(connected, 3, "Expected 3 servers to connect");

    // Verify all are connected
    assert!(manager.is_connected("server1"));
    assert!(manager.is_connected("server2"));
    assert!(manager.is_connected("server3"));

    // List tools from all servers
    let all_tools = manager.list_all_tools().expect("Failed to list tools");
    assert_eq!(all_tools.len(), 3, "Expected tools from 3 servers");

    // Each server should have 4 tools (echo, add, slow, crash)
    for (name, tools) in &all_tools {
        assert_eq!(tools.len(), 4, "Server {} should have 4 tools", name);
    }

    // Shutdown all
    manager.shutdown_all().expect("Failed to shutdown");
    assert!(!manager.has_connections());
}

#[test]
fn test_manager_connect_and_disconnect_individual() {
    if !mock_server_exists() {
        eprintln!("Skipping test: mock-mcp-server not built");
        return;
    }

    let server_path = mock_server_path().to_string_lossy().to_string();

    let mut manager = McpManager::new();
    manager.add_server(McpServerConfig::new("server-a", &server_path));
    manager.add_server(McpServerConfig::new("server-b", &server_path));

    // Connect only server-a
    manager
        .connect_server_by_name("server-a")
        .expect("Failed to connect server-a");
    assert!(manager.is_connected("server-a"));
    assert!(!manager.is_connected("server-b"));

    // Connect server-b
    manager
        .connect_server_by_name("server-b")
        .expect("Failed to connect server-b");
    assert!(manager.is_connected("server-a"));
    assert!(manager.is_connected("server-b"));

    // Disconnect server-a
    assert!(manager.shutdown_server("server-a"));
    assert!(!manager.is_connected("server-a"));
    assert!(manager.is_connected("server-b"));

    // Cleanup
    manager.shutdown_all().expect("Failed to shutdown");
}

#[test]
fn test_manager_remove_server() {
    if !mock_server_exists() {
        eprintln!("Skipping test: mock-mcp-server not built");
        return;
    }

    let server_path = mock_server_path().to_string_lossy().to_string();

    let mut manager = McpManager::new();
    manager.add_server(McpServerConfig::new("to-remove", &server_path));
    manager.connect_all().expect("Failed to connect");

    assert!(manager.has_server("to-remove"));
    assert!(manager.is_connected("to-remove"));

    // Remove the server
    assert!(manager.remove_server("to-remove"));
    assert!(!manager.has_server("to-remove"));
    assert!(!manager.is_connected("to-remove"));

    // Try to remove again - should return false
    assert!(!manager.remove_server("to-remove"));
}

#[test]
fn test_manager_tool_count() {
    if !mock_server_exists() {
        eprintln!("Skipping test: mock-mcp-server not built");
        return;
    }

    let server_path = mock_server_path().to_string_lossy().to_string();

    let mut manager = McpManager::new();
    manager.add_server(McpServerConfig::new("s1", &server_path));
    manager.add_server(McpServerConfig::new("s2", &server_path));
    manager.connect_all().expect("Failed to connect");

    // 2 servers × 4 tools each = 8 total
    let count = manager.tool_count().expect("Failed to count tools");
    assert_eq!(count, 8);

    manager.shutdown_all().expect("Failed to shutdown");
}

// ─────────────────────────────────────────────────────────────────────────────
// HTTP transport tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_http_transport_config() {
    let config = HttpTransportConfig::new("http://localhost:8080/mcp")
        .with_timeout(Duration::from_secs(60))
        .with_retries(5)
        .with_header("Authorization", "Bearer token123");

    assert_eq!(config.url, "http://localhost:8080/mcp");
    assert_eq!(config.timeout, Duration::from_secs(60));
    assert_eq!(config.retries, 5);
    assert_eq!(config.headers.len(), 1);
}

#[test]
fn test_http_transport_creation() {
    let config = HttpTransportConfig::new("http://localhost:8080/mcp");
    let result = McpTransport::connect_http(config);

    // Should succeed (just creates the client, doesn't connect)
    assert!(result.is_ok());
    let transport = result.unwrap();
    assert!(transport.is_http());
    assert!(!transport.is_stdio());
}

#[test]
fn test_http_transport_invalid_url() {
    let config = HttpTransportConfig::new("not a valid url");
    let result = McpTransport::connect_http(config);

    // Should fail with invalid URL error
    assert!(result.is_err());
}

#[test]
fn test_server_config_http_builder() {
    let config = McpServerConfig::http("my-http-server", "http://api.example.com/mcp")
        .with_header("X-Api-Key", "secret123")
        .with_timeout(Duration::from_secs(45))
        .with_retries(2);

    assert_eq!(config.name, "my-http-server");
    assert_eq!(config.url, Some("http://api.example.com/mcp".to_string()));
    assert!(config.is_http());
    assert!(!config.is_stdio());
}

#[test]
fn test_client_connect_auto_selects_transport() {
    if !mock_server_exists() {
        eprintln!("Skipping test: mock-mcp-server not built");
        return;
    }

    let server_path = mock_server_path().to_string_lossy().to_string();

    // Stdio config should use stdio transport
    let stdio_config = McpServerConfig::new("stdio-test", &server_path);
    let stdio_client = McpClient::connect(stdio_config).expect("Failed to connect stdio");
    assert!(stdio_client.is_stdio());

    // HTTP config should use HTTP transport
    let http_config = McpServerConfig::http("http-test", "http://localhost:9999/mcp");
    let http_client = McpClient::connect(http_config).expect("Failed to connect http");
    assert!(http_client.is_http());
}

// ─────────────────────────────────────────────────────────────────────────────
// Additional client tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_all_tools_flat() {
    if !mock_server_exists() {
        eprintln!("Skipping test: mock-mcp-server not built");
        return;
    }

    let server_path = mock_server_path().to_string_lossy().to_string();

    let mut manager = McpManager::new();
    manager.add_server(McpServerConfig::new("s1", &server_path));
    manager.add_server(McpServerConfig::new("s2", &server_path));
    manager.connect_all().expect("Failed to connect");

    let flat_tools = manager.all_tools_flat().expect("Failed to get flat tools");

    // 2 servers × 4 tools = 8 (server_name, tool_info) tuples
    assert_eq!(flat_tools.len(), 8);

    // Verify server names are present
    let server_names: Vec<_> = flat_tools.iter().map(|(s, _)| s.as_str()).collect();
    assert!(server_names.iter().filter(|&&s| s == "s1").count() == 4);
    assert!(server_names.iter().filter(|&&s| s == "s2").count() == 4);

    manager.shutdown_all().expect("Failed to shutdown");
}
