mod crawler;
mod db;
mod export;
mod mcp;
mod model;
mod parser;
mod util;

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use crawler::{CrawlConfig, Crawler};
use db::Store;
use model::SyncUpdateResult;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

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
    /// One-shot sync update: check deterministic sources, run news incremental sync, and optionally fall back to full sync.
    SyncUpdate {
        /// Maximum update-like news articles to inspect.
        #[arg(long, default_value_t = 10)]
        news_limit: usize,
        /// Analyze and print planned work without refreshing detail pages or full-syncing.
        #[arg(long)]
        dry_run: bool,
        /// If deterministic list-source hashes changed, run a polite full sync after incremental analysis.
        #[arg(long)]
        fallback_full: bool,
        /// Print the full machine-readable result instead of a compact human summary.
        #[arg(long)]
        json: bool,
        /// Disable random polite delay between affected hero/detail requests.
        #[arg(long)]
        no_polite: bool,
        /// Disable the sync-update lock. Not recommended for cron/scheduled runs.
        #[arg(long)]
        no_lock: bool,
        /// Override the lock file path. Defaults to <db>.sync-update.lock.
        #[arg(long)]
        lock_file: Option<PathBuf>,
        /// Wait this many milliseconds for an existing sync-update lock before failing.
        #[arg(long, default_value_t = 0)]
        lock_timeout_ms: u64,
        #[arg(long, default_value_t = 3000)]
        min_delay_ms: u64,
        #[arg(long, default_value_t = 12000)]
        max_delay_ms: u64,
    },
    /// Analyze official update-like news and refresh affected hero detail pages.
    SyncChanged {
        /// Maximum update-like news articles to inspect.
        #[arg(long, default_value_t = 10)]
        news_limit: usize,
        /// Analyze and print affected heroes without refreshing detail pages.
        #[arg(long)]
        dry_run: bool,
        /// Disable random polite delay between affected hero detail requests.
        #[arg(long)]
        no_polite: bool,
        #[arg(long, default_value_t = 3000)]
        min_delay_ms: u64,
        #[arg(long, default_value_t = 12000)]
        max_delay_ms: u64,
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
    /// List heroes for discovery.
    ListHeroes {
        #[arg(long, default_value_t = 200)]
        limit: usize,
    },
    /// Search hero skills across all heroes.
    SearchSkills {
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
    /// List items for discovery.
    ListItems {
        #[arg(long, default_value_t = 200)]
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
    /// Export local dataset. JSON writes one file; CSV writes a directory of CSV files.
    Export {
        #[arg(long)]
        format: String,
        #[arg(long)]
        out: PathBuf,
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
        Commands::SyncUpdate {
            news_limit,
            dry_run,
            fallback_full,
            json,
            no_polite,
            no_lock,
            lock_file,
            lock_timeout_ms,
            min_delay_ms,
            max_delay_ms,
        } => {
            let lock_path = lock_file.unwrap_or_else(|| default_sync_update_lock_path(&cli.db));
            let _lock = if no_lock {
                None
            } else {
                Some(SyncUpdateLock::acquire(lock_path.clone(), lock_timeout_ms)?)
            };

            let mut store = Store::open_existing(&cli.db)?;
            let crawler = Crawler::new(CrawlConfig {
                min_delay_ms,
                max_delay_ms,
                ..Default::default()
            })?;

            let source_status = crawler.check_updates(&store)?;

            let news_incremental =
                crawler.sync_changed_from_news(&mut store, news_limit, dry_run, !no_polite)?;

            let mut full_sync_ran = false;
            let mut full_sync_reason = None;
            let mut warnings = news_incremental.warnings.clone();

            if source_status.changed && dry_run {
                warnings.push(
                    "deterministic source hash changed; dry-run did not refresh the full dataset"
                        .to_string(),
                );
            } else if source_status.changed && fallback_full {
                crawler.sync_all(&mut store, !no_polite, None)?;
                full_sync_ran = true;
                full_sync_reason = Some("deterministic source hash changed".to_string());
            } else if source_status.changed {
                warnings.push(
                    "deterministic source hash changed; snapshots were not advanced because full sync did not run; run `sync` or `sync-update --fallback-full` to refresh the full dataset"
                        .to_string(),
                );
            } else if !dry_run {
                crawler.write_update_snapshots(&store, &source_status)?;
            }

            let status = if dry_run {
                "dry_run"
            } else if full_sync_ran {
                "full_synced"
            } else if !news_incremental.synced_heroes.is_empty() {
                "updated"
            } else if source_status.changed {
                "source_changed"
            } else {
                "unchanged"
            }
            .to_string();

            let result = SyncUpdateResult {
                status,
                dry_run,
                lock_file: if no_lock {
                    None
                } else {
                    Some(lock_path.display().to_string())
                },
                source_status,
                news_incremental,
                full_sync_ran,
                full_sync_reason,
                warnings,
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                print_sync_update_summary(&result);
            }
        }
        Commands::SyncChanged {
            news_limit,
            dry_run,
            no_polite,
            min_delay_ms,
            max_delay_ms,
        } => {
            let mut store = Store::open_existing(&cli.db)?;
            let crawler = Crawler::new(CrawlConfig {
                min_delay_ms,
                max_delay_ms,
                ..Default::default()
            })?;
            let result =
                crawler.sync_changed_from_news(&mut store, news_limit, dry_run, !no_polite)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
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
        Commands::ListHeroes { limit } => {
            let store = Store::open_existing(&cli.db)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&store.list_heroes(limit)?)?
            );
        }
        Commands::SearchSkills { query, limit } => {
            let store = Store::open_existing(&cli.db)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&store.search_hero_skills(&query, limit)?)?
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
        Commands::ListItems { limit } => {
            let store = Store::open_existing(&cli.db)?;
            let mut items = store.all_items()?;
            items.truncate(limit);
            println!("{}", serde_json::to_string_pretty(&items)?);
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
        Commands::Export { format, out } => {
            let store = Store::open_existing(&cli.db)?;
            let format = export::ExportFormat::parse(&format)?;
            export::export_store(&store, format, &out)?;
            println!("exported {:?} to {}", format, out.display());
        }
        Commands::Serve => mcp::serve_stdio(&db_path)?,
    }
    Ok(())
}

fn default_sync_update_lock_path(db_path: &Path) -> PathBuf {
    let mut lock_path = db_path.to_path_buf();
    let extension = db_path
        .extension()
        .map(|ext| format!("{}.sync-update.lock", ext.to_string_lossy()))
        .unwrap_or_else(|| "sync-update.lock".to_string());
    lock_path.set_extension(extension);
    lock_path
}

struct SyncUpdateLock {
    path: PathBuf,
    _file: File,
}

impl SyncUpdateLock {
    fn acquire(path: PathBuf, timeout_ms: u64) -> Result<Self> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)
                .with_context(|| format!("create lock directory {}", parent.display()))?;
        }

        let started = Instant::now();
        loop {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    writeln!(file, "pid={}", std::process::id())?;
                    return Ok(Self { path, _file: file });
                }
                Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                    if started.elapsed() >= Duration::from_millis(timeout_ms) {
                        return Err(anyhow!(
                            "sync-update lock already exists: {}; remove it only after confirming no sync-update process is running, or pass --no-lock for a controlled manual run",
                            path.display()
                        ));
                    }
                    thread::sleep(Duration::from_millis(500));
                }
                Err(err) => {
                    return Err(err)
                        .with_context(|| format!("create sync-update lock {}", path.display()));
                }
            }
        }
    }
}

