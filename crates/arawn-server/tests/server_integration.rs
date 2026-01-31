//! Server integration tests.
//!
//! These tests verify the server starts correctly and handles requests.

mod common;

use anyhow::Result;

#[tokio::test]
async fn test_server_starts_and_responds_to_health() -> Result<()> {
    let server = common::TestServer::start().await?;

    // Health check should succeed
    let healthy = server.health().await?;
    assert!(healthy, "Server should be healthy");

    Ok(())
}

#[tokio::test]
async fn test_server_health_returns_version() -> Result<()> {
    let server = common::TestServer::start().await?;

    let resp = server
        .client
        .get(format!("{}/health", server.base_url()))
        .send()
        .await?;

    assert!(resp.status().is_success());

    let body: serde_json::Value = resp.json().await?;
    assert!(body.get("status").is_some());
    assert!(body.get("version").is_some());

    Ok(())
}

#[tokio::test]
async fn test_api_requires_auth() -> Result<()> {
    let server = common::TestServer::start().await?;

    // Request without auth should fail
    let resp = server
        .client
        .get(format!("{}/api/v1/sessions", server.base_url()))
        .send()
        .await?;

    assert_eq!(resp.status().as_u16(), 401);

    Ok(())
}

#[tokio::test]
async fn test_api_accepts_valid_auth() -> Result<()> {
    let server = common::TestServer::start().await?;

    // Request with auth should succeed
    let resp = server.get("/api/v1/sessions").send().await?;

    assert!(resp.status().is_success());

    Ok(())
}

#[tokio::test]
async fn test_api_rejects_invalid_auth() -> Result<()> {
    let server = common::TestServer::start().await?;

    // Request with wrong token should fail
    let resp = server
        .client
        .get(format!("{}/api/v1/sessions", server.base_url()))
        .bearer_auth("wrong-token")
        .send()
        .await?;

    assert_eq!(resp.status().as_u16(), 401);

    Ok(())
}

#[tokio::test]
async fn test_multiple_servers_different_ports() -> Result<()> {
    // Start two servers - they should get different ports
    let server1 = common::TestServer::start().await?;
    let server2 = common::TestServer::start().await?;

    assert_ne!(
        server1.addr, server2.addr,
        "Servers should be on different ports"
    );

    // Both should be healthy
    assert!(server1.health().await?);
    assert!(server2.health().await?);

    Ok(())
}
