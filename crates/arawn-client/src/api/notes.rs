//! Notes API.

use crate::client::ArawnClient;
use crate::error::Result;
use crate::types::{CreateNoteRequest, ListNotesResponse, Note, NoteResponse, UpdateNoteRequest};

/// Query parameters for listing notes.
#[derive(Debug, Default, serde::Serialize)]
pub struct ListNotesQuery {
    /// Filter by tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Maximum number of notes to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// Notes API client.
pub struct NotesApi {
    client: ArawnClient,
}

impl NotesApi {
    pub(crate) fn new(client: ArawnClient) -> Self {
        Self { client }
    }

    /// List all notes.
    pub async fn list(&self) -> Result<ListNotesResponse> {
        self.client.get("notes").await
    }

    /// List notes with query parameters.
    pub async fn list_with_query(&self, query: ListNotesQuery) -> Result<ListNotesResponse> {
        self.client.get_with_query("notes", &query).await
    }

    /// List notes with a specific tag.
    pub async fn list_by_tag(&self, tag: &str) -> Result<ListNotesResponse> {
        self.list_with_query(ListNotesQuery {
            tag: Some(tag.to_string()),
            ..Default::default()
        })
        .await
    }

    /// Get a note by ID.
    pub async fn get(&self, id: &str) -> Result<Note> {
        let response: NoteResponse = self.client.get(&format!("notes/{}", id)).await?;
        Ok(response.note)
    }

    /// Create a new note.
    pub async fn create(&self, request: CreateNoteRequest) -> Result<Note> {
        let response: NoteResponse = self.client.post("notes", &request).await?;
        Ok(response.note)
    }

    /// Create a note with just content.
    pub async fn create_simple(&self, content: impl Into<String>) -> Result<Note> {
        self.create(CreateNoteRequest {
            content: content.into(),
            tags: Vec::new(),
        })
        .await
    }

    /// Update a note.
    pub async fn update(&self, id: &str, request: UpdateNoteRequest) -> Result<Note> {
        let response: NoteResponse = self.client.put(&format!("notes/{}", id), &request).await?;
        Ok(response.note)
    }

    /// Delete a note.
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("notes/{}", id)).await
    }
}
