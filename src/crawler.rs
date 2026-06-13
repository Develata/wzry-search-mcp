use crate::db::Store;
use crate::model::*;
use crate::util::*;
use anyhow::{Context, Result, anyhow};
use rand::Rng;
use regex::Regex;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::{thread, time::Duration};

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
            user_agent: "wzry-search-mcp/0.1 (+https://github.com/Develata/wzry-search-mcp)"
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
                        "parse_warning",
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
            ("news_index", NEWS_INDEX_URL),
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

    pub fn sync_hero_list(&self, store: &Store) -> Result<Vec<HeroBasic>> {
        let bytes = self.fetch_bytes(HERO_LIST_URL)?;
        let hash = sha256_hex(&bytes);
        let fetched_at = now_rfc3339();
        let text = decode_response(&bytes, HERO_LIST_URL)?;
        let raw: Vec<RawHero> = serde_json::from_str(&text).context("parse herolist.json")?;
        let mut heroes = Vec::with_capacity(raw.len());
        for h in raw {
            let hero_type = h.hero_type;
            let hero = HeroBasic {
                hero_id: h.ename,
                ename: h.ename,
                cname: h.cname,
                id_name: h.id_name,
                title: h.title,
                hero_type,
                roles: hero_type_to_roles(hero_type),
                moss_id: h.moss_id,
                source: SourceInfo {
                    url: HERO_LIST_URL.to_string(),
                    fetched_at: fetched_at.clone(),
                    content_hash: hash.clone(),
                },
            };
            store.upsert_hero(&hero)?;
            heroes.push(hero);
        }
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
        let url = hero_detail_url_for(hero);
        let bytes = self.fetch_bytes(&url)?;
        let hash = sha256_hex(&bytes);
        let fetched_at = now_rfc3339();
        let text = decode_response(&bytes, &url)?;
        let (skills, warnings) = parse_hero_skills(hero.hero_id, &url, &fetched_at, &hash, &text)?;
        store.replace_hero_skills(hero.hero_id, &skills, &warnings)?;
        Ok(skills.len())
    }

    pub fn sync_items(&self, store: &Store) -> Result<usize> {
        let bytes = self.fetch_bytes(ITEM_LIST_URL)?;
        let hash = sha256_hex(&bytes);
        let fetched_at = now_rfc3339();
        let text = decode_response(&bytes, ITEM_LIST_URL)?;
        let raw: Vec<RawItem> = serde_json::from_str(&text).context("parse item.json")?;
        let count = raw.len();
        for item in raw {
            let description_text = item.des1.as_deref().map(strip_html_to_text);
            store.upsert_item(&Item {
                item_id: item.item_id,
                item_name: item.item_name,
                item_type: item.item_type,
                price: item.price,
                total_price: item.total_price,
                description_html: item.des1,
                description_text,
                source: SourceInfo {
                    url: ITEM_LIST_URL.to_string(),
                    fetched_at: fetched_at.clone(),
                    content_hash: hash.clone(),
                },
            })?;
        }
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
        let raw: Vec<RawSummoner> = serde_json::from_str(&text).context("parse summoner.json")?;
        let count = raw.len();
        let rank_re = Regex::new(r"\d+").unwrap();
        for s in raw {
            let rank = s
                .summoner_rank
                .as_deref()
                .and_then(|r| rank_re.find(r).and_then(|m| m.as_str().parse::<i64>().ok()));
            store.upsert_summoner_skill(&SummonerSkill {
                skill_id: s.summoner_id,
                name: s.summoner_name,
                rank,
                description: s.summoner_description,
                source: SourceInfo {
                    url: SUMMONER_JSON_URL.to_string(),
                    fetched_at: fetched_at.clone(),
                    content_hash: hash.clone(),
                },
            })?;
        }
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

fn hero_detail_url_for(hero: &HeroBasic) -> String {
    let use_slug = is_new_style_hero(hero);
    match (use_slug, hero.id_name.as_ref()) {
        (true, Some(id_name)) => format!("https://pvp.qq.com/web201605/herodetail/{id_name}.shtml"),
        _ => hero_detail_url(hero.hero_id),
    }
}

fn is_new_style_hero(hero: &HeroBasic) -> bool {
    hero.hero_id >= 500 || hero.hero_id == 151 || hero.hero_id == 172 || hero.hero_id == 188
}

pub fn parse_hero_skills(
    hero_id: i64,
    url: &str,
    fetched_at: &str,
    page_hash: &str,
    html: &str,
) -> Result<(Vec<HeroSkill>, Vec<String>)> {
    let document = Html::parse_document(html);
    let show_sel = Selector::parse(".skill-show .show-list").unwrap();
    let name_sel = Selector::parse(".skill-name").unwrap();
    let b_sel = Selector::parse("b").unwrap();
    let span_sel = Selector::parse("span").unwrap();
    let desc_sel = Selector::parse(".skill-desc").unwrap();
    let mut skills = Vec::new();
    let mut warnings = Vec::new();

    for (idx, node) in document.select(&show_sel).enumerate() {
        let Some(name_node) = node.select(&name_sel).next() else {
            continue;
        };
        let name = name_node
            .select(&b_sel)
            .next()
            .map(|n| normalize_ws(&n.text().collect::<Vec<_>>().join("")))
            .unwrap_or_default();
        let desc = node
            .select(&desc_sel)
            .next()
            .map(|n| normalize_ws(&n.text().collect::<Vec<_>>().join("")))
            .unwrap_or_default();
        if name.is_empty() && desc.is_empty() {
            continue;
        }
        let spans = name_node
            .select(&span_sel)
            .map(|s| normalize_ws(&s.text().collect::<Vec<_>>().join("")))
            .collect::<Vec<_>>();
        let cooldown = spans.iter().find_map(|s| {
            s.strip_prefix("冷却值：")
                .or_else(|| s.strip_prefix("冷却值:"))
                .map(|x| x.to_string())
        });
        let cost = spans.iter().find_map(|s| {
            s.strip_prefix("消耗：")
                .or_else(|| s.strip_prefix("消耗:"))
                .map(|x| x.to_string())
        });
        let slot = match idx {
            0 => "passive".to_string(),
            1 => "skill_1".to_string(),
            2 => "skill_2".to_string(),
            3 => "skill_3".to_string(),
            n => format!("extra_{n}"),
        };
        let content_hash = text_sha256_hex(&format!(
            "{hero_id}|{slot}|{name}|{cooldown:?}|{cost:?}|{desc}"
        ));
        skills.push(HeroSkill {
            hero_id,
            slot,
            name,
            cooldown,
            cost,
            description: desc,
            source: SourceInfo {
                url: url.to_string(),
                fetched_at: fetched_at.to_string(),
                content_hash,
            },
        });
    }

    if skills.is_empty() {
        warnings.push("no skills parsed from official hero detail page".to_string());
    }
    if !skills.iter().any(|s| s.slot == "passive") {
        warnings.push("passive skill missing after parse".to_string());
    }
    if skills.len() < 4 {
        warnings.push(format!(
            "expected at least passive + 3 skills, parsed {} from page hash {page_hash}",
            skills.len()
        ));
    }
    Ok((skills, warnings))
}

#[derive(Debug, Deserialize)]
struct RawHero {
    ename: i64,
    cname: String,
    id_name: Option<String>,
    title: Option<String>,
    hero_type: Option<i64>,
    moss_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct RawItem {
    item_id: i64,
    item_name: String,
    item_type: Option<i64>,
    price: Option<i64>,
    total_price: Option<i64>,
    des1: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawSummoner {
    summoner_id: i64,
    summoner_name: String,
    summoner_rank: Option<String>,
    summoner_description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_skill_slots() {
        let html = r#"
        <div class="skill-show">
          <div class="show-list"><p class="skill-name"><b>被动名</b><span>冷却值：0</span><span>消耗：0</span></p><p class="skill-desc">被动描述</p></div>
          <div class="show-list"><p class="skill-name"><b>一技能</b><span>冷却值：1</span><span>消耗：2</span></p><p class="skill-desc">一描述</p></div>
          <div class="show-list"><p class="skill-name"><b>二技能</b></p><p class="skill-desc">二描述</p></div>
          <div class="show-list"><p class="skill-name"><b>三技能</b></p><p class="skill-desc">三描述</p></div>
        </div>
        "#;
        let (skills, warnings) = parse_hero_skills(1, "u", "t", "h", html).unwrap();
        assert_eq!(skills.len(), 4);
        assert_eq!(skills[0].slot, "passive");
        assert_eq!(skills[1].slot, "skill_1");
        assert!(warnings.is_empty());
    }

    #[test]
    fn parse_extra_skill_and_skip_empty_placeholder() {
        let html = r#"
        <div class="skill-show">
          <div class="show-list"><p class="skill-name"><b>被动</b><span>冷却值：0</span><span>消耗：0</span></p><p class="skill-desc">被动描述</p></div>
          <div class="show-list"><p class="skill-name"><b>一</b></p><p class="skill-desc">一描述</p></div>
          <div class="show-list"><p class="skill-name"><b>二</b></p><p class="skill-desc">二描述</p></div>
          <div class="show-list"><p class="skill-name"><b>三</b></p><p class="skill-desc">三描述</p></div>
          <div class="show-list"><p class="skill-name"><b>额外形态</b><span>冷却值：12/10</span></p><p class="skill-desc">额外技能描述</p></div>
          <div class="show-list"><p class="skill-name"><b></b><span>冷却值：</span><span>消耗：</span></p><p class="skill-desc"></p></div>
        </div>
        "#;
        let (skills, warnings) = parse_hero_skills(2, "u", "t", "h", html).unwrap();
        assert_eq!(skills.len(), 5);
        assert_eq!(skills[4].slot, "extra_4");
        assert_eq!(skills[4].name, "额外形态");
        assert_eq!(skills[4].cooldown.as_deref(), Some("12/10"));
        assert!(warnings.is_empty());
    }
}
