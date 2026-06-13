use super::Store;
use anyhow::Result;

impl Store {
    pub fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS heroes (
              hero_id INTEGER PRIMARY KEY,
              ename INTEGER NOT NULL UNIQUE,
              cname TEXT NOT NULL,
              id_name TEXT,
              title TEXT,
              hero_type INTEGER,
              roles_json TEXT NOT NULL,
              moss_id INTEGER,
              source_url TEXT NOT NULL,
              fetched_at TEXT NOT NULL,
              content_hash TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_heroes_cname ON heroes(cname);
            CREATE INDEX IF NOT EXISTS idx_heroes_id_name ON heroes(id_name);

            CREATE TABLE IF NOT EXISTS hero_skills (
              hero_id INTEGER NOT NULL,
              slot TEXT NOT NULL,
              name TEXT NOT NULL,
              cooldown TEXT,
              cost TEXT,
              description TEXT NOT NULL,
              source_url TEXT NOT NULL,
              fetched_at TEXT NOT NULL,
              content_hash TEXT NOT NULL,
              PRIMARY KEY(hero_id, slot),
              FOREIGN KEY(hero_id) REFERENCES heroes(hero_id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS hero_parse_warnings (
              hero_id INTEGER NOT NULL,
              warning TEXT NOT NULL,
              fetched_at TEXT NOT NULL,
              FOREIGN KEY(hero_id) REFERENCES heroes(hero_id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS items (
              item_id INTEGER PRIMARY KEY,
              item_name TEXT NOT NULL,
              item_type INTEGER,
              price INTEGER,
              total_price INTEGER,
              description_html TEXT,
              description_text TEXT,
              source_url TEXT NOT NULL,
              fetched_at TEXT NOT NULL,
              content_hash TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_items_name ON items(item_name);

            CREATE TABLE IF NOT EXISTS summoner_skills (
              skill_id INTEGER PRIMARY KEY,
              name TEXT NOT NULL,
              rank INTEGER,
              description TEXT,
              source_url TEXT NOT NULL,
              fetched_at TEXT NOT NULL,
              content_hash TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_summoner_name ON summoner_skills(name);

            CREATE TABLE IF NOT EXISTS source_snapshots (
              source_key TEXT PRIMARY KEY,
              url TEXT NOT NULL,
              fetched_at TEXT NOT NULL,
              content_hash TEXT NOT NULL,
              byte_len INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS update_events (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              event_type TEXT NOT NULL,
              source_key TEXT,
              message TEXT NOT NULL,
              created_at TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }
}
