//! Focus management for TUI panels and overlays.
//!
//! Centralizes focus state, transitions, and input routing to make
//! adding new panels easier and focus behavior more predictable.

/// Focus targets - all focusable areas in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusTarget {
    /// Default - chat input field.
    #[default]
    Input,
    /// Sidebar navigation (workstreams + sessions tree).
    Sidebar,
    /// Tool output pane.
    ToolPane,
    /// Logs panel.
    Logs,
    /// Command palette overlay (Ctrl+K).
    CommandPalette,
    /// Sessions list overlay.
    Sessions,
    /// Workstreams list overlay.
    Workstreams,
}

impl FocusTarget {
    /// Returns true if this target is an overlay (modal popup).
    ///
    /// Overlays take priority over regular panels and return to
    /// the previous focus when dismissed.
    pub fn is_overlay(&self) -> bool {
        matches!(
            self,
            FocusTarget::CommandPalette | FocusTarget::Sessions | FocusTarget::Workstreams
        )
    }

    /// Returns true if this is a main panel (not an overlay).
    pub fn is_panel(&self) -> bool {
        !self.is_overlay()
    }

    /// Get the display name for this focus target.
    pub fn name(&self) -> &'static str {
        match self {
            FocusTarget::Input => "Input",
            FocusTarget::Sidebar => "Sidebar",
            FocusTarget::ToolPane => "Tools",
            FocusTarget::Logs => "Logs",
            FocusTarget::CommandPalette => "Command Palette",
            FocusTarget::Sessions => "Sessions",
            FocusTarget::Workstreams => "Workstreams",
        }
    }
}

/// Main panels that can be cycled through with Tab.
const CYCLABLE_PANELS: &[FocusTarget] = &[
    FocusTarget::Input,
    FocusTarget::Sidebar,
    FocusTarget::ToolPane,
    FocusTarget::Logs,
];

