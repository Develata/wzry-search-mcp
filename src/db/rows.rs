use super::Store;
use crate::model::*;

impl Store {
    pub(super) fn row_hero(row: &rusqlite::Row<'_>) -> rusqlite::Result<HeroBasic> {
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

    pub(super) fn row_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<Item> {
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

    pub(super) fn row_summoner(row: &rusqlite::Row<'_>) -> rusqlite::Result<SummonerSkill> {
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
