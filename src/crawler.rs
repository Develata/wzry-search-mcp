use crate::db::Store;
use crate::model::*;
use crate::parser::{
    detect_affected_heroes, is_update_like_news_title, parse_hero_list, parse_hero_skills,
    parse_items, parse_news_index, parse_summoner_skills,
};
use crate::util::*;
use anyhow::{Context, Result, anyhow};
use rand::Rng;
use reqwest::blocking::Client;
use std::{collections::HashSet, thread, time::Duration};

#[derive(Debug, Clone)]
pub struct CrawlConfig {
    pub min_delay_ms: u64,
    pub max_delay_ms: u64,
    pub max_retries: usize,
    pub user_agent: String,
}

impl Default for CrawlConfig {
    fn default() -> Self {
        Self {
            min_delay_ms: 3000,
            max_delay_ms: 12000,
            max_retries: 2,
            user_agent: "wzry-search-mcp/0.3 (+https://github.com/Develata/wzry-search-mcp)"
                .to_string(),
        }
    }
}

pub struct Crawler {
    client: Client,
    cfg: CrawlConfig,
}

impl Crawler {
    pub fn new(cfg: CrawlConfig) -> Result<Self> {
        let client = Client::builder()
            .user_agent(&cfg.user_agent)
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self { client, cfg })
    }

    pub fn sync_all(
        &self,
        store: &mut Store,
        polite: bool,
        limit_heroes: Option<usize>,
    ) -> Result<()> {
        let heroes = self.sync_hero_list(store)?;
        self.sync_items(store)?;
        self.sync_summoner_skills(store)?;
        let iter = heroes.iter().take(limit_heroes.unwrap_or(usize::MAX));
        for (idx, hero) in iter.enumerate() {
            if polite && idx > 0 {
                self.sleep_polite();
            }
            match self.sync_hero_detail(store, hero) {
                Ok(count) => eprintln!("synced {} skills for {}", count, hero.cname),
                Err(err) => {
                    eprintln!(
                        "WARN: failed to sync hero detail {} {}: {err:#}",
                        hero.hero_id, hero.cname
                    );
                    store.add_update_event(
                        "hero_detail_error",
                        Some("hero_detail"),
                        &format!("{} {}: {err:#}", hero.hero_id, hero.cname),
                    )?;
                }
            }
        }
        Ok(())
    }

    pub fn check_updates(&self, store: &Store) -> Result<UpdateStatus> {
        let sources = [
            ("herolist", HERO_LIST_URL),
            ("items", ITEM_LIST_URL),
            ("summoner", SUMMONER_JSON_URL),
        ];
        let mut snapshots = Vec::new();
        let mut changed = false;
        for (key, url) in sources {
            let bytes = self.fetch_bytes(url)?;
            let hash = sha256_hex(&bytes);
            let prev = store.get_snapshot_hash(key)?;
            let is_changed = prev.as_deref() != Some(hash.as_str());
            changed |= is_changed;
            snapshots.push(SourceSnapshot {
                source_key: key.to_string(),
                url: url.to_string(),
                fetched_at: now_rfc3339(),
                content_hash: hash,
                byte_len: bytes.len() as i64,
                changed: is_changed,
            });
        }
        Ok(UpdateStatus {
            changed,
            snapshots,
            message: if changed {
                "one or more sources changed"
            } else {
                "no source hash changed"
            }
            .to_string(),
        })
    }

    pub fn write_update_snapshots(&self, store: &Store, status: &UpdateStatus) -> Result<()> {
        for snap in &status.snapshots {
            if snap.changed {
                store.add_update_event(
                    "source_changed",
                    Some(&snap.source_key),
                    &format!("{} changed", snap.url),
                )?;
            }
            store.upsert_snapshot(snap)?;
        }
        Ok(())
    }

    pub fn sync_changed_from_news(
        &self,
        store: &mut Store,
        news_limit: usize,
        dry_run: bool,
        polite: bool,
    ) -> Result<NewsIncrementalSyncResult> {
        let heroes = store.list_heroes(usize::MAX)?;
        if heroes.is_empty() {
            return Err(anyhow!(
                "local hero catalog is empty; run `wzry-search-mcp --db <path> sync` first"
            ));
        }

        let news_bytes = self.fetch_bytes(NEWS_INDEX_URL)?;
        let news_text = decode_response(&news_bytes, NEWS_INDEX_URL)?;
        let update_articles = parse_news_index(&news_text)
            .into_iter()
            .filter(|article| is_update_like_news_title(&article.title))
            .take(news_limit)
            .collect::<Vec<_>>();

        let mut matched_articles = Vec::new();
        let mut affected_heroes = Vec::new();
        let mut seen_hero_ids = HashSet::new();
        let mut warnings = Vec::new();

        for (idx, article) in update_articles.iter().enumerate() {
            if polite && idx > 0 {
                self.sleep_polite();
            }
            let mut haystack = article.title.clone();
            match self.fetch_bytes(&article.url) {
                Ok(bytes) => match decode_response(&bytes, &article.url) {
                    Ok(text) => {
                        haystack.push('\n');
                        haystack.push_str(&strip_html_to_text(&text));
                    }
                    Err(err) => warnings.push(format!(
                        "failed to decode news article `{}` {}: {err:#}",
                        article.title, article.url
                    )),
                },
                Err(err) => warnings.push(format!(
                    "failed to fetch news article `{}` {}: {err:#}",
                    article.title, article.url
                )),
            }

            let article_heroes = detect_affected_heroes(&haystack, &heroes);
            if !article_heroes.is_empty() {
                for hero in &article_heroes {
                    if seen_hero_ids.insert(hero.hero_id) {
                        affected_heroes.push(hero.clone());
                    }
                }
                matched_articles.push(NewsArticleMatch {
                    article: article.clone(),
                    affected_heroes: article_heroes,
                });
            }
        }

        affected_heroes.sort_by_key(|hero| hero.hero_id);
        let mut synced_heroes = Vec::new();
        if !dry_run {
            for (idx, hero) in affected_heroes.iter().enumerate() {
                if polite && idx > 0 {
                    self.sleep_polite();
                }
                match self.sync_hero_detail(store, hero) {
                    Ok(_) => synced_heroes.push(hero.clone()),
                    Err(err) => warnings.push(format!(
                        "failed to sync affected hero {} {}: {err:#}",
                        hero.hero_id, hero.cname
                    )),
                }
            }
        }

        if !dry_run {
            store.add_update_event(
                "news_incremental_sync",
                Some("news_index"),
                &format!(
                    "checked {} update-like articles, affected {}, synced {}",
                    update_articles.len(),
                    affected_heroes.len(),
                    synced_heroes.len()
                ),
            )?;
        }

        Ok(NewsIncrementalSyncResult {
            checked_articles: update_articles.len(),
            matched_articles,
            affected_heroes,
            synced_heroes,
            dry_run,
            warnings,
        })
    }

    pub fn sync_hero_list(&self, store: &Store) -> Result<Vec<HeroBasic>> {
        let bytes = self.fetch_bytes(HERO_LIST_URL)?;
        let hash = sha256_hex(&bytes);
        let fetched_at = now_rfc3339();
        let text = decode_response(&bytes, HERO_LIST_URL)?;
        let heroes = parse_hero_list(&text, HERO_LIST_URL, &fetched_at, &hash)?;
        if heroes.is_empty() {
            return Err(anyhow!(
                "herolist source parsed as empty; refusing destructive sync"
            ));
        }
        for hero in &heroes {
            store.upsert_hero(hero)?;
        }
        store.retain_heroes_by_ids(&heroes.iter().map(|h| h.hero_id).collect::<Vec<_>>())?;
        store.upsert_snapshot(&SourceSnapshot {
            source_key: "herolist".to_string(),
            url: HERO_LIST_URL.to_string(),
            fetched_at,
            content_hash: hash,
            byte_len: bytes.len() as i64,
            changed: true,
        })?;
        Ok(heroes)
    }

    pub fn sync_hero_detail(&self, store: &mut Store, hero: &HeroBasic) -> Result<usize> {
        let mut last_err: Option<anyhow::Error> = None;
        let mut fetched: Option<(String, Vec<u8>)> = None;
        for url in hero_detail_url_candidates(hero) {
            match self.fetch_bytes(&url) {
                Ok(bytes) => {
                    fetched = Some((url, bytes));
                    break;
                }
                Err(err) => last_err = Some(err),
            }
        }
        let (url, bytes) = fetched.ok_or_else(|| {
            last_err.unwrap_or_else(|| anyhow!("no hero detail URL candidate for {}", hero.cname))
        })?;
        let hash = sha256_hex(&bytes);
        let fetched_at = now_rfc3339();
        let text = decode_response(&bytes, &url)?;
        let (skills, warnings) = parse_hero_skills(hero.hero_id, &url, &fetched_at, &hash, &text)?;
        if !warnings.is_empty() {
            return Err(anyhow!(
                "hero detail parse warnings for {}: {}",
                hero.cname,
                warnings.join("; ")
            ));
        }
        store.replace_hero_skills(hero.hero_id, &skills, &warnings)?;
        Ok(skills.len())
    }

    pub fn sync_items(&self, store: &Store) -> Result<usize> {
        let bytes = self.fetch_bytes(ITEM_LIST_URL)?;
        let hash = sha256_hex(&bytes);
        let fetched_at = now_rfc3339();
        let text = decode_response(&bytes, ITEM_LIST_URL)?;
        let items = parse_items(&text, ITEM_LIST_URL, &fetched_at, &hash)?;
        if items.is_empty() {
            return Err(anyhow!(
                "item source parsed as empty; refusing destructive sync"
            ));
        }
        let count = items.len();
        for item in &items {
            store.upsert_item(item)?;
        }
        store.retain_items_by_ids(&items.iter().map(|i| i.item_id).collect::<Vec<_>>())?;
        store.upsert_snapshot(&SourceSnapshot {
            source_key: "items".to_string(),
            url: ITEM_LIST_URL.to_string(),
            fetched_at,
            content_hash: hash,
            byte_len: bytes.len() as i64,
            changed: true,
        })?;
        Ok(count)
    }

    pub fn sync_summoner_skills(&self, store: &Store) -> Result<usize> {
        let bytes = self.fetch_bytes(SUMMONER_JSON_URL)?;
        let hash = sha256_hex(&bytes);
        let fetched_at = now_rfc3339();
        let text = decode_response(&bytes, SUMMONER_JSON_URL)?;
        let skills = parse_summoner_skills(&text, SUMMONER_JSON_URL, &fetched_at, &hash)?;
        if skills.is_empty() {
            return Err(anyhow!(
                "summoner source parsed as empty; refusing destructive sync"
            ));
        }
        let count = skills.len();
        for skill in &skills {
            store.upsert_summoner_skill(skill)?;
        }
        store.retain_summoner_skills_by_ids(
            &skills.iter().map(|s| s.skill_id).collect::<Vec<_>>(),
        )?;
        store.upsert_snapshot(&SourceSnapshot {
            source_key: "summoner".to_string(),
            url: SUMMONER_JSON_URL.to_string(),
            fetched_at,
            content_hash: hash,
            byte_len: bytes.len() as i64,
            changed: true,
        })?;
        Ok(count)
    }

    fn fetch_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let mut last_err: Option<anyhow::Error> = None;
        for attempt in 0..=self.cfg.max_retries {
            match self
                .client
                .get(url)
                .header("Referer", "https://pvp.qq.com/")
                .send()
            {
                Ok(resp) => match resp.error_for_status() {
                    Ok(ok) => return ok.bytes().map(|b| b.to_vec()).map_err(Into::into),
                    Err(err) => last_err = Some(err.into()),
                },
                Err(err) => last_err = Some(err.into()),
            }
            if attempt < self.cfg.max_retries {
                thread::sleep(Duration::from_millis(500 * (attempt as u64 + 1)));
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow!("fetch failed: {url}")))
            .with_context(|| format!("fetch {url}"))
    }

    fn sleep_polite(&self) {
        let min = self.cfg.min_delay_ms.min(self.cfg.max_delay_ms);
        let max = self.cfg.min_delay_ms.max(self.cfg.max_delay_ms);
        let delay = rand::rng().random_range(min..=max);
        thread::sleep(Duration::from_millis(delay));
    }
}

fn hero_detail_url_candidates(hero: &HeroBasic) -> Vec<String> {
    let mut urls = Vec::new();
    urls.push(hero_detail_url(hero.hero_id));
    if let Some(id_name) = &hero.id_name {
        let slug_url = format!("https://pvp.qq.com/web201605/herodetail/{id_name}.shtml");
        if !urls.iter().any(|u| u == &slug_url) {
            urls.push(slug_url);
        }
    }
    urls
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hero_detail_candidates_try_numeric_then_slug() {
        let hero = HeroBasic {
            hero_id: 519,
            ename: 519,
            cname: "敖隐".to_string(),
            id_name: Some("aoyin".to_string()),
            title: None,
            hero_type: Some(5),
            roles: vec!["射手".to_string()],
            moss_id: None,
            source: SourceInfo {
                url: "u".to_string(),
                fetched_at: "t".to_string(),
                content_hash: "h".to_string(),
            },
        };
        let urls = hero_detail_url_candidates(&hero);
        assert_eq!(urls[0], "https://pvp.qq.com/web201605/herodetail/519.shtml");
        assert_eq!(
            urls[1],
            "https://pvp.qq.com/web201605/herodetail/aoyin.shtml"
        );
    }
}
