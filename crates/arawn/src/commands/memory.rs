//! Memory command - memory operations.

use anyhow::Result;
use clap::{Args, Subcommand};
use console::{Style, style};

use super::Context;
use crate::client::Client;

/// Arguments for the memory command.
#[derive(Args, Debug)]
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
        println!(
            "{}",
            dim.apply_to(format!("Searching: \"{}\" (limit: {})", query, limit))
        );
        println!();
    }

    match client.memory_search(query, limit).await {
        Ok(results) => {
            if ctx.json_output {
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else if results.is_empty() {
                println!("{}", dim.apply_to("No results found"));
            } else {
                println!("{}", style("Memory Search Results").bold());
                println!("{}", dim.apply_to("─".repeat(50)));
                println!();

                for (i, result) in results.iter().enumerate() {
                    let score_str = format!("(score: {:.3})", result.score);
                    println!("{}. {}", style(i + 1).cyan(), truncate(&result.content, 70));
                    println!("   {}", dim.apply_to(score_str));
                    println!();
                }
            }
        }
        Err(e) => {
            let red = Style::new().red();
            eprintln!("{} {}", red.apply_to("Error:"), e);
        }
    }

    Ok(())
}

async fn cmd_recent(limit: usize, ctx: &Context) -> Result<()> {
    let dim = Style::new().dim();
    println!("{}", style("Recent Memories").bold());
    println!("{}", dim.apply_to("─".repeat(50)));
    println!();
    println!(
        "{}",
        dim.apply_to(format!("(limit: {} - not yet implemented)", limit))
    );
    let _ = ctx;
    Ok(())
}

async fn cmd_stats(_ctx: &Context) -> Result<()> {
    let dim = Style::new().dim();
    let store = open_memory_store()?;

    let stats = store.stats()?;

    println!("{}", style("Memory Statistics").bold());
    println!("{}", dim.apply_to("─".repeat(50)));
    println!();
    println!("  Memories:    {}", style(stats.memory_count).cyan());
    println!("  Sessions:    {}", style(stats.session_count).cyan());
    println!("  Notes:       {}", style(stats.note_count).cyan());
    println!("  Embeddings:  {}", style(stats.embedding_count).cyan());
    println!("  Schema:      v{}", stats.schema_version);
    println!();

    println!("{}", style("Embedding Configuration").bold());
    println!("{}", dim.apply_to("─".repeat(50)));
    println!();
    match &stats.embedding_provider {
        Some(provider) => println!("  Provider:    {}", style(provider).cyan()),
        None => println!("  Provider:    {}", dim.apply_to("(not configured)")),
    }
    match stats.embedding_dimensions {
        Some(dims) => println!("  Dimensions:  {}", style(dims).cyan()),
        None => println!("  Dimensions:  {}", dim.apply_to("(not configured)")),
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
    let dim = Style::new().dim();
    let store = open_memory_store()?;

    if dry_run {
        let dry = store.reindex_dry_run()?;
        println!("{}", style("Reindex Dry Run").bold());
        println!("{}", dim.apply_to("─".repeat(50)));
        println!();
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
    println!("{}", style("Memory Reindex").bold());
    println!("{}", dim.apply_to("─".repeat(50)));
    println!();
    println!("  Provider:     {}", style(&new_provider).cyan());
    println!("  Dimensions:   {}", style(new_dims).cyan());
    println!("  Memories:     {}", style(dry.memory_count).cyan());
    println!(
        "  Est. tokens:  {}",
        style(format!("~{}", dry.estimated_tokens)).cyan()
    );
    println!();

    if dry.memory_count == 0 {
        println!("{}", dim.apply_to("No memories to embed."));
        return Ok(());
    }

    // Confirmation
    if !yes {
        eprint!("Continue? [y/N] ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", dim.apply_to("Aborted."));
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

async fn cmd_export(output: Option<String>, _ctx: &Context) -> Result<()> {
    let dim = Style::new().dim();
    match output {
        Some(path) => println!(
            "{}",
            dim.apply_to(format!("Exporting to: {} (not yet implemented)", path))
        ),
        None => println!(
            "{}",
            dim.apply_to("Exporting to stdout (not yet implemented)")
        ),
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
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    let s = s.replace('\n', " ");
    if s.len() <= max_len {
        s
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
