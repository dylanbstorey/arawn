//! Sidebar state for workstreams and sessions navigation.

use crate::sessions::SessionSummary;

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
    /// Whether this is a scratch workstream.
    pub is_scratch: bool,
    /// Total disk usage in bytes.
    pub usage_bytes: Option<u64>,
    /// Usage limit in bytes (None = no limit).
    pub limit_bytes: Option<u64>,
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
    ///
    /// Returns Some(workstream_id) if workstream selection changed and sessions should be refreshed.
    pub fn select_prev(&mut self) -> Option<String> {
        match self.section {
            SidebarSection::Workstreams => {
                if self.workstreams.is_empty() {
                    return None;
                }
                if self.workstream_index > 0 {
                    self.workstream_index -= 1;
                } else {
                    // Wrap to bottom
                    self.workstream_index = self.workstreams.len() - 1;
                }
                // Clear sessions and return workstream ID for refresh
                self.session_index = 0;
                self.sessions.clear();
                self.selected_workstream().map(|ws| ws.id.clone())
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
                None
            }
        }
    }

    /// Move selection down in current section (circular).
    ///
    /// Returns Some(workstream_id) if workstream selection changed and sessions should be refreshed.
    pub fn select_next(&mut self) -> Option<String> {
        match self.section {
            SidebarSection::Workstreams => {
                if self.workstreams.is_empty() {
                    return None;
                }
                if self.workstream_index < self.workstreams.len() - 1 {
                    self.workstream_index += 1;
                } else {
                    // Wrap to top
                    self.workstream_index = 0;
                }
                // Clear sessions and return workstream ID for refresh
                self.session_index = 0;
                self.sessions.clear();
                self.selected_workstream().map(|ws| ws.id.clone())
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
                None
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
    use chrono::Utc;

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

    /// Helper to set up test workstreams.
    fn setup_test_workstreams(sidebar: &mut Sidebar) {
        sidebar.workstreams = vec![
            WorkstreamEntry {
                id: "ws-1".to_string(),
                name: "default".to_string(),
                session_count: 5,
                is_current: true,
                is_scratch: false,
                usage_bytes: None,
                limit_bytes: None,
            },
            WorkstreamEntry {
                id: "ws-2".to_string(),
                name: "project-alpha".to_string(),
                session_count: 3,
                is_current: false,
                is_scratch: false,
                usage_bytes: None,
                limit_bytes: None,
            },
            WorkstreamEntry {
                id: "ws-3".to_string(),
                name: "research".to_string(),
                session_count: 8,
                is_current: false,
                is_scratch: false,
                usage_bytes: None,
                limit_bytes: None,
            },
            WorkstreamEntry {
                id: "ws-4".to_string(),
                name: "reviews".to_string(),
                session_count: 2,
                is_current: false,
                is_scratch: false,
                usage_bytes: None,
                limit_bytes: None,
            },
        ];
    }

    /// Helper to set up test sessions.
    fn setup_test_sessions(sidebar: &mut Sidebar) {
        let now = Utc::now();
        sidebar.sessions = vec![
            SessionSummary {
                id: "sess-1".to_string(),
                title: "Session 1".to_string(),
                last_active: now,
                message_count: 5,
                is_current: true,
            },
            SessionSummary {
                id: "sess-2".to_string(),
                title: "Session 2".to_string(),
                last_active: now,
                message_count: 10,
                is_current: false,
            },
            SessionSummary {
                id: "sess-3".to_string(),
                title: "Session 3".to_string(),
                last_active: now,
                message_count: 3,
                is_current: false,
            },
            SessionSummary {
                id: "sess-4".to_string(),
                title: "Session 4".to_string(),
                last_active: now,
                message_count: 8,
                is_current: false,
            },
            SessionSummary {
                id: "sess-5".to_string(),
                title: "Session 5".to_string(),
                last_active: now,
                message_count: 12,
                is_current: false,
            },
        ];
    }

    #[test]
    fn test_navigation() {
        let mut sidebar = Sidebar::new();
        setup_test_workstreams(&mut sidebar);
        setup_test_sessions(&mut sidebar);

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

        // Switch to sessions (re-add sessions since workstream navigation clears them)
        setup_test_sessions(&mut sidebar);
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
        // 5 sessions, so total items = 6 (including "+ New Session")
        // Last index = 5
        assert_eq!(sidebar.session_index, 5);
        assert!(!sidebar.is_new_session_selected());

        // Circular sessions: wrap from bottom to top
        sidebar.select_next();
        assert_eq!(sidebar.session_index, 0);
        assert!(sidebar.is_new_session_selected());
    }

    #[test]
    fn test_workstream_navigation_returns_id() {
        let mut sidebar = Sidebar::new();
        setup_test_workstreams(&mut sidebar);

        // Navigate down returns new workstream ID
        let id = sidebar.select_next();
        assert_eq!(id, Some("ws-2".to_string()));
        assert_eq!(sidebar.workstream_index, 1);
        // Sessions should be cleared
        assert!(sidebar.sessions.is_empty());

        // Navigate up returns new workstream ID
        let id = sidebar.select_prev();
        assert_eq!(id, Some("ws-1".to_string()));
        assert_eq!(sidebar.workstream_index, 0);

        // Session navigation returns None
        setup_test_sessions(&mut sidebar);
        sidebar.toggle_section();
        let id = sidebar.select_next();
        assert_eq!(id, None);
    }
}
