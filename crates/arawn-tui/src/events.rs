//! Event handling for the TUI.

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use futures::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing;

/// Terminal events.
#[derive(Debug, Clone)]
pub enum Event {
    /// Keyboard input.
    Key(KeyEvent),
    /// Terminal resize.
    Resize(u16, u16),
    /// Tick for periodic updates.
    Tick,
}

/// Handles terminal events using crossterm's async event stream.
pub struct EventHandler {
    /// Event receiver channel.
    rx: mpsc::UnboundedReceiver<Event>,
    /// Task handle for the event loop (kept alive).
    #[allow(dead_code)]
    task: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    /// Create a new event handler.
    pub fn new() -> Self {
        let tick_rate = Duration::from_millis(100);
        let (tx, rx) = mpsc::unbounded_channel();

        let task = tokio::spawn(async move {
            let mut reader = event::EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_rate);

            loop {
                tokio::select! {
                    // Handle terminal events
                    maybe_event = reader.next() => {
                        match maybe_event {
                            Some(Ok(evt)) => {
                                let event = match evt {
                                    CrosstermEvent::Key(key) => Some(Event::Key(key)),
                                    CrosstermEvent::Resize(w, h) => Some(Event::Resize(w, h)),
                                    _ => None,
                                };
                                if let Some(e) = event
                                    && tx.send(e).is_err() {
                                        tracing::debug!("Event channel closed, receiver dropped");
                                        break;
                                    }
                            }
                            Some(Err(e)) => {
                                tracing::warn!(error = %e, "Error reading terminal event, stopping event loop");
                                break;
                            }
                            None => {
                                tracing::debug!("Event stream ended, stopping event loop");
                                break;
                            }
                        }
                    }
                    // Handle tick events
                    _ = tick_interval.tick() => {
                        if tx.send(Event::Tick).is_err() {
                            tracing::debug!("Event channel closed during tick, stopping event loop");
                            break;
                        }
                    }
                }
            }
        });

        Self { rx, task }
    }

    /// Wait for the next event.
    pub async fn next(&mut self) -> Result<Event> {
        self.rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Event channel closed"))
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
