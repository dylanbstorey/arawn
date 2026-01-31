//! Tasks command - manage running and recent tasks.

use anyhow::Result;
use clap::{Args, Subcommand};

use super::Context;

/// Arguments for the tasks command.
#[derive(Args, Debug)]
pub struct TasksArgs {
    #[command(subcommand)]
    pub command: Option<TasksCommand>,
}

#[derive(Subcommand, Debug)]
pub enum TasksCommand {
    /// List all tasks (default)
    List,

    /// Show detailed status of a task
    Status {
        /// Task ID
        id: String,
    },

    /// Cancel a running task
    Cancel {
        /// Task ID
        id: String,
    },

    /// Show task result/output
    Result {
        /// Task ID
        id: String,
    },
}

/// Run the tasks command.
pub async fn run(args: TasksArgs, ctx: &Context) -> Result<()> {
    let cmd = args.command.unwrap_or(TasksCommand::List);

    match cmd {
        TasksCommand::List => {
            println!("Tasks");
            println!("-----");
            // TODO: Fetch tasks from server
            println!("No tasks (task listing not yet implemented)");
        }
        TasksCommand::Status { id } => {
            println!("Task Status: {}", id);
            // TODO: Fetch task status from server
            println!("Task status not yet implemented");
        }
        TasksCommand::Cancel { id } => {
            println!("Cancelling task: {}", id);
            // TODO: Cancel task via server
            println!("Task cancellation not yet implemented");
        }
        TasksCommand::Result { id } => {
            println!("Task Result: {}", id);
            // TODO: Fetch task result from server
            println!("Task results not yet implemented");
        }
    }

    if ctx.verbose {
        println!("\nServer: {}", ctx.server_url);
    }

    Ok(())
}
