use crate::model::*;
use crate::util::{NEWS_INDEX_URL, normalize_ws, strip_html_to_text};
use anyhow::{Context, Result};
use regex::Regex;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::collections::HashSet;

pub fn parse_hero_list(
    text: &str,
    source_url: &str,
    fetched_at: &str,
    source_hash: &str,
) -> Result<Vec<HeroBasic>> {
    let raw: Vec<RawHero> = serde_json::from_str(text).context("parse herolist.json")?;
    Ok(raw
        .into_iter()
        .map(|h| {
            let hero_type = h.hero_type;
            HeroBasic {
                hero_id: h.ename,
                ename: h.ename,
                cname: h.cname,
                id_name: h.id_name,
                title: h.title,
                hero_type,
                roles: hero_type_to_roles(hero_type),
                moss_id: h.moss_id,
                source: SourceInfo {
                    url: source_url.to_string(),
                    fetched_at: fetched_at.to_string(),
                    content_hash: source_hash.to_string(),
                },
            }
        })
        .collect())
}

pub fn parse_items(
    text: &str,
    source_url: &str,
    fetched_at: &str,
    source_hash: &str,
) -> Result<Vec<Item>> {
    let raw: Vec<RawItem> = serde_json::from_str(text).context("parse item.json")?;
    Ok(raw
        .into_iter()
        .map(|item| Item {
            item_id: item.item_id,
            item_name: item.item_name,
            item_type: item.item_type,
            price: item.price,
            total_price: item.total_price,
            description_text: item.des1.as_deref().map(strip_html_to_text),
            description_html: item.des1,
            source: SourceInfo {
                url: source_url.to_string(),
                fetched_at: fetched_at.to_string(),
                content_hash: source_hash.to_string(),
            },
        })
        .collect())
}

pub fn parse_summoner_skills(
    text: &str,
    source_url: &str,
    fetched_at: &str,
    source_hash: &str,
) -> Result<Vec<SummonerSkill>> {
    let raw: Vec<RawSummoner> = serde_json::from_str(text).context("parse summoner.json")?;
    let rank_re = Regex::new(r"\d+").expect("valid summoner rank regex");
    Ok(raw
        .into_iter()
        .map(|s| {
            let rank = s
                .summoner_rank
                .as_deref()
                .and_then(|r| rank_re.find(r).and_then(|m| m.as_str().parse::<i64>().ok()));
            SummonerSkill {
                skill_id: s.summoner_id,
                name: s.summoner_name,
                rank,
                description: s.summoner_description,
                source: SourceInfo {
                    url: source_url.to_string(),
                    fetched_at: fetched_at.to_string(),
                    content_hash: source_hash.to_string(),
                },
            }
        })
        .collect())
}

