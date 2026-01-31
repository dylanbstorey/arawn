//! Memory and notes integration tests.
//!
//! These tests verify memory persistence through the server API.
//!
//! Note: The current note store is a global static, so tests are not fully
//! isolated. Tests should account for notes created by other tests.

mod common;

use anyhow::Result;
use serde_json::json;

#[tokio::test]
async fn test_create_note() -> Result<()> {
    let server = common::TestServer::start().await?;

    let resp = server
        .post("/api/v1/notes")
        .json(&json!({
            "content": "This is a test note for test_create_note"
        }))
        .send()
        .await?;

    assert!(resp.status().is_success(), "Create note should succeed");

    let body: serde_json::Value = resp.json().await?;
    // Response is wrapped in { "note": { ... } }
    let note = body.get("note").expect("Response should have 'note' field");
    assert!(note.get("id").is_some(), "Note should have id");
    assert_eq!(
        note.get("content").and_then(|v| v.as_str()),
        Some("This is a test note for test_create_note"),
        "Note content should match"
    );

    Ok(())
}

#[tokio::test]
async fn test_list_notes_returns_array() -> Result<()> {
    let server = common::TestServer::start().await?;

    // List notes
    let resp = server.get("/api/v1/notes").send().await?;
    assert!(resp.status().is_success());

    let body: serde_json::Value = resp.json().await?;
    let notes = body
        .get("notes")
        .and_then(|v| v.as_array())
        .expect("Should have notes array");

    // Just verify it's an array - content depends on what other tests created
    assert!(notes.is_empty() || notes[0].get("id").is_some());

    Ok(())
}

#[tokio::test]
async fn test_create_note_appears_in_list() -> Result<()> {
    let server = common::TestServer::start().await?;

    // Create a note with unique content
    let unique_content = format!("Unique note {}", uuid::Uuid::new_v4());
    let resp = server
        .post("/api/v1/notes")
        .json(&json!({
            "content": unique_content
        }))
        .send()
        .await?;
    assert!(resp.status().is_success());

    // List notes
    let resp = server.get("/api/v1/notes").send().await?;
    assert!(resp.status().is_success());

    let body: serde_json::Value = resp.json().await?;
    let notes = body
        .get("notes")
        .and_then(|v| v.as_array())
        .expect("Should have notes array");

    // Find our unique note
    let found = notes
        .iter()
        .any(|n| n.get("content").and_then(|v| v.as_str()) == Some(&unique_content));
    assert!(found, "Created note should be in list");

    Ok(())
}

#[tokio::test]
async fn test_create_note_requires_content() -> Result<()> {
    let server = common::TestServer::start().await?;

    // Try to create note without content
    let resp = server.post("/api/v1/notes").json(&json!({})).send().await?;

    assert!(
        resp.status().is_client_error(),
        "Should reject note without content"
    );

    Ok(())
}

#[tokio::test]
async fn test_note_has_created_at() -> Result<()> {
    let server = common::TestServer::start().await?;

    let resp = server
        .post("/api/v1/notes")
        .json(&json!({
            "content": "Note with timestamp"
        }))
        .send()
        .await?;

    assert!(resp.status().is_success());

    let body: serde_json::Value = resp.json().await?;
    let note = body.get("note").expect("Should have note");
    assert!(
        note.get("created_at").is_some(),
        "Note should have created_at"
    );

    Ok(())
}

#[tokio::test]
async fn test_note_with_tags() -> Result<()> {
    let server = common::TestServer::start().await?;

    let resp = server
        .post("/api/v1/notes")
        .json(&json!({
            "content": "Tagged note",
            "tags": ["rust", "test"]
        }))
        .send()
        .await?;

    assert!(resp.status().is_success());

    let body: serde_json::Value = resp.json().await?;
    let note = body.get("note").expect("Should have note");
    let tags = note
        .get("tags")
        .and_then(|v| v.as_array())
        .expect("Should have tags");

    assert_eq!(tags.len(), 2);
    assert!(tags.iter().any(|t| t.as_str() == Some("rust")));
    assert!(tags.iter().any(|t| t.as_str() == Some("test")));

    Ok(())
}

#[tokio::test]
async fn test_memory_search_endpoint() -> Result<()> {
    let server = common::TestServer::start().await?;

    // Create a searchable note
    let resp = server
        .post("/api/v1/notes")
        .json(&json!({
            "content": "Searchable content about memory testing"
        }))
        .send()
        .await?;
    assert!(resp.status().is_success());

    // Search for it
    let resp = server
        .get("/api/v1/memory/search")
        .query(&[("q", "memory")])
        .send()
        .await?;

    assert!(resp.status().is_success());

    let body: serde_json::Value = resp.json().await?;
    assert!(body.get("results").is_some(), "Should have results field");
    assert!(body.get("query").is_some(), "Should have query field");

    Ok(())
}

#[tokio::test]
async fn test_memory_search_finds_matching_notes() -> Result<()> {
    let server = common::TestServer::start().await?;

    // Create a note with unique searchable content
    let unique_word = format!("xyzzy{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
    let resp = server
        .post("/api/v1/notes")
        .json(&json!({
            "content": format!("This note contains {}", unique_word)
        }))
        .send()
        .await?;
    assert!(resp.status().is_success());

    // Search for the unique word
    let resp = server
        .get("/api/v1/memory/search")
        .query(&[("q", &unique_word)])
        .send()
        .await?;

    assert!(resp.status().is_success());

    let body: serde_json::Value = resp.json().await?;
    let results = body
        .get("results")
        .and_then(|v| v.as_array())
        .expect("Should have results array");

    assert!(!results.is_empty(), "Search should find the note");
    assert!(
        results[0]
            .get("content")
            .and_then(|v| v.as_str())
            .map(|s| s.contains(&unique_word))
            .unwrap_or(false),
        "Result should contain the search term"
    );

    Ok(())
}
