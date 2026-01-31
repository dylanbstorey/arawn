//! REPL (Read-Eval-Print Loop) implementation for interactive chat.

use anyhow::Result;
use console::{Style, Term, style};
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{Config, Editor};
use std::io::Write;

use crate::client::{ChatEvent, Client};

/// REPL state and configuration.
pub struct Repl {
    client: Client,
    session_id: Option<String>,
    editor: Editor<(), DefaultHistory>,
    term: Term,
    verbose: bool,
}

impl Repl {
    /// Create a new REPL instance.
    pub fn new(client: Client, session_id: Option<String>, verbose: bool) -> Result<Self> {
        let config = Config::builder()
            .history_ignore_space(true)
            .auto_add_history(true)
            .build();

        let editor = Editor::with_config(config)?;

        Ok(Self {
            client,
            session_id,
            editor,
            term: Term::stdout(),
            verbose,
        })
    }

    /// Run the REPL loop.
    pub async fn run(&mut self) -> Result<()> {
        self.print_welcome();

        loop {
            let prompt = self.format_prompt();

            match self.editor.readline(&prompt) {
                Ok(line) => {
                    let line = line.trim();

                    if line.is_empty() {
                        continue;
                    }

                    // Handle slash commands
                    if line.starts_with('/') {
                        match self.handle_slash_command(line).await {
                            Ok(ControlFlow::Continue) => continue,
                            Ok(ControlFlow::Exit) => break,
                            Err(e) => {
                                self.print_error(&format!("Command error: {}", e));
                                continue;
                            }
                        }
                    }

                    // Send as chat message
                    if let Err(e) = self.send_message(line).await {
                        self.print_error(&format!("Error: {}", e));
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl+C - cancel current operation but don't exit
                    println!();
                    self.print_dim("(Interrupted - type /quit to exit)");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl+D - exit
                    println!();
                    break;
                }
                Err(e) => {
                    self.print_error(&format!("Input error: {}", e));
                    break;
                }
            }
        }

        self.print_dim("Goodbye!");
        Ok(())
    }

    /// Send a message and stream the response.
    async fn send_message(&mut self, message: &str) -> Result<()> {
        let mut stream = self
            .client
            .chat_stream(message, self.session_id.as_deref())
            .await?;

        // Print response with streaming
        while let Some(event) = stream.next_event().await {
            match event? {
                ChatEvent::Text(text) => {
                    print!("{}", text);
                    std::io::stdout().flush()?;
                }
                ChatEvent::ToolStart { name, .. } => {
                    println!();
                    self.print_tool_start(&name);
                }
                ChatEvent::ToolEnd { success, .. } => {
                    self.print_tool_end(success);
                }
                ChatEvent::Done => {
                    println!();
                    println!();
                }
                ChatEvent::Error(e) => {
                    self.print_error(&e);
                }
            }
        }

        Ok(())
    }

    /// Handle a slash command.
    async fn handle_slash_command(&mut self, input: &str) -> Result<ControlFlow> {
        let parts: Vec<&str> = input[1..].split_whitespace().collect();
        let cmd = parts.first().copied().unwrap_or("");
        let args = &parts[1..];

        match cmd {
            "quit" | "q" | "exit" => {
                return Ok(ControlFlow::Exit);
            }
            "help" | "h" | "?" => {
                self.print_help();
            }
            "clear" | "cls" => {
                self.term.clear_screen()?;
            }
            "status" => {
                self.print_status().await?;
            }
            "new" => {
                self.session_id = None;
                self.print_dim("Started new session");
            }
            "session" => {
                if let Some(ref id) = self.session_id {
                    println!("Current session: {}", id);
                } else {
                    self.print_dim("No active session (will create on first message)");
                }
            }
            "memory" if args.len() >= 2 && args[0] == "search" => {
                let query = args[1..].join(" ");
                self.search_memory(&query).await?;
            }
            "note" if !args.is_empty() => {
                let content = args.join(" ");
                self.add_note(&content).await?;
            }
            "" => {
                self.print_dim("Type /help for available commands");
            }
            _ => {
                self.print_error(&format!("Unknown command: /{}", cmd));
                self.print_dim("Type /help for available commands");
            }
        }

        Ok(ControlFlow::Continue)
    }

    fn print_welcome(&self) {
        let dim = Style::new().dim();
        println!();
        println!("{}", style("Arawn Chat").bold().cyan());
        println!("{}", dim.apply_to("─".repeat(40)));
        println!(
            "{}",
            dim.apply_to("Type your message and press Enter to chat.")
        );
        println!(
            "{}",
            dim.apply_to("Use /help for commands, Ctrl+D to exit.")
        );
        println!();
    }

    fn print_help(&self) {
        let dim = Style::new().dim();
        println!();
        println!("{}", style("Available Commands").bold());
        println!("{}", dim.apply_to("─".repeat(40)));
        println!("  {}  - Exit the REPL", style("/quit, /q").cyan());
        println!("  {}  - Show this help", style("/help, /h, /?").cyan());
        println!("  {}  - Clear the screen", style("/clear").cyan());
        println!("  {}  - Show server status", style("/status").cyan());
        println!("  {}  - Start a new session", style("/new").cyan());
        println!("  {}  - Show current session ID", style("/session").cyan());
        println!(
            "  {}  - Search memories",
            style("/memory search <query>").cyan()
        );
        println!("  {}  - Add a quick note", style("/note <text>").cyan());
        println!();
        println!("{}", dim.apply_to("Keyboard shortcuts:"));
        println!("  {} - Interrupt current operation", dim.apply_to("Ctrl+C"));
        println!("  {} - Exit the REPL", dim.apply_to("Ctrl+D"));
        println!();
    }

    async fn print_status(&self) -> Result<()> {
        let dim = Style::new().dim();
        match self.client.health().await {
            Ok(health) => {
                let green = Style::new().green();
                println!(
                    "Server: {} (v{})",
                    green.apply_to("● running"),
                    health.version
                );
            }
            Err(e) => {
                let red = Style::new().red();
                println!("Server: {}", red.apply_to("● not running"));
                if self.verbose {
                    println!("  {}", dim.apply_to(format!("Error: {}", e)));
                }
            }
        }
        Ok(())
    }

    async fn search_memory(&self, query: &str) -> Result<()> {
        let dim = Style::new().dim();
        println!("Searching: {}", dim.apply_to(query));

        match self.client.memory_search(query, 5).await {
            Ok(results) => {
                if results.is_empty() {
                    self.print_dim("No results found");
                } else {
                    for (i, result) in results.iter().enumerate() {
                        println!(
                            "{}. {} {}",
                            i + 1,
                            result.content,
                            dim.apply_to(format!("(score: {:.3})", result.score))
                        );
                    }
                }
            }
            Err(e) => {
                self.print_error(&format!("Search failed: {}", e));
            }
        }
        Ok(())
    }

    async fn add_note(&self, content: &str) -> Result<()> {
        match self.client.create_note(content).await {
            Ok(note) => {
                let green = Style::new().green();
                println!("{} Note saved: {}", green.apply_to("✓"), note.id);
            }
            Err(e) => {
                self.print_error(&format!("Failed to save note: {}", e));
            }
        }
        Ok(())
    }

    fn format_prompt(&self) -> String {
        format!("{} ", style("arawn>").cyan().bold())
    }

    fn print_dim(&self, msg: &str) {
        let dim = Style::new().dim();
        println!("{}", dim.apply_to(msg));
    }

    fn print_error(&self, msg: &str) {
        let red = Style::new().red();
        println!("{} {}", red.apply_to("Error:"), msg);
    }

    fn print_tool_start(&self, name: &str) {
        let dim = Style::new().dim();
        println!("{}", dim.apply_to(format!("[Running: {}]", name)));
    }

    fn print_tool_end(&self, success: bool) {
        let dim = Style::new().dim();
        let status = if success { "done" } else { "failed" };
        println!("{}", dim.apply_to(format!("[{}]", status)));
    }
}

/// Control flow for the REPL.
pub enum ControlFlow {
    Continue,
    Exit,
}
