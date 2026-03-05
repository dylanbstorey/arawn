//! Memory command - memory operations.

use anyhow::Result;
use clap::{Args, Subcommand};
use console::{Style, style};

use super::Context;
use super::output;
use crate::client::Client;

/// Arguments for the memory command.
#[derive(Args, Debug)]
#[command(after_help = "\x1b[1mExamples:\x1b[0m
  arawn memory search \"Rust ownership\"
  arawn memory search \"API design\" -l 5
  arawn memory recent
  arawn memory stats
  arawn memory export -o memories.json
  arawn memory reindex --dry-run")]
pub struct MemoryArgs {
    #[command(subcommand)]
    pub command: MemoryCommand,
}

#[derive(Subcommand, Debug)]
pub enum MemoryCommand {
    /// Semantic search through memories
    Search {
        /// Search query
        query: String,

        /// Maximum results to return
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Show recent memories
    Recent {
        /// Number of recent memories to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Show memory database statistics
    Stats,

    /// Re-embed all memories with the current configured provider
    Reindex {
        /// Show what would be done without doing it
        #[arg(long)]
        dry_run: bool,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Export memories to JSON
    Export {
        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,
    },
}

/// Run the memory command.
pub async fn run(args: MemoryArgs, ctx: &Context) -> Result<()> {
    match args.command {
        MemoryCommand::Search { query, limit } => cmd_search(&query, limit, ctx).await,
        MemoryCommand::Recent { limit } => cmd_recent(limit, ctx).await,
        MemoryCommand::Stats => cmd_stats(ctx).await,
        MemoryCommand::Reindex { dry_run, yes } => cmd_reindex(dry_run, yes, ctx).await,
        MemoryCommand::Export { output } => cmd_export(output, ctx).await,
    }
}

async fn cmd_search(query: &str, limit: usize, ctx: &Context) -> Result<()> {
    let client = Client::new(&ctx.server_url)?;
    let dim = Style::new().dim();

    if ctx.verbose {
        output::hint(format!("Searching: \"{}\" (limit: {})", query, limit));
        println!();
    }

    match client.memory_search(query, limit).await {
        Ok(results) => {
            if ctx.json_output {
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else if results.is_empty() {
                output::hint("No results found");
            } else {
                output::header("Memory Search Results");

                for (i, result) in results.iter().enumerate() {
                    let score_str = format!("(score: {:.3})", result.score);
                    println!(
                        "{}. {}",
                        style(i + 1).cyan(),
                        output::truncate(&result.content, 70)
                    );
                    println!("   {}", dim.apply_to(score_str));
                    println!();
                }
            }
        }
        Err(e) => {
            super::print_cli_error(&e, &ctx.server_url, ctx.verbose);
        }
    }

    Ok(())
}

async fn cmd_recent(limit: usize, ctx: &Context) -> Result<()> {
    let dim = Style::new().dim();
    let store = open_memory_store()?;

    let memories = store.list_memories(None, limit, 0)?;

    if ctx.json_output {
        println!("{}", serde_json::to_string_pretty(&memories)?);
    } else {
        output::header("Recent Memories");

        if memories.is_empty() {
            output::hint("No memories found");
        } else {
            for memory in &memories {
                let type_label = memory.content_type.as_str();
                let time = memory.created_at.format("%Y-%m-%d %H:%M");
                println!(
                    "{} {} {}",
                    dim.apply_to(format!("[{}]", type_label)),
                    output::truncate(&memory.content, 60),
                    dim.apply_to(format!("({})", time)),
                );
            }
            println!();
            output::hint(format!(
                "{} memor{}",
                memories.len(),
                if memories.len() == 1 { "y" } else { "ies" }
            ));
        }
    }

    Ok(())
}

async fn cmd_stats(_ctx: &Context) -> Result<()> {
    let store = open_memory_store()?;
    let stats = store.stats()?;

    output::header("Memory Statistics");
    println!("  Memories:    {}", style(stats.memory_count).cyan());
    println!("  Sessions:    {}", style(stats.session_count).cyan());
    println!("  Notes:       {}", style(stats.note_count).cyan());
    println!("  Embeddings:  {}", style(stats.embedding_count).cyan());
    println!("  Schema:      v{}", stats.schema_version);
    println!();

    output::header("Embedding Configuration");
    match &stats.embedding_provider {
        Some(provider) => println!("  Provider:    {}", style(provider).cyan()),
        None => output::hint("  Provider:    (not configured)"),
    }
    match stats.embedding_dimensions {
        Some(dims) => println!("  Dimensions:  {}", style(dims).cyan()),
        None => output::hint("  Dimensions:  (not configured)"),
    }
    if stats.vectors_stale {
        println!(
            "  Status:      {}",
            Style::new()
                .red()
                .apply_to("STALE — run `arawn memory reindex`")
        );
    } else {
        println!("  Status:      {}", Style::new().green().apply_to("ok"));
    }
    println!();

    Ok(())
}

async fn cmd_reindex(dry_run: bool, yes: bool, _ctx: &Context) -> Result<()> {
    let store = open_memory_store()?;

    if dry_run {
        let dry = store.reindex_dry_run()?;
        output::header("Reindex Dry Run");
        println!("  Memories to embed:  {}", style(dry.memory_count).cyan());
        println!(
            "  Estimated tokens:   {}",
            style(format!("~{}", dry.estimated_tokens)).cyan()
        );
        println!();
        return Ok(());
    }

    // Load config to build embedder
    let loaded = arawn_config::load_config(None)?;
    let config = loaded.config;
    let embedding_config = config.embedding.clone().unwrap_or_default();

    let embedder_spec = build_embedder_spec(&embedding_config);
    let embedder = arawn_llm::build_embedder(&embedder_spec)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to build embedder: {e}"))?;

    let new_dims = embedding_config.effective_dimensions();
    let new_provider = format!("{:?}", embedding_config.provider).to_lowercase();

    // Show what we're about to do
    let dry = store.reindex_dry_run()?;
    output::header("Memory Reindex");
    println!("  Provider:     {}", style(&new_provider).cyan());
    println!("  Dimensions:   {}", style(new_dims).cyan());
    println!("  Memories:     {}", style(dry.memory_count).cyan());
    println!(
        "  Est. tokens:  {}",
        style(format!("~{}", dry.estimated_tokens)).cyan()
    );
    println!();

    if dry.memory_count == 0 {
        output::hint("No memories to embed.");
        return Ok(());
    }

    // Confirmation
    if !yes {
        eprint!("Continue? [y/N] ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            output::hint("Aborted.");
            return Ok(());
        }
    }

    // Run reindex
    let embedder_ref = embedder.clone();
    let report = store
        .reindex(
            |texts| {
                let emb = embedder_ref.clone();
                async move {
                    let refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
                    emb.embed_batch(&refs).await.map_err(|e| e.to_string())
                }
            },
            new_dims,
            &new_provider,
        )
        .await?;

    println!();
    println!("{}", style("Reindex Complete").bold().green());
    println!("  Total:     {}", report.total);
    println!("  Embedded:  {}", style(report.embedded).green());
    if report.skipped > 0 {
        println!(
            "  Skipped:   {}",
            Style::new().yellow().apply_to(report.skipped)
        );
    }
    println!("  Elapsed:   {:.1?}", report.elapsed);

    Ok(())
}

async fn cmd_export(output: Option<String>, ctx: &Context) -> Result<()> {
    let store = open_memory_store()?;

    let memories = store.list_memories(None, usize::MAX, 0)?;
    let notes = store.list_notes(usize::MAX, 0)?;

    let export = serde_json::json!({
        "memories": memories,
        "notes": notes,
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "counts": {
            "memories": memories.len(),
            "notes": notes.len(),
        }
    });

    let json = serde_json::to_string_pretty(&export)?;

    match output {
        Some(path) => {
            std::fs::write(&path, &json)?;
            if !ctx.json_output {
                super::output::success(format!(
                    "Exported {} memories and {} notes to {}",
                    memories.len(),
                    notes.len(),
                    Style::new().dim().apply_to(&path),
                ));
            }
        }
        None => {
            println!("{}", json);
        }
    }

    Ok(())
}

/// Open the memory store at the default data directory.
fn open_memory_store() -> Result<arawn_memory::MemoryStore> {
    let data_dir = arawn_config::xdg_config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?;
    let db_path = data_dir.join("memory.db");
    let store = arawn_memory::MemoryStore::open(&db_path)?;
    Ok(store)
}

/// Build an EmbedderSpec from EmbeddingConfig (same logic as start.rs).
fn build_embedder_spec(config: &arawn_config::EmbeddingConfig) -> arawn_llm::EmbedderSpec {
    use arawn_config::EmbeddingProvider;

    let provider = match config.provider {
        EmbeddingProvider::Local => "local",
        EmbeddingProvider::OpenAi => "openai",
        EmbeddingProvider::Mock => "mock",
    };

    let openai_config = config.openai.as_ref();
    let local_config = config.local.as_ref();

    // Resolve OpenAI API key: config → env var
    let openai_api_key = openai_config
        .and_then(|c| c.api_key.clone())
        .or_else(|| std::env::var("OPENAI_API_KEY").ok());

    arawn_llm::EmbedderSpec {
        provider: provider.to_string(),
        openai_api_key,
        openai_model: openai_config.map(|c| c.model.clone()),
        openai_base_url: openai_config.and_then(|c| c.base_url.clone()),
        local_model_path: local_config.and_then(|c| c.model_path.clone()),
        local_tokenizer_path: local_config.and_then(|c| c.tokenizer_path.clone()),
        dimensions: config.dimensions,
        local_model_url: local_config.and_then(|c| c.model_url.clone()),
        local_tokenizer_url: local_config.and_then(|c| c.tokenizer_url.clone()),
    }
}
