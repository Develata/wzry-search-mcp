use crate::db::Store;
use anyhow::Result;
use serde_json::json;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Csv,
}

impl ExportFormat {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            _ => anyhow::bail!("unsupported export format `{s}`; expected json or csv"),
        }
    }
}

pub fn export_store(store: &Store, format: ExportFormat, out: &Path) -> Result<()> {
    match format {
        ExportFormat::Json => export_json(store, out),
        ExportFormat::Csv => export_csv(store, out),
    }
}

fn export_json(store: &Store, out: &Path) -> Result<()> {
    let heroes = store.all_hero_profiles()?;
    let items = store.all_items()?;
    let summoner_skills = store.get_summoner_skills()?;
    let doc = json!({
        "schema_version": 1,
        "heroes": heroes,
        "items": items,
        "summoner_skills": summoner_skills,
        "excluded": ["skins", "image_assets", "skin_image_urls", "runes"]
    });
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(out)?;
    serde_json::to_writer_pretty(BufWriter::new(file), &doc)?;
    Ok(())
}

fn export_csv(store: &Store, out_dir: &Path) -> Result<()> {
    fs::create_dir_all(out_dir)?;
    let profiles = store.all_hero_profiles()?;
    let items = store.all_items()?;
    let summoner = store.get_summoner_skills()?;

    let mut heroes = CsvWriter::create(&out_dir.join("heroes.csv"))?;
    heroes.row(&[
        "hero_id",
        "ename",
        "cname",
        "id_name",
        "title",
        "hero_type",
        "roles",
        "moss_id",
        "source_url",
        "fetched_at",
        "content_hash",
    ])?;
    let mut skills = CsvWriter::create(&out_dir.join("hero_skills.csv"))?;
    skills.row(&[
        "hero_id",
        "slot",
        "name",
        "cooldown",
        "cost",
        "description",
        "source_url",
        "fetched_at",
        "content_hash",
    ])?;
    for profile in &profiles {
        let h = &profile.hero;
        heroes.row(&[
            h.hero_id.to_string(),
            h.ename.to_string(),
            h.cname.clone(),
            h.id_name.clone().unwrap_or_default(),
            h.title.clone().unwrap_or_default(),
            h.hero_type.map(|x| x.to_string()).unwrap_or_default(),
            h.roles.join("|"),
            h.moss_id.map(|x| x.to_string()).unwrap_or_default(),
            h.source.url.clone(),
            h.source.fetched_at.clone(),
            h.source.content_hash.clone(),
        ])?;
        for s in &profile.skills {
            skills.row(&[
                s.hero_id.to_string(),
                s.slot.clone(),
                s.name.clone(),
                s.cooldown.clone().unwrap_or_default(),
                s.cost.clone().unwrap_or_default(),
                s.description.clone(),
                s.source.url.clone(),
                s.source.fetched_at.clone(),
                s.source.content_hash.clone(),
            ])?;
        }
    }

    let mut item_writer = CsvWriter::create(&out_dir.join("items.csv"))?;
    item_writer.row(&[
        "item_id",
        "item_name",
        "item_type",
        "price",
        "total_price",
        "description_text",
        "source_url",
        "fetched_at",
        "content_hash",
    ])?;
    for item in &items {
        item_writer.row(&[
            item.item_id.to_string(),
            item.item_name.clone(),
            item.item_type.map(|x| x.to_string()).unwrap_or_default(),
            item.price.map(|x| x.to_string()).unwrap_or_default(),
            item.total_price.map(|x| x.to_string()).unwrap_or_default(),
            item.description_text.clone().unwrap_or_default(),
            item.source.url.clone(),
            item.source.fetched_at.clone(),
            item.source.content_hash.clone(),
        ])?;
    }

    let mut summoner_writer = CsvWriter::create(&out_dir.join("summoner_skills.csv"))?;
    summoner_writer.row(&[
        "skill_id",
        "name",
        "rank",
        "description",
        "source_url",
        "fetched_at",
        "content_hash",
    ])?;
    for s in &summoner {
        summoner_writer.row(&[
            s.skill_id.to_string(),
            s.name.clone(),
            s.rank.map(|x| x.to_string()).unwrap_or_default(),
            s.description.clone().unwrap_or_default(),
            s.source.url.clone(),
            s.source.fetched_at.clone(),
            s.source.content_hash.clone(),
        ])?;
    }
    Ok(())
}

struct CsvWriter {
    writer: BufWriter<File>,
}

impl CsvWriter {
    fn create(path: &Path) -> Result<Self> {
        Ok(Self {
            writer: BufWriter::new(File::create(path)?),
        })
    }

    fn row<S: AsRef<str>>(&mut self, fields: &[S]) -> Result<()> {
        for (idx, field) in fields.iter().enumerate() {
            if idx > 0 {
                self.writer.write_all(b",")?;
            }
            write_csv_field(&mut self.writer, field.as_ref())?;
        }
        self.writer.write_all(b"\n")?;
        Ok(())
    }
}

fn write_csv_field(writer: &mut impl Write, value: &str) -> Result<()> {
    let needs_quote = value.contains([',', '"', '\n', '\r']);
    if !needs_quote {
        writer.write_all(value.as_bytes())?;
        return Ok(());
    }
    writer.write_all(b"\"")?;
    for ch in value.chars() {
        if ch == '"' {
            writer.write_all(b"\"\"")?;
        } else {
            write!(writer, "{ch}")?;
        }
    }
    writer.write_all(b"\"")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_escape_quotes_commas_and_newlines() {
        let mut out = Vec::new();
        write_csv_field(&mut out, "a,\"b\"\nc").unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "\"a,\"\"b\"\"\nc\"");
    }
}