pub fn parse_hero_skills(
    hero_id: i64,
    url: &str,
    fetched_at: &str,
    page_hash: &str,
    html: &str,
) -> Result<(Vec<HeroSkill>, Vec<String>)> {
    let document = Html::parse_document(html);
    let show_sel = Selector::parse(".skill-show .show-list").expect("valid hero skill selector");
    let name_sel = Selector::parse(".skill-name").expect("valid hero skill name selector");
    let b_sel = Selector::parse("b").expect("valid skill name text selector");
    let span_sel = Selector::parse("span").expect("valid cooldown/cost selector");
    let desc_sel = Selector::parse(".skill-desc").expect("valid hero skill description selector");
    let mut skills = Vec::new();
    let mut warnings = Vec::new();

    for node in document.select(&show_sel) {
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
        let slot = match skills.len() {
            0 => "passive".to_string(),
            1 => "skill_1".to_string(),
            2 => "skill_2".to_string(),
            3 => "skill_3".to_string(),
            n => format!("extra_{n}"),
        };
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
                content_hash: page_hash.to_string(),
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

pub fn parse_news_index(html: &str) -> Vec<NewsArticle> {
    let document = Html::parse_document(html);
    let anchor_sel = Selector::parse("a").expect("valid anchor selector");
    let mut seen = HashSet::new();
    let mut articles = Vec::new();
    for anchor in document.select(&anchor_sel) {
        let Some(href) = anchor.value().attr("href") else {
            continue;
        };
        if !href.contains("newsdetail.shtml") {
            continue;
        }
        let title = normalize_ws(&anchor.text().collect::<Vec<_>>().join(""));
        if title.is_empty() {
            continue;
        }
        let url = normalize_news_url(href);
        if seen.insert(url.clone()) {
            articles.push(NewsArticle { title, url });
        }
    }
    articles
}

pub fn is_update_like_news_title(title: &str) -> bool {
    const INCLUDE: &[&str] = &[
        "版本更新",
        "不停机更新",
        "英雄平衡",
        "平衡性调整",
        "英雄调整",
        "装备调整",
        "召唤师技能调整",
        "体验服",
    ];
    const EXCLUDE: &[&str] = &["活动", "福利", "皮肤", "赛事", "处罚", "排行榜"];
    INCLUDE.iter().any(|word| title.contains(word))
        && !EXCLUDE.iter().any(|word| title.contains(word))
}

pub fn detect_affected_heroes(text: &str, heroes: &[HeroBasic]) -> Vec<HeroBasic> {
    heroes
        .iter()
        .filter(|hero| hero_name_matches(text, &hero.cname))
        .cloned()
        .collect()
}

fn hero_name_matches(text: &str, cname: &str) -> bool {
    if cname.chars().count() > 1 {
        return text.contains(cname);
    }
    short_hero_name_matches(text, cname)
}

fn short_hero_name_matches(text: &str, cname: &str) -> bool {
    text.match_indices(cname).any(|(idx, _)| {
        let before = text[..idx].chars().next_back();
        let after = text[idx + cname.len()..].chars().next();
        before.is_none_or(is_hero_name_boundary) && after.is_none_or(is_hero_name_boundary)
    })
}

fn is_hero_name_boundary(ch: char) -> bool {
    !('\u{4e00}'..='\u{9fff}').contains(&ch) && !ch.is_ascii_alphanumeric()
}

fn normalize_news_url(href: &str) -> String {
    if href.starts_with("https://") || href.starts_with("http://") {
        href.to_string()
    } else if href.starts_with("//") {
        format!("https:{href}")
    } else if href.starts_with('/') {
        format!("https://pvp.qq.com{href}")
    } else {
        let base = NEWS_INDEX_URL
            .rsplit_once('/')
            .map(|(base, _)| base)
            .unwrap_or("https://pvp.qq.com/web201706");
        format!("{base}/{href}")
    }
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

    #[test]
    fn parse_news_index_filters_detail_links_and_deduplicates() {
        let html = r#"
        <a href="https://pvp.qq.com/web201706/newsdetail.shtml?tid=1">6月12日版本更新公告</a>
        <a href="https://pvp.qq.com/web201706/newsdetail.shtml?tid=1">6月12日版本更新公告</a>
        <a href="newsdetail.shtml?tid=2">英雄平衡性调整 | 鲁班大师玩法升级</a>
        <a href="javascript:;">公告</a>
        "#;
        let articles = parse_news_index(html);
        assert_eq!(articles.len(), 2);
        assert_eq!(articles[0].title, "6月12日版本更新公告");
        assert_eq!(
            articles[1].url,
            "https://pvp.qq.com/web201706/newsdetail.shtml?tid=2"
        );
    }

    #[test]
    fn update_like_title_and_hero_detection_are_conservative() {
        assert!(is_update_like_news_title(
            "英雄平衡性调整 | 鲁班大师玩法升级"
        ));
        assert!(is_update_like_news_title("6月12日版本更新公告"));
        assert!(!is_update_like_news_title("限时返场活动公告"));
        let heroes = vec![test_hero(1, "鲁班大师"), test_hero(2, "小乔")];
        let affected = detect_affected_heroes("鲁班大师玩法升级", &heroes);
        assert_eq!(affected.len(), 1);
        assert_eq!(affected[0].cname, "鲁班大师");

        let short_heroes = vec![test_hero(3, "镜"), test_hero(4, "铠")];
        assert!(
            detect_affected_heroes("镜：技能调整", &short_heroes)
                .iter()
                .any(|hero| hero.cname == "镜")
        );
        assert!(detect_affected_heroes("破镜重圆", &short_heroes).is_empty());
    }

    fn test_hero(hero_id: i64, cname: &str) -> HeroBasic {
        HeroBasic {
            hero_id,
            ename: hero_id,
            cname: cname.to_string(),
            id_name: None,
            title: None,
            hero_type: None,
            roles: vec![],
            moss_id: None,
            source: SourceInfo {
                url: "u".to_string(),
                fetched_at: "t".to_string(),
                content_hash: "h".to_string(),
            },
        }
    }
}
