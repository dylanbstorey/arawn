//! Arawn TUI - Terminal User Interface
//!
//! A minimal, keyboard-driven terminal interface for Arawn.

pub mod app;
pub mod bounded;
pub mod client;
pub mod events;
pub mod focus;
pub mod input;
pub mod logs;
pub mod palette;
pub mod protocol;
pub mod sessions;
pub mod sidebar;
pub mod ui;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io::{self, Stdout};
use std::panic;
use tracing::Level;
use tracing_subscriber::prelude::*;

pub use app::App;
pub use logs::{LogBuffer, TuiLogLayer};

/// Terminal type alias for convenience.
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal for TUI mode.
pub fn init_terminal() -> Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore the terminal to normal mode.
pub fn restore_terminal(terminal: &mut Tui) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Install a panic hook that restores the terminal before panicking.
pub fn install_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Attempt to restore terminal state
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));
}

/// Configuration for running the TUI.
pub struct TuiConfig {
    /// Server URL to connect to.
    pub server_url: String,
    /// Workstream to use.
    pub workstream: Option<String>,
    /// Context name (for display purposes).
    pub context_name: Option<String>,
    /// Log buffer for capturing logs (optional, created if not provided).
    pub log_buffer: Option<LogBuffer>,
}

impl TuiConfig {
    /// Create config with just a server URL.
    pub fn new(server_url: impl Into<String>) -> Self {
        Self {
            server_url: server_url.into(),
            workstream: None,
            context_name: None,
            log_buffer: None,
        }
    }

    /// Load config from the client config file.
    ///
    /// Uses the specified context or falls back to current context.
    pub fn from_client_config(context_name: Option<&str>) -> Result<Self> {
        let config = arawn_config::load_client_config()?;

        let context = if let Some(name) = context_name {
            config
                .get_context(name)
                .ok_or_else(|| anyhow::anyhow!("Context '{}' not found", name))?
        } else {
            config
                .current()
                .ok_or_else(|| anyhow::anyhow!("No current context. Set one with: arawn config use-context <name>"))?
        };

        Ok(Self {
            server_url: context.server.clone(),
            workstream: context
                .workstream
                .clone()
                .or_else(|| Some(config.defaults.workstream.clone())),
            context_name: Some(context.name.clone()),
            log_buffer: None,
        })
    }
}

/// Run the TUI application.
pub async fn run(server_url: &str) -> Result<()> {
    run_with_config(TuiConfig::new(server_url)).await
}

/// Run the TUI application with full configuration.
pub async fn run_with_config(config: TuiConfig) -> Result<()> {
    install_panic_hook();

    // Set up TUI-specific logging that captures to a buffer
    let log_buffer = config.log_buffer.unwrap_or_default();
    let tui_layer = TuiLogLayer::new(log_buffer.clone()).with_min_level(Level::DEBUG);

    // Initialize tracing with TUI log layer
    tracing_subscriber::registry().with(tui_layer).init();

    let mut terminal = init_terminal()?;
    let mut app = App::new(config.server_url, log_buffer)?;

    // Apply workstream from config
    if let Some(ws) = config.workstream {
        app.workstream = ws;
    }

    // Store context name for display
    if let Some(ctx) = config.context_name {
        app.context_name = Some(ctx);
    }

    let result = app.run(&mut terminal).await;

    restore_terminal(&mut terminal)?;

    result
}
