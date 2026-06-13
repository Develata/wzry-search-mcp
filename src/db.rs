mod query;
mod rows;
mod schema;
mod write;

#[cfg(test)]
mod tests;

use anyhow::{Context, Result, anyhow};
use rusqlite::Connection;
use std::path::Path;

pub struct Store {
    pub(crate) conn: Connection,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path.as_ref())
            .with_context(|| format!("open sqlite db {}", path.as_ref().display()))?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    pub fn open_existing(path: impl AsRef<Path>) -> Result<Self> {
        if !path.as_ref().exists() {
            return Err(anyhow!(
                "database {} does not exist; run `wzry-search-mcp sync --db <path>` first",
                path.as_ref().display()
            ));
        }
        Self::open(path)
    }
}
