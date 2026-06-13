use super::Store;
use crate::model::*;
use anyhow::{Result, anyhow};
use rusqlite::{OptionalExtension, params};

impl Store {
    pub fn search_heroes(&self, query: &str, limit: usize) -> Result<Vec<HeroBasic>> {
        let pat = format!("%{}%", query.trim());
        let mut stmt = self.conn.prepare(
            r#"SELECT hero_id, ename, cname, id_name, title, hero_type, roles_json, moss_id, source_url, fetched_at, content_hash
            FROM heroes
            WHERE cname LIKE ?1 OR id_name LIKE ?1 OR title LIKE ?1
               OR CAST(hero_id AS TEXT) = ?2
            ORDER BY hero_id
            LIMIT ?3"#,
        )?;
        let rows = stmt.query_map(params![pat, query.trim(), limit as i64], Self::row_hero)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn list_heroes(&self, limit: usize) -> Result<Vec<HeroBasic>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT hero_id, ename, cname, id_name, title, hero_type, roles_json, moss_id, source_url, fetched_at, content_hash
            FROM heroes ORDER BY hero_id LIMIT ?1"#,
        )?;
        stmt.query_map(params![limit as i64], Self::row_hero)?
            .collect::<rusqlite::Result<Vec<_>>>()
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
            FROM hero_skills WHERE hero_id = ?1 ORDER BY
              CASE slot WHEN 'passive' THEN 0 WHEN 'skill_1' THEN 1 WHEN 'skill_2' THEN 2 WHEN 'skill_3' THEN 3 ELSE 9 END,
              slot"#,
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
            FROM hero_skills WHERE hero_id = ?1 AND (slot = ?2 OR name = ?3) LIMIT 1"#,
            params![hero.hero_id, normalized, slot],
            |row| Ok(HeroSkill {
                hero_id: row.get(0)?, slot: row.get(1)?, name: row.get(2)?, cooldown: row.get(3)?, cost: row.get(4)?, description: row.get(5)?,
                source: SourceInfo { url: row.get(6)?, fetched_at: row.get(7)?, content_hash: row.get(8)? },
            })
        ).optional()?.ok_or_else(|| anyhow!("skill `{slot}` not found for hero {}", hero.cname))
    }

    pub fn search_hero_skills(&self, query: &str, limit: usize) -> Result<Vec<HeroSkillSearchHit>> {
        let pat = format!("%{}%", query.trim());
        let normalized = normalize_skill_slot(query);
        let mut stmt = self.conn.prepare(
            r#"SELECT
              h.hero_id, h.ename, h.cname, h.id_name, h.title, h.hero_type, h.roles_json, h.moss_id, h.source_url, h.fetched_at, h.content_hash,
              s.hero_id, s.slot, s.name, s.cooldown, s.cost, s.description, s.source_url, s.fetched_at, s.content_hash
            FROM hero_skills s
            JOIN heroes h ON h.hero_id = s.hero_id
            WHERE h.cname LIKE ?1 OR h.id_name LIKE ?1 OR s.name LIKE ?1 OR s.description LIKE ?1 OR s.slot = ?2
            ORDER BY h.hero_id,
              CASE s.slot WHEN 'passive' THEN 0 WHEN 'skill_1' THEN 1 WHEN 'skill_2' THEN 2 WHEN 'skill_3' THEN 3 ELSE 9 END,
              s.slot
            LIMIT ?3"#,
        )?;
        stmt.query_map(params![pat, normalized, limit as i64], |row| {
            let roles_json: String = row.get(6)?;
            let roles = serde_json::from_str(&roles_json).unwrap_or_default();
            let hero = HeroBasic {
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
            };
            let skill = HeroSkill {
                hero_id: row.get(11)?,
                slot: row.get(12)?,
                name: row.get(13)?,
                cooldown: row.get(14)?,
                cost: row.get(15)?,
                description: row.get(16)?,
                source: SourceInfo {
                    url: row.get(17)?,
                    fetched_at: row.get(18)?,
                    content_hash: row.get(19)?,
                },
            };
            Ok(HeroSkillSearchHit { hero, skill })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
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
}
