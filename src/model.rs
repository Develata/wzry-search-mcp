use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SourceInfo {
    pub url: String,
    pub fetched_at: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HeroBasic {
    pub hero_id: i64,
    pub ename: i64,
    pub cname: String,
    pub id_name: Option<String>,
    pub title: Option<String>,
    pub hero_type: Option<i64>,
    pub roles: Vec<String>,
    pub moss_id: Option<i64>,
    pub source: SourceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HeroSkill {
    pub hero_id: i64,
    pub slot: String,
    pub name: String,
    pub cooldown: Option<String>,
    pub cost: Option<String>,
    pub description: String,
    pub source: SourceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HeroProfile {
    pub hero: HeroBasic,
    pub skills: Vec<HeroSkill>,
    pub parse_warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HeroSkillSearchHit {
    pub hero: HeroBasic,
    pub skill: HeroSkill,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Item {
    pub item_id: i64,
    pub item_name: String,
    pub item_type: Option<i64>,
    pub price: Option<i64>,
    pub total_price: Option<i64>,
    pub description_html: Option<String>,
    pub description_text: Option<String>,
    pub source: SourceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SummonerSkill {
    pub skill_id: i64,
    pub name: String,
    pub rank: Option<i64>,
    pub description: Option<String>,
    pub source: SourceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateStatus {
    pub changed: bool,
    pub snapshots: Vec<SourceSnapshot>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NewsArticle {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NewsArticleMatch {
    pub article: NewsArticle,
    pub affected_heroes: Vec<HeroBasic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NewsIncrementalSyncResult {
    pub checked_articles: usize,
    pub matched_articles: Vec<NewsArticleMatch>,
    pub affected_heroes: Vec<HeroBasic>,
    pub synced_heroes: Vec<HeroBasic>,
    pub dry_run: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SyncUpdateResult {
    pub status: String,
    pub dry_run: bool,
    pub lock_file: Option<String>,
    pub source_status: UpdateStatus,
    pub news_incremental: NewsIncrementalSyncResult,
    pub full_sync_ran: bool,
    pub full_sync_reason: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SourceSnapshot {
    pub source_key: String,
    pub url: String,
    pub fetched_at: String,
    pub content_hash: String,
    pub byte_len: i64,
    pub changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LineupContext {
    pub allies: Vec<HeroProfile>,
    pub enemies: Vec<HeroProfile>,
    pub candidate_pool: Vec<HeroProfile>,
    pub recommendation_should_be_done_by_model: bool,
}

pub fn hero_type_to_roles(hero_type: Option<i64>) -> Vec<String> {
    // Official herolist uses a single numeric primary type. Keep the mapping factual/coarse.
    match hero_type {
        Some(1) => vec!["战士".to_string()],
        Some(2) => vec!["法师".to_string()],
        Some(3) => vec!["坦克".to_string()],
        Some(4) => vec!["刺客".to_string()],
        Some(5) => vec!["射手".to_string()],
        Some(6) => vec!["辅助".to_string()],
        _ => vec![],
    }
}

pub fn normalize_skill_slot(input: &str) -> String {
    let x = input.trim().to_lowercase();
    match x.as_str() {
        "passive" | "被动" | "0" => "passive".to_string(),
        "1" | "一" | "一技能" | "skill1" | "skill_1" => "skill_1".to_string(),
        "2" | "二" | "二技能" | "skill2" | "skill_2" => "skill_2".to_string(),
        "3" | "三" | "三技能" | "大招" | "skill3" | "skill_3" => "skill_3".to_string(),
        _ => x,
    }
}
