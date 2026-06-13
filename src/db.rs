use crate::model::*;
use crate::util::now_rfc3339;
use anyhow::{Context, Result, anyhow};
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

pub struct Store {
    conn: Connection,
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

    pub fn upsert_hero(&self, hero: &HeroBasic) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO heroes
            (hero_id, ename, cname, id_name, title, hero_type, roles_json, moss_id, source_url, fetched_at, content_hash)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(hero_id) DO UPDATE SET
              ename=excluded.ename, cname=excluded.cname, id_name=excluded.id_name,
              title=excluded.title, hero_type=excluded.hero_type, roles_json=excluded.roles_json,
              moss_id=excluded.moss_id, source_url=excluded.source_url,
              fetched_at=excluded.fetched_at, content_hash=excluded.content_hash"#,
            params![
                hero.hero_id,
                hero.ename,
                hero.cname,
                hero.id_name,
                hero.title,
                hero.hero_type,
                serde_json::to_string(&hero.roles)?,
                hero.moss_id,
                hero.source.url,
                hero.source.fetched_at,
                hero.source.content_hash
            ],
        )?;
        Ok(())
    }

    pub fn replace_hero_skills(
        &mut self,
        hero_id: i64,
        skills: &[HeroSkill],
        warnings: &[String],
    ) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "DELETE FROM hero_skills WHERE hero_id = ?1",
            params![hero_id],
        )?;
        for skill in skills {
            tx.execute(
                r#"INSERT INTO hero_skills
                (hero_id, slot, name, cooldown, cost, description, source_url, fetched_at, content_hash)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
                params![
                    skill.hero_id,
                    skill.slot,
                    skill.name,
                    skill.cooldown,
                    skill.cost,
                    skill.description,
                    skill.source.url,
                    skill.source.fetched_at,
                    skill.source.content_hash
                ],
            )?;
        }
        tx.execute(
            "DELETE FROM hero_parse_warnings WHERE hero_id = ?1",
            params![hero_id],
        )?;
        for warning in warnings {
            tx.execute(
                "INSERT INTO hero_parse_warnings (hero_id, warning, fetched_at) VALUES (?1, ?2, ?3)",
                params![hero_id, warning, now_rfc3339()],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn upsert_item(&self, item: &Item) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO items
            (item_id, item_name, item_type, price, total_price, description_html, description_text, source_url, fetched_at, content_hash)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(item_id) DO UPDATE SET
              item_name=excluded.item_name, item_type=excluded.item_type, price=excluded.price,
              total_price=excluded.total_price, description_html=excluded.description_html,
              description_text=excluded.description_text, source_url=excluded.source_url,
              fetched_at=excluded.fetched_at, content_hash=excluded.content_hash"#,
            params![
                item.item_id,
                item.item_name,
                item.item_type,
                item.price,
                item.total_price,
                item.description_html,
                item.description_text,
                item.source.url,
                item.source.fetched_at,
                item.source.content_hash
            ],
        )?;
        Ok(())
    }

    pub fn upsert_summoner_skill(&self, skill: &SummonerSkill) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO summoner_skills
            (skill_id, name, rank, description, source_url, fetched_at, content_hash)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(skill_id) DO UPDATE SET
              name=excluded.name, rank=excluded.rank, description=excluded.description,
              source_url=excluded.source_url, fetched_at=excluded.fetched_at,
              content_hash=excluded.content_hash"#,
            params![
                skill.skill_id,
                skill.name,
                skill.rank,
                skill.description,
                skill.source.url,
                skill.source.fetched_at,
                skill.source.content_hash
            ],
        )?;
        Ok(())
    }

    pub fn get_snapshot_hash(&self, key: &str) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT content_hash FROM source_snapshots WHERE source_key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn upsert_snapshot(&self, snap: &SourceSnapshot) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO source_snapshots (source_key, url, fetched_at, content_hash, byte_len)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(source_key) DO UPDATE SET
              url=excluded.url, fetched_at=excluded.fetched_at,
              content_hash=excluded.content_hash, byte_len=excluded.byte_len"#,
            params![
                snap.source_key,
                snap.url,
                snap.fetched_at,
                snap.content_hash,
                snap.byte_len
            ],
        )?;
        Ok(())
    }

    pub fn add_update_event(
        &self,
        event_type: &str,
        source_key: Option<&str>,
        message: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO update_events (event_type, source_key, message, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![event_type, source_key, message, now_rfc3339()],
        )?;
        Ok(())
    }

    pub fn search_heroes(&self, query: &str, limit: usize) -> Result<Vec<HeroBasic>> {
        let pat = format!("%{}%", query.trim());
        let mut stmt = self.conn.prepare(
            r#"SELECT hero_id, ename, cname, id_name, title, hero_type, roles_json, moss_id, source_url, fetched_at, content_hash
            FROM heroes
            WHERE cname LIKE ?1 OR id_name LIKE ?1 OR title LIKE ?1
            ORDER BY CASE WHEN cname = ?2 THEN 0 WHEN cname LIKE ?1 THEN 1 ELSE 2 END, hero_id
            LIMIT ?3"#,
        )?;
        let rows = stmt.query_map(params![pat, query.trim(), limit as i64], Self::row_hero)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn resolve_hero(&self, query: &str) -> Result<HeroBasic> {
        let maybe_id = query.trim().parse::<i64>().ok();
        let maybe_hero = maybe_id
            .map(|id| self.get_hero_by_id(id))
            .transpose()?
            .flatten();
        if let Some(hero) = maybe_hero {
            return Ok(hero);
        }
        let exact: Option<HeroBasic> = self.conn.query_row(
            r#"SELECT hero_id, ename, cname, id_name, title, hero_type, roles_json, moss_id, source_url, fetched_at, content_hash
            FROM heroes WHERE cname = ?1 OR id_name = ?1 LIMIT 1"#,
            params![query.trim()],
            Self::row_hero,
        ).optional()?;
        if let Some(hero) = exact {
            return Ok(hero);
        }
        let matches = self.search_heroes(query, 8)?;
        match matches.len() {
            0 => Err(anyhow!("hero not found: {query}")),
            1 => Ok(matches.into_iter().next().unwrap()),
            _ => Err(anyhow!(
                "ambiguous hero `{}`; candidates: {}",
                query,
                matches
                    .iter()
                    .map(|h| h.cname.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }

    pub fn get_hero_by_id(&self, hero_id: i64) -> Result<Option<HeroBasic>> {
        self.conn.query_row(
            r#"SELECT hero_id, ename, cname, id_name, title, hero_type, roles_json, moss_id, source_url, fetched_at, content_hash
            FROM heroes WHERE hero_id = ?1"#,
            params![hero_id],
            Self::row_hero,
        ).optional().map_err(Into::into)
    }

    pub fn get_hero_profile(&self, query: &str) -> Result<HeroProfile> {
        let hero = self.resolve_hero(query)?;
        let mut stmt = self.conn.prepare(
            r#"SELECT hero_id, slot, name, cooldown, cost, description, source_url, fetched_at, content_hash
            FROM hero_skills WHERE hero_id = ?1
            ORDER BY CASE slot WHEN 'passive' THEN 0 WHEN 'skill_1' THEN 1 WHEN 'skill_2' THEN 2 WHEN 'skill_3' THEN 3 ELSE 9 END, slot"#,
        )?;
        let skills = stmt
            .query_map(params![hero.hero_id], |row| {
                Ok(HeroSkill {
                    hero_id: row.get(0)?,
                    slot: row.get(1)?,
                    name: row.get(2)?,
                    cooldown: row.get(3)?,
                    cost: row.get(4)?,
                    description: row.get(5)?,
                    source: SourceInfo {
                        url: row.get(6)?,
                        fetched_at: row.get(7)?,
                        content_hash: row.get(8)?,
                    },
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let mut wstmt = self
            .conn
            .prepare("SELECT warning FROM hero_parse_warnings WHERE hero_id = ?1 ORDER BY rowid")?;
        let warnings = wstmt
            .query_map(params![hero.hero_id], |row| row.get(0))?
            .collect::<rusqlite::Result<Vec<String>>>()?;
        Ok(HeroProfile {
            hero,
            skills,
            parse_warnings: warnings,
        })
    }

    pub fn get_hero_skill(&self, hero_query: &str, slot: &str) -> Result<HeroSkill> {
        let hero = self.resolve_hero(hero_query)?;
        let normalized = normalize_skill_slot(slot);
        self.conn.query_row(
            r#"SELECT hero_id, slot, name, cooldown, cost, description, source_url, fetched_at, content_hash
            FROM hero_skills WHERE hero_id = ?1 AND slot = ?2"#,
            params![hero.hero_id, normalized],
            |row| Ok(HeroSkill {
                hero_id: row.get(0)?, slot: row.get(1)?, name: row.get(2)?, cooldown: row.get(3)?,
                cost: row.get(4)?, description: row.get(5)?,
                source: SourceInfo { url: row.get(6)?, fetched_at: row.get(7)?, content_hash: row.get(8)? },
            })
        ).optional()?.ok_or_else(|| anyhow!("skill not found: {hero_query} {slot}"))
    }

    pub fn search_items(&self, query: &str, limit: usize) -> Result<Vec<Item>> {
        let pat = format!("%{}%", query.trim());
        let mut stmt = self.conn.prepare(
            r#"SELECT item_id, item_name, item_type, price, total_price, description_html, description_text, source_url, fetched_at, content_hash
            FROM items WHERE item_name LIKE ?1 OR description_text LIKE ?1 ORDER BY item_id LIMIT ?2"#,
        )?;
        stmt.query_map(params![pat, limit as i64], Self::row_item)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn get_item(&self, query: &str) -> Result<Item> {
        let matches = self.search_items(query, 8)?;
        let exact = matches
            .iter()
            .find(|i| i.item_name == query.trim())
            .cloned();
        if let Some(item) = exact {
            return Ok(item);
        }
        match matches.len() {
            0 => Err(anyhow!("item not found: {query}")),
            1 => Ok(matches.into_iter().next().unwrap()),
            _ => Err(anyhow!(
                "ambiguous item `{}`; candidates: {}",
                query,
                matches
                    .iter()
                    .map(|i| i.item_name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }

    pub fn all_hero_profiles(&self) -> Result<Vec<HeroProfile>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT hero_id, ename, cname, id_name, title, hero_type, roles_json, moss_id, source_url, fetched_at, content_hash
            FROM heroes ORDER BY hero_id"#,
        )?;
        let heroes = stmt
            .query_map([], Self::row_hero)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        heroes
            .iter()
            .map(|h| self.get_hero_profile(&h.cname))
            .collect::<Result<Vec<_>>>()
    }

    pub fn all_items(&self) -> Result<Vec<Item>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT item_id, item_name, item_type, price, total_price, description_html, description_text, source_url, fetched_at, content_hash
            FROM items ORDER BY item_id"#,
        )?;
        stmt.query_map([], Self::row_item)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn get_summoner_skills(&self) -> Result<Vec<SummonerSkill>> {
        let mut stmt = self.conn.prepare(
            "SELECT skill_id, name, rank, description, source_url, fetched_at, content_hash FROM summoner_skills ORDER BY skill_id",
        )?;
        stmt.query_map([], Self::row_summoner)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn get_summoner_skill(&self, query: &str) -> Result<SummonerSkill> {
        let pat = format!("%{}%", query.trim());
        let mut stmt = self.conn.prepare(
            "SELECT skill_id, name, rank, description, source_url, fetched_at, content_hash FROM summoner_skills WHERE name LIKE ?1 OR description LIKE ?1 ORDER BY skill_id LIMIT 8",
        )?;
        let matches = stmt
            .query_map(params![pat], Self::row_summoner)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let exact = matches.iter().find(|s| s.name == query.trim()).cloned();
        if let Some(skill) = exact {
            return Ok(skill);
        }
        match matches.len() {
            0 => Err(anyhow!("summoner skill not found: {query}")),
            1 => Ok(matches.into_iter().next().unwrap()),
            _ => Err(anyhow!(
                "ambiguous summoner skill `{}`; candidates: {}",
                query,
                matches
                    .iter()
                    .map(|s| s.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }

    fn row_hero(row: &rusqlite::Row<'_>) -> rusqlite::Result<HeroBasic> {
        let roles_json: String = row.get(6)?;
        let roles = serde_json::from_str(&roles_json).unwrap_or_default();
        Ok(HeroBasic {
            hero_id: row.get(0)?,
            ename: row.get(1)?,
            cname: row.get(2)?,
            id_name: row.get(3)?,
            title: row.get(4)?,
            hero_type: row.get(5)?,
            roles,
            moss_id: row.get(7)?,
            source: SourceInfo {
                url: row.get(8)?,
                fetched_at: row.get(9)?,
                content_hash: row.get(10)?,
            },
        })
    }

    fn row_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<Item> {
        Ok(Item {
            item_id: row.get(0)?,
            item_name: row.get(1)?,
            item_type: row.get(2)?,
            price: row.get(3)?,
            total_price: row.get(4)?,
            description_html: row.get(5)?,
            description_text: row.get(6)?,
            source: SourceInfo {
                url: row.get(7)?,
                fetched_at: row.get(8)?,
                content_hash: row.get(9)?,
            },
        })
    }

    fn row_summoner(row: &rusqlite::Row<'_>) -> rusqlite::Result<SummonerSkill> {
        Ok(SummonerSkill {
            skill_id: row.get(0)?,
            name: row.get(1)?,
            rank: row.get(2)?,
            description: row.get(3)?,
            source: SourceInfo {
                url: row.get(4)?,
                fetched_at: row.get(5)?,
                content_hash: row.get(6)?,
            },
        })
    }
}