/// Manages focus state and transitions for the TUI.
///
/// Handles:
/// - Current focus tracking
/// - Overlay stack (command palette takes priority)
/// - Focus cycling between main panels
/// - Return-to-previous behavior for overlays
#[derive(Debug, Clone)]
pub struct FocusManager {
    /// Current focus target.
    current: FocusTarget,
    /// Previous focus (for returning from overlays).
    previous: Option<FocusTarget>,
    /// Stack of active overlays (supports nested overlays if needed).
    overlay_stack: Vec<FocusTarget>,
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusManager {
    /// Create a new focus manager with default focus on Input.
    pub fn new() -> Self {
        Self {
            current: FocusTarget::default(),
            previous: None,
            overlay_stack: Vec::new(),
        }
    }

    /// Get the current focus target.
    pub fn current(&self) -> FocusTarget {
        self.current
    }

    /// Check if currently focused on a specific target.
    pub fn is(&self, target: FocusTarget) -> bool {
        self.current == target
    }

    /// Check if any overlay is active.
    pub fn has_overlay(&self) -> bool {
        !self.overlay_stack.is_empty()
    }

    /// Direct focus change to a panel (not an overlay).
    ///
    /// For overlays, use `push_overlay` instead.
    pub fn focus(&mut self, target: FocusTarget) {
        if target.is_overlay() {
            // Use push_overlay for overlays to maintain proper stack behavior
            self.push_overlay(target);
        } else {
            // Store previous for potential return
            if self.current != target {
                self.previous = Some(self.current);
            }
            self.current = target;
            // Clear overlay stack when directly focusing a panel
            self.overlay_stack.clear();
        }
    }

    /// Open an overlay, remembering the current focus to return to.
    pub fn push_overlay(&mut self, overlay: FocusTarget) {
        debug_assert!(
            overlay.is_overlay(),
            "push_overlay should only be used with overlay targets"
        );

        // Store current as previous if it's a panel
        if self.current.is_panel() {
            self.previous = Some(self.current);
        }

        self.overlay_stack.push(overlay);
        self.current = overlay;
    }

    /// Close the current overlay and return to previous focus.
    ///
    /// Returns the overlay that was closed, or None if no overlay was active.
    pub fn pop_overlay(&mut self) -> Option<FocusTarget> {
        let closed = self.overlay_stack.pop();
        if closed.is_some() {
            // Return to previous overlay or to the saved panel
            self.current = self
                .overlay_stack
                .last()
                .copied()
                .or(self.previous)
                .unwrap_or(FocusTarget::Input);
        }
        closed
    }

    /// Close all overlays and return to the previous panel focus.
    pub fn close_all_overlays(&mut self) {
        self.overlay_stack.clear();
        self.current = self.previous.unwrap_or(FocusTarget::Input);
    }

    /// Cycle focus to the next main panel.
    ///
    /// Only cycles through panels (Input, Sidebar, ToolPane, Logs).
    /// Does nothing if an overlay is active.
    pub fn cycle_next(&mut self) {
        if self.has_overlay() {
            return; // Don't cycle while overlay is active
        }

        if let Some(pos) = CYCLABLE_PANELS.iter().position(|&p| p == self.current) {
            let next_pos = (pos + 1) % CYCLABLE_PANELS.len();
            self.focus(CYCLABLE_PANELS[next_pos]);
        } else {
            // Current focus isn't cyclable, go to Input
            self.focus(FocusTarget::Input);
        }
    }

    /// Cycle focus to the previous main panel.
    ///
    /// Only cycles through panels (Input, Sidebar, ToolPane, Logs).
    /// Does nothing if an overlay is active.
    pub fn cycle_prev(&mut self) {
        if self.has_overlay() {
            return; // Don't cycle while overlay is active
        }

        if let Some(pos) = CYCLABLE_PANELS.iter().position(|&p| p == self.current) {
            let prev_pos = if pos == 0 {
                CYCLABLE_PANELS.len() - 1
            } else {
                pos - 1
            };
            self.focus(CYCLABLE_PANELS[prev_pos]);
        } else {
            // Current focus isn't cyclable, go to Input
            self.focus(FocusTarget::Input);
        }
    }

    /// Toggle focus between the current panel and a specific target.
    ///
    /// If already focused on target, returns to Input.
    /// Otherwise, focuses on target.
    pub fn toggle(&mut self, target: FocusTarget) {
        if self.current == target {
            self.focus(FocusTarget::Input);
        } else {
            self.focus(target);
        }
    }

    /// Return focus to Input (common operation).
    pub fn return_to_input(&mut self) {
        self.close_all_overlays();
        self.current = FocusTarget::Input;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_focus() {
        let fm = FocusManager::new();
        assert_eq!(fm.current(), FocusTarget::Input);
        assert!(!fm.has_overlay());
    }

    #[test]
    fn test_direct_focus() {
        let mut fm = FocusManager::new();
        fm.focus(FocusTarget::Sidebar);
        assert_eq!(fm.current(), FocusTarget::Sidebar);
        assert!(!fm.has_overlay());
    }

    #[test]
    fn test_overlay_push_pop() {
        let mut fm = FocusManager::new();

        // Start at Input
        assert_eq!(fm.current(), FocusTarget::Input);

        // Push command palette overlay
        fm.push_overlay(FocusTarget::CommandPalette);
        assert_eq!(fm.current(), FocusTarget::CommandPalette);
        assert!(fm.has_overlay());

        // Pop returns to Input
        let closed = fm.pop_overlay();
        assert_eq!(closed, Some(FocusTarget::CommandPalette));
        assert_eq!(fm.current(), FocusTarget::Input);
        assert!(!fm.has_overlay());
    }

    #[test]
    fn test_overlay_returns_to_previous_panel() {
        let mut fm = FocusManager::new();

        // Focus sidebar first
        fm.focus(FocusTarget::Sidebar);
        assert_eq!(fm.current(), FocusTarget::Sidebar);

        // Open overlay
        fm.push_overlay(FocusTarget::Sessions);
        assert_eq!(fm.current(), FocusTarget::Sessions);

        // Pop should return to Sidebar
        fm.pop_overlay();
        assert_eq!(fm.current(), FocusTarget::Sidebar);
    }

    #[test]
    fn test_cycle_next() {
        let mut fm = FocusManager::new();

        // Input -> Sidebar -> ToolPane -> Logs -> Input
        assert_eq!(fm.current(), FocusTarget::Input);

        fm.cycle_next();
        assert_eq!(fm.current(), FocusTarget::Sidebar);

        fm.cycle_next();
        assert_eq!(fm.current(), FocusTarget::ToolPane);

        fm.cycle_next();
        assert_eq!(fm.current(), FocusTarget::Logs);

        fm.cycle_next();
        assert_eq!(fm.current(), FocusTarget::Input);
    }

    #[test]
    fn test_cycle_prev() {
        let mut fm = FocusManager::new();

        // Input -> Logs -> ToolPane -> Sidebar -> Input
        fm.cycle_prev();
        assert_eq!(fm.current(), FocusTarget::Logs);

        fm.cycle_prev();
        assert_eq!(fm.current(), FocusTarget::ToolPane);
    }

    #[test]
    fn test_cycle_blocked_during_overlay() {
        let mut fm = FocusManager::new();

        fm.push_overlay(FocusTarget::CommandPalette);

        // Cycling should do nothing while overlay is active
        fm.cycle_next();
        assert_eq!(fm.current(), FocusTarget::CommandPalette);
    }

    #[test]
    fn test_toggle() {
        let mut fm = FocusManager::new();

        // Toggle to ToolPane
        fm.toggle(FocusTarget::ToolPane);
        assert_eq!(fm.current(), FocusTarget::ToolPane);

        // Toggle again returns to Input
        fm.toggle(FocusTarget::ToolPane);
        assert_eq!(fm.current(), FocusTarget::Input);
    }

    #[test]
    fn test_is_overlay() {
        assert!(FocusTarget::CommandPalette.is_overlay());
        assert!(FocusTarget::Sessions.is_overlay());
        assert!(FocusTarget::Workstreams.is_overlay());
        assert!(!FocusTarget::Input.is_overlay());
        assert!(!FocusTarget::Sidebar.is_overlay());
    }

    #[test]
    fn test_close_all_overlays() {
        let mut fm = FocusManager::new();
        fm.focus(FocusTarget::Sidebar);
        fm.push_overlay(FocusTarget::CommandPalette);
        fm.push_overlay(FocusTarget::Sessions); // Nested overlay

        fm.close_all_overlays();
        assert_eq!(fm.current(), FocusTarget::Sidebar);
        assert!(!fm.has_overlay());
    }
}
