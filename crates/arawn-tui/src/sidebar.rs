//! Sidebar state for workstreams and sessions navigation.

use crate::sessions::SessionSummary;
use chrono::{Duration, Utc};

/// A workstream entry for display.
#[derive(Debug, Clone)]
pub struct WorkstreamEntry {
    /// Workstream ID.
    pub id: String,
    /// Workstream name/title.
    pub name: String,
    /// Number of sessions in this workstream.
    pub session_count: usize,
    /// Whether this is the current workstream.
    pub is_current: bool,
}

/// Which section of the sidebar has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarSection {
    #[default]
    Workstreams,
    Sessions,
}

/// Sidebar state managing workstreams and sessions lists.
///
/// The sidebar has two states:
/// - **Closed**: Shows a minimal hint line, not interactive
/// - **Open**: Full expanded view, has focus and is interactive
///
/// Sessions list always has "+ New Session" at index 0.
/// Actual sessions are at indices 1+.
#[derive(Debug)]
pub struct Sidebar {
    /// Whether the sidebar is open (expanded and focused).
    pub open: bool,
    /// Current focused section.
    pub section: SidebarSection,
    /// Available workstreams.
    pub workstreams: Vec<WorkstreamEntry>,
    /// Selected workstream index.
    pub workstream_index: usize,
    /// Sessions for the selected workstream.
    pub sessions: Vec<SessionSummary>,
    /// Selected session index (0 = "+ New Session", 1+ = actual sessions).
    pub session_index: usize,
    /// Filter text for searching.
    pub filter: String,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Sidebar {
    /// Create a new sidebar (starts closed).
    pub fn new() -> Self {
        Self {
            open: false,
            section: SidebarSection::Workstreams,
            workstreams: Vec::new(),
            workstream_index: 0,
            sessions: Vec::new(),
            session_index: 0,
            filter: String::new(),
        }
    }

    /// Toggle sidebar open/closed.
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    /// Open the sidebar.
    pub fn open(&mut self) {
        self.open = true;
    }

    /// Close the sidebar.
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Check if the sidebar is open.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Switch focus between workstreams and sessions.
    pub fn toggle_section(&mut self) {
        self.section = match self.section {
            SidebarSection::Workstreams => SidebarSection::Sessions,
            SidebarSection::Sessions => SidebarSection::Workstreams,
        };
    }

    /// Move selection up in current section (circular).
    pub fn select_prev(&mut self) {
        match self.section {
            SidebarSection::Workstreams => {
                if self.workstreams.is_empty() {
                    return;
                }
                if self.workstream_index > 0 {
                    self.workstream_index -= 1;
                } else {
                    // Wrap to bottom
                    self.workstream_index = self.workstreams.len() - 1;
                }
                self.refresh_sessions_for_workstream();
            }
            SidebarSection::Sessions => {
                // Index 0 = "+ New Session", indices 1..=len = actual sessions
                // Total items = sessions.len() + 1 (for "+ New Session")
                let total = self.sessions.len() + 1;
                if self.session_index > 0 {
                    self.session_index -= 1;
                } else {
                    // Wrap to bottom (last actual session)
                    self.session_index = total - 1;
                }
            }
        }
    }

    /// Move selection down in current section (circular).
    pub fn select_next(&mut self) {
        match self.section {
            SidebarSection::Workstreams => {
                if self.workstreams.is_empty() {
                    return;
                }
                if self.workstream_index < self.workstreams.len() - 1 {
                    self.workstream_index += 1;
                } else {
                    // Wrap to top
                    self.workstream_index = 0;
                }
                self.refresh_sessions_for_workstream();
            }
            SidebarSection::Sessions => {
                // Index 0 = "+ New Session", indices 1..=len = actual sessions
                // Total items = sessions.len() + 1 (for "+ New Session")
                let total = self.sessions.len() + 1;
                if self.session_index < total - 1 {
                    self.session_index += 1;
                } else {
                    // Wrap to top ("+ New Session")
                    self.session_index = 0;
                }
            }
        }
    }

    /// Get the currently selected workstream.
    pub fn selected_workstream(&self) -> Option<&WorkstreamEntry> {
        self.workstreams.get(self.workstream_index)
    }

    /// Check if "+ New Session" is currently selected.
    pub fn is_new_session_selected(&self) -> bool {
        self.session_index == 0
    }

    /// Get the currently selected session (None if "+ New Session" is selected).
    pub fn selected_session(&self) -> Option<&SessionSummary> {
        if self.session_index == 0 {
            None // "+ New Session" is selected
        } else {
            self.sessions.get(self.session_index - 1)
        }
    }

    /// Add a character to the filter.
    pub fn filter_push(&mut self, c: char) {
        self.filter.push(c);
    }

    /// Remove the last character from the filter.
    pub fn filter_pop(&mut self) {
        self.filter.pop();
    }

    /// Clear the filter.
    pub fn filter_clear(&mut self) {
        self.filter.clear();
    }

    /// Get visible workstreams (filtered).
    pub fn visible_workstreams(&self) -> impl Iterator<Item = (bool, &WorkstreamEntry)> {
        let filter = self.filter.to_lowercase();
        self.workstreams
            .iter()
            .enumerate()
            .filter(move |(_, ws)| filter.is_empty() || ws.name.to_lowercase().contains(&filter))
            .map(move |(i, ws)| (i == self.workstream_index, ws))
    }

    /// Get visible sessions (filtered).
    /// Note: session_index 0 = "+ New Session", actual sessions start at index 1.
    pub fn visible_sessions(&self) -> impl Iterator<Item = (bool, &SessionSummary)> {
        let filter = self.filter.to_lowercase();
        let session_index = self.session_index;
        self.sessions
            .iter()
            .enumerate()
            .filter(move |(_, s)| filter.is_empty() || s.title.to_lowercase().contains(&filter))
            // Data index i maps to visual index i+1 (since 0 is "+ New Session")
            .map(move |(i, s)| (i + 1 == session_index, s))
    }

    /// Refresh sessions list when workstream selection changes.
    fn refresh_sessions_for_workstream(&mut self) {
        // TODO: In the future, fetch sessions from server for the selected workstream
        // For now, generate mock data based on selected workstream
        self.session_index = 0;
        self.populate_mock_sessions();
    }

    /// Populate with mock workstreams for UI demonstration.
    pub fn populate_mock_workstreams(&mut self, current_workstream: &str) {
        self.workstreams = vec![
            WorkstreamEntry {
                id: "mock-1".to_string(),
                name: "default".to_string(),
                session_count: 5,
                is_current: current_workstream == "default",
            },
            WorkstreamEntry {
                id: "mock-2".to_string(),
                name: "project-alpha".to_string(),
                session_count: 3,
                is_current: current_workstream == "project-alpha",
            },
            WorkstreamEntry {
                id: "mock-3".to_string(),
                name: "research-notes".to_string(),
                session_count: 8,
                is_current: current_workstream == "research-notes",
            },
            WorkstreamEntry {
                id: "mock-4".to_string(),
                name: "code-reviews".to_string(),
                session_count: 2,
                is_current: current_workstream == "code-reviews",
            },
        ];

        // Set initial selection to current workstream
        self.workstream_index = self
            .workstreams
            .iter()
            .position(|ws| ws.is_current)
            .unwrap_or(0);

        self.populate_mock_sessions();
    }

    /// Populate sessions for the currently selected workstream.
    fn populate_mock_sessions(&mut self) {
        let now = Utc::now();
        let ws_name = self
            .selected_workstream()
            .map(|ws| ws.name.as_str())
            .unwrap_or("default");

        // Generate different sessions based on workstream
        self.sessions = match ws_name {
            "default" => vec![
                SessionSummary {
                    id: "default-1".to_string(),
                    title: "Current session".to_string(),
                    last_active: now,
                    message_count: 0,
                    is_current: true,
                },
                SessionSummary {
                    id: "default-2".to_string(),
                    title: "async/await explanation".to_string(),
                    last_active: now - Duration::minutes(15),
                    message_count: 8,
                    is_current: false,
                },
                SessionSummary {
                    id: "default-3".to_string(),
                    title: "Debug auth middleware".to_string(),
                    last_active: now - Duration::hours(2),
                    message_count: 12,
                    is_current: false,
                },
                SessionSummary {
                    id: "default-4".to_string(),
                    title: "Rust workspace setup".to_string(),
                    last_active: now - Duration::days(1),
                    message_count: 5,
                    is_current: false,
                },
                SessionSummary {
                    id: "default-5".to_string(),
                    title: "Memory indexing questions".to_string(),
                    last_active: now - Duration::days(3),
                    message_count: 15,
                    is_current: false,
                },
            ],
            "project-alpha" => vec![
                SessionSummary {
                    id: "alpha-1".to_string(),
                    title: "API design discussion".to_string(),
                    last_active: now - Duration::hours(1),
                    message_count: 20,
                    is_current: false,
                },
                SessionSummary {
                    id: "alpha-2".to_string(),
                    title: "Database schema review".to_string(),
                    last_active: now - Duration::days(2),
                    message_count: 8,
                    is_current: false,
                },
                SessionSummary {
                    id: "alpha-3".to_string(),
                    title: "Performance optimization".to_string(),
                    last_active: now - Duration::days(5),
                    message_count: 35,
                    is_current: false,
                },
            ],
            "research-notes" => vec![
                SessionSummary {
                    id: "research-1".to_string(),
                    title: "LLM fine-tuning approaches".to_string(),
                    last_active: now - Duration::minutes(30),
                    message_count: 45,
                    is_current: false,
                },
                SessionSummary {
                    id: "research-2".to_string(),
                    title: "RAG implementation patterns".to_string(),
                    last_active: now - Duration::hours(4),
                    message_count: 28,
                    is_current: false,
                },
                SessionSummary {
                    id: "research-3".to_string(),
                    title: "Vector database comparison".to_string(),
                    last_active: now - Duration::days(1),
                    message_count: 15,
                    is_current: false,
                },
                SessionSummary {
                    id: "research-4".to_string(),
                    title: "Embedding models review".to_string(),
                    last_active: now - Duration::days(3),
                    message_count: 22,
                    is_current: false,
                },
                SessionSummary {
                    id: "research-5".to_string(),
                    title: "Prompt engineering tips".to_string(),
                    last_active: now - Duration::days(4),
                    message_count: 18,
                    is_current: false,
                },
                SessionSummary {
                    id: "research-6".to_string(),
                    title: "Agent architecture patterns".to_string(),
                    last_active: now - Duration::days(6),
                    message_count: 32,
                    is_current: false,
                },
                SessionSummary {
                    id: "research-7".to_string(),
                    title: "Tool use strategies".to_string(),
                    last_active: now - Duration::days(7),
                    message_count: 12,
                    is_current: false,
                },
                SessionSummary {
                    id: "research-8".to_string(),
                    title: "Memory systems comparison".to_string(),
                    last_active: now - Duration::days(10),
                    message_count: 40,
                    is_current: false,
                },
            ],
            "code-reviews" => vec![
                SessionSummary {
                    id: "review-1".to_string(),
                    title: "PR #142: Auth refactor".to_string(),
                    last_active: now - Duration::hours(3),
                    message_count: 15,
                    is_current: false,
                },
                SessionSummary {
                    id: "review-2".to_string(),
                    title: "PR #139: WebSocket fixes".to_string(),
                    last_active: now - Duration::days(2),
                    message_count: 8,
                    is_current: false,
                },
            ],
            _ => vec![],
        };
    }

    /// Set the current session as selected in sessions list.
    pub fn set_current_session(&mut self, session_id: &str) {
        for session in &mut self.sessions {
            session.is_current = session.id == session_id;
        }
        // Update selection to current (add 1 for "+ New Session" at index 0)
        if let Some(pos) = self.sessions.iter().position(|s| s.id == session_id) {
            self.session_index = pos + 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidebar_toggle() {
        let mut sidebar = Sidebar::new();
        // Starts closed
        assert!(!sidebar.is_open());

        // First toggle: open
        sidebar.toggle();
        assert!(sidebar.is_open());

        // Second toggle: close
        sidebar.toggle();
        assert!(!sidebar.is_open());

        // Third toggle: open again
        sidebar.toggle();
        assert!(sidebar.is_open());
    }

    #[test]
    fn test_section_toggle() {
        let mut sidebar = Sidebar::new();
        assert_eq!(sidebar.section, SidebarSection::Workstreams);

        sidebar.toggle_section();
        assert_eq!(sidebar.section, SidebarSection::Sessions);

        sidebar.toggle_section();
        assert_eq!(sidebar.section, SidebarSection::Workstreams);
    }

    #[test]
    fn test_navigation() {
        let mut sidebar = Sidebar::new();
        sidebar.populate_mock_workstreams("default");

        // Navigate workstreams
        assert_eq!(sidebar.workstream_index, 0);
        sidebar.select_next();
        assert_eq!(sidebar.workstream_index, 1);
        sidebar.select_prev();
        assert_eq!(sidebar.workstream_index, 0);

        // Circular: wrap from top to bottom
        sidebar.select_prev();
        assert_eq!(sidebar.workstream_index, 3); // 4 workstreams, last index is 3

        // Circular: wrap from bottom to top
        sidebar.select_next();
        assert_eq!(sidebar.workstream_index, 0);

        // Switch to sessions
        sidebar.toggle_section();

        // Session index 0 = "+ New Session" (no actual session selected)
        assert_eq!(sidebar.session_index, 0);
        assert!(sidebar.is_new_session_selected());
        assert!(sidebar.selected_session().is_none());

        // Move to first actual session
        sidebar.select_next();
        assert_eq!(sidebar.session_index, 1);
        assert!(!sidebar.is_new_session_selected());
        assert!(sidebar.selected_session().is_some());

        // Move back to "+ New Session"
        sidebar.select_prev();
        assert_eq!(sidebar.session_index, 0);
        assert!(sidebar.is_new_session_selected());

        // Circular sessions: wrap from top to bottom
        sidebar.select_prev();
        // "default" workstream has 5 sessions, so total items = 6 (including "+ New Session")
        // Last index = 5
        assert_eq!(sidebar.session_index, 5);
        assert!(!sidebar.is_new_session_selected());

        // Circular sessions: wrap from bottom to top
        sidebar.select_next();
        assert_eq!(sidebar.session_index, 0);
        assert!(sidebar.is_new_session_selected());
    }
}
