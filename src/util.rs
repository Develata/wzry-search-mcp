use anyhow::Result;
use chrono::Utc;
use sha2::{Digest, Sha256};

pub const HERO_LIST_URL: &str = "https://pvp.qq.com/web201605/js/herolist.json";
pub const ITEM_LIST_URL: &str = "https://pvp.qq.com/web201605/js/item.json";
pub const SUMMONER_JSON_URL: &str = "https://pvp.qq.com/web201605/js/summoner.json";
#[allow(dead_code)]
pub const SUMMONER_PAGE_URL: &str = "https://pvp.qq.com/web201605/summoner.shtml";
// News index pages are intentionally not part of deterministic source snapshots;
// their dynamic markup can change between immediate checks.

pub fn hero_detail_url(hero_id: i64) -> String {
    format!("https://pvp.qq.com/web201605/herodetail/{hero_id}.shtml")
}

pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn text_sha256_hex(text: &str) -> String {
    sha256_hex(text.as_bytes())
}

pub fn decode_response(bytes: &[u8], url: &str) -> Result<String> {
    if let Ok(s) = String::from_utf8(bytes.to_vec()) {
        return Ok(s);
    }
    let (cow, _, _) = encoding_rs::GBK.decode(bytes);
    let s = cow.into_owned();
    if s.is_empty() {
        anyhow::bail!("decode response from {url}: empty decoded text");
    }
    Ok(s)
}

pub fn strip_html_to_text(html: &str) -> String {
    let text = html2text::from_read(html.as_bytes(), 120).unwrap_or_else(|_| html.to_string());
    normalize_ws(&text)
}

pub fn normalize_ws(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}
