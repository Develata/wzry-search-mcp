use super::Store;
use crate::model::*;
use crate::util::now_rfc3339;
use anyhow::Result;
use rusqlite::{OptionalExtension, params, params_from_iter};

impl Store {
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

    pub fn retain_heroes_by_ids(&self, hero_ids: &[i64]) -> Result<usize> {
        self.delete_not_in("heroes", "hero_id", hero_ids)
    }

    pub fn retain_items_by_ids(&self, item_ids: &[i64]) -> Result<usize> {
        self.delete_not_in("items", "item_id", item_ids)
    }

    pub fn retain_summoner_skills_by_ids(&self, skill_ids: &[i64]) -> Result<usize> {
        self.delete_not_in("summoner_skills", "skill_id", skill_ids)
    }

    fn delete_not_in(&self, table: &str, id_column: &str, ids: &[i64]) -> Result<usize> {
        if ids.is_empty() {
            return self
                .conn
                .execute(&format!("DELETE FROM {table}"), [])
                .map_err(Into::into);
        }
        let placeholders = std::iter::repeat_n("?", ids.len())
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!("DELETE FROM {table} WHERE {id_column} NOT IN ({placeholders})");
        self.conn
            .execute(&sql, params_from_iter(ids.iter()))
            .map_err(Into::into)
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
}