impl Drop for SyncUpdateLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn print_sync_update_summary(result: &SyncUpdateResult) {
    println!("sync-update status: {}", result.status);
    println!("dry_run: {}", result.dry_run);
    if let Some(lock_file) = &result.lock_file {
        println!("lock_file: {lock_file}");
    }
    println!(
        "deterministic_sources_changed: {}",
        result.source_status.changed
    );
    println!(
        "news_articles_checked: {}",
        result.news_incremental.checked_articles
    );
    println!(
        "affected_heroes: {}",
        result.news_incremental.affected_heroes.len()
    );
    println!(
        "synced_heroes: {}",
        result.news_incremental.synced_heroes.len()
    );
    println!("full_sync_ran: {}", result.full_sync_ran);
    if let Some(reason) = &result.full_sync_reason {
        println!("full_sync_reason: {reason}");
    }
    if !result.warnings.is_empty() {
        println!("warnings:");
        for warning in &result.warnings {
            println!("- {warning}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_sync_update_lock_path_appends_sync_update_lock() {
        assert_eq!(
            default_sync_update_lock_path(Path::new("/tmp/wzry.sqlite")),
            PathBuf::from("/tmp/wzry.sqlite.sync-update.lock")
        );
        assert_eq!(
            default_sync_update_lock_path(Path::new("/tmp/wzry")),
            PathBuf::from("/tmp/wzry.sync-update.lock")
        );
    }

    #[test]
    fn sync_update_lock_fails_when_existing_and_cleans_up_on_drop() {
        let dir = tempfile::tempdir().expect("tempdir");
        let lock_path = dir.path().join("wzry.sqlite.sync-update.lock");

        let first = SyncUpdateLock::acquire(lock_path.clone(), 0).expect("first lock");
        assert!(lock_path.exists());
        assert!(SyncUpdateLock::acquire(lock_path.clone(), 0).is_err());

        drop(first);
        assert!(!lock_path.exists());

        let second = SyncUpdateLock::acquire(lock_path.clone(), 0).expect("second lock");
        drop(second);
        assert!(!lock_path.exists());
    }
}
