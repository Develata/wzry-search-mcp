mod crawler;
mod db;
mod mcp;
mod model;
mod util;

use anyhow::Result;
use clap::{Parser, Subcommand};
use crawler::{CrawlConfig, Crawler};
use db::Store;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "wzry-search-mcp",
    version,
    about = "王者荣耀 local factual retrieval MCP"
)]
struct Cli {
    #[arg(long, default_value = "./wzry.sqlite", global = true)]
    db: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Sync official hero/item/summoner sources into local SQLite.
    Sync {
        /// Disable random polite delay between hero detail requests.
        #[arg(long)]
        no_polite: bool,
        /// Limit hero detail sync count; useful for smoke tests.
        #[arg(long)]
        limit_heroes: Option<usize>,
        #[arg(long, default_value_t = 3000)]
        min_delay_ms: u64,
        #[arg(long, default_value_t = 12000)]
        max_delay_ms: u64,
    },
    /// Check source hashes without writing by default.
    CheckUpdates {
        /// Persist snapshots and update_events after checking.
        #[arg(long)]
        write_snapshots: bool,
    },
    /// Query a complete hero profile: basic metadata + passive/skills.
    Hero { hero: String },
    /// Query one hero skill.
    Skill { hero: String, skill: String },
    /// Search heroes.
    SearchHeroes {
        query: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Query one item.
    Item { item: String },
    /// Search items.
    SearchItems {
        query: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Query summoner skill or list all when no name is supplied.
    Summoner { skill: Option<String> },
    /// Build lineup evidence context. Comma-separated hero names.
    LineupContext {
        #[arg(long, value_delimiter = ',')]
        allies: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        enemies: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        candidates: Vec<String>,
    },
    /// Serve MCP over stdio.
    Serve,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    let db_path = cli.db.to_string_lossy().to_string();
    match cli.command {
        Commands::Sync {
            no_polite,
            limit_heroes,
            min_delay_ms,
            max_delay_ms,
        } => {
            let mut store = Store::open(&cli.db)?;
            let crawler = Crawler::new(CrawlConfig {
                min_delay_ms,
                max_delay_ms,
                ..Default::default()
            })?;
            crawler.sync_all(&mut store, !no_polite, limit_heroes)?;
            println!("sync complete: {}", cli.db.display());
        }
        Commands::CheckUpdates { write_snapshots } => {
            let store = Store::open(&cli.db)?;
            let crawler = Crawler::new(CrawlConfig::default())?;
            let status = crawler.check_updates(&store)?;
            if write_snapshots {
                crawler.write_update_snapshots(&store, &status)?;
            }
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
        Commands::Hero { hero } => {
            let store = Store::open_existing(&cli.db)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&store.get_hero_profile(&hero)?)?
            );
        }
        Commands::Skill { hero, skill } => {
            let store = Store::open_existing(&cli.db)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&store.get_hero_skill(&hero, &skill)?)?
            );
        }
        Commands::SearchHeroes { query, limit } => {
            let store = Store::open_existing(&cli.db)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&store.search_heroes(&query, limit)?)?
            );
        }
        Commands::Item { item } => {
            let store = Store::open_existing(&cli.db)?;
            println!("{}", serde_json::to_string_pretty(&store.get_item(&item)?)?);
        }
        Commands::SearchItems { query, limit } => {
            let store = Store::open_existing(&cli.db)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&store.search_items(&query, limit)?)?
            );
        }
        Commands::Summoner { skill } => {
            let store = Store::open_existing(&cli.db)?;
            if let Some(skill) = skill {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&store.get_summoner_skill(&skill)?)?
                );
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&store.get_summoner_skills()?)?
                );
            }
        }
        Commands::LineupContext {
            allies,
            enemies,
            candidates,
        } => {
            let store = Store::open_existing(&cli.db)?;
            let ctx = model::LineupContext {
                allies: allies
                    .iter()
                    .map(|h| store.get_hero_profile(h))
                    .collect::<Result<Vec<_>>>()?,
                enemies: enemies
                    .iter()
                    .map(|h| store.get_hero_profile(h))
                    .collect::<Result<Vec<_>>>()?,
                candidate_pool: candidates
                    .iter()
                    .map(|h| store.get_hero_profile(h))
                    .collect::<Result<Vec<_>>>()?,
                recommendation_should_be_done_by_model: true,
            };
            println!("{}", serde_json::to_string_pretty(&ctx)?);
        }
        Commands::Serve => mcp::serve_stdio(&db_path)?,
    }
    Ok(())
}
