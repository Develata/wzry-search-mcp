use crate::db::Store;
use crate::model::LineupContext;
use anyhow::{Context, Result, anyhow};
use serde_json::{Value, json};
use std::io::{Read, Write};

pub fn serve_stdio(db_path: &str) -> Result<()> {
    let mut input = std::io::stdin().lock();
    let mut output = std::io::stdout().lock();
    while let Some(message) = read_mcp_message(&mut input)? {
        let response = handle_message(db_path, message)?;
        if let Some(resp) = response {
            write_mcp_message(&mut output, &resp)?;
        }
    }
    Ok(())
}

fn handle_message(db_path: &str, msg: Value) -> Result<Option<Value>> {
    let Some(id) = msg.get("id").cloned() else {
        return Ok(None);
    };
    let method = msg
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let result = match method {
        "initialize" => json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {"listChanged": false}},
            "serverInfo": {"name": "wzry-search-mcp", "version": env!("CARGO_PKG_VERSION")}
        }),
        "ping" => json!({}),
        "tools/list" => json!({"tools": tool_specs()}),
        "tools/call" => {
            let params = msg.get("params").cloned().unwrap_or_else(|| json!({}));
            let Some(name) = params.get("name").and_then(Value::as_str) else {
                return Ok(Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {"code": -32602, "message": "tools/call missing string params.name"}
                })));
            };
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            if !is_known_tool(name) {
                return Ok(Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {"code": -32602, "message": format!("unknown tool: {name}")}
                })));
            }
            return Ok(Some(
                json!({"jsonrpc": "2.0", "id": id, "result": call_tool(db_path, name, &args)}),
            ));
        }
        _ => {
            return Ok(Some(
                json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32601, "message": format!("method not found: {method}")}}),
            ));
        }
    };
    Ok(Some(json!({"jsonrpc": "2.0", "id": id, "result": result})))
}

fn call_tool(db_path: &str, name: &str, args: &Value) -> Value {
    match call_tool_inner(db_path, name, args) {
        Ok(value) => {
            json!({"content": [{"type": "text", "text": serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())}]})
        }
        Err(err) => {
            json!({"isError": true, "content": [{"type": "text", "text": format!("{err:#}")}]})
        }
    }
}

fn call_tool_inner(db_path: &str, name: &str, args: &Value) -> Result<Value> {
    let store = Store::open_existing(db_path)?;
    match name {
        "wzry_list_heroes" => {
            let limit = optional_usize(args, "limit").unwrap_or(200).min(500);
            Ok(serde_json::to_value(store.list_heroes(limit)?)?)
        }
        "wzry_search_heroes" => {
            let query = required_str(args, "query")?;
            let limit = optional_usize(args, "limit").unwrap_or(10).min(50);
            Ok(serde_json::to_value(store.search_heroes(query, limit)?)?)
        }
        "wzry_get_hero_profile" => {
            let hero = required_str(args, "hero")?;
            Ok(serde_json::to_value(store.get_hero_profile(hero)?)?)
        }
        "wzry_get_hero_profiles" => {
            let heroes = required_string_array(args, "heroes")?;
            let profiles = heroes
                .iter()
                .map(|h| store.get_hero_profile(h))
                .collect::<Result<Vec<_>>>()?;
            Ok(serde_json::to_value(profiles)?)
        }
        "wzry_get_hero_skill" => {
            let hero = required_str(args, "hero")?;
            let skill = required_str(args, "skill")?;
            Ok(serde_json::to_value(store.get_hero_skill(hero, skill)?)?)
        }
        "wzry_search_hero_skills" => {
            let query = required_str(args, "query")?;
            let limit = optional_usize(args, "limit").unwrap_or(10).min(50);
            Ok(serde_json::to_value(
                store.search_hero_skills(query, limit)?,
            )?)
        }
        "wzry_list_items" => {
            let limit = optional_usize(args, "limit").unwrap_or(200).min(500);
            let mut items = store.all_items()?;
            items.truncate(limit);
            Ok(serde_json::to_value(items)?)
        }
        "wzry_search_items" => {
            let query = required_str(args, "query")?;
            let limit = optional_usize(args, "limit").unwrap_or(10).min(50);
            Ok(serde_json::to_value(store.search_items(query, limit)?)?)
        }
        "wzry_get_item" => {
            let item = required_str(args, "item")?;
            Ok(serde_json::to_value(store.get_item(item)?)?)
        }
        "wzry_get_summoner_skills" => Ok(serde_json::to_value(store.get_summoner_skills()?)?),
        "wzry_get_summoner_skill" => {
            let skill = required_str(args, "skill")?;
            Ok(serde_json::to_value(store.get_summoner_skill(skill)?)?)
        }
        "wzry_get_lineup_context" => {
            let allies = optional_string_array(args, "allies")?;
            let enemies = optional_string_array(args, "enemies")?;
            let candidate_pool = optional_string_array(args, "candidate_pool")?;
            let ctx = LineupContext {
                allies: allies
                    .iter()
                    .map(|h| store.get_hero_profile(h))
                    .collect::<Result<Vec<_>>>()?,
                enemies: enemies
                    .iter()
                    .map(|h| store.get_hero_profile(h))
                    .collect::<Result<Vec<_>>>()?,
                candidate_pool: candidate_pool
                    .iter()
                    .map(|h| store.get_hero_profile(h))
                    .collect::<Result<Vec<_>>>()?,
                recommendation_should_be_done_by_model: true,
            };
            Ok(serde_json::to_value(ctx)?)
        }
        _ => Err(anyhow!("unknown tool: {name}")),
    }
}

fn is_known_tool(name: &str) -> bool {
    tool_specs()
        .iter()
        .any(|tool| tool["name"].as_str() == Some(name))
}

fn tool_specs() -> Vec<Value> {
    vec![
        tool(
            "wzry_list_heroes",
            "List local heroes so agents can discover valid hero names before detailed queries.",
            json!({"type":"object","properties":{"limit":{"type":"integer","minimum":1,"maximum":500}}}),
        ),
        tool(
            "wzry_search_heroes",
            "Search local hero candidates by name/id_name/title.",
            json!({"type":"object","properties":{"query":{"type":"string"},"limit":{"type":"integer","minimum":1,"maximum":50}},"required":["query"]}),
        ),
        tool(
            "wzry_get_hero_profile",
            "Get bound hero basic info plus passive and active skills.",
            json!({"type":"object","properties":{"hero":{"type":"string"}},"required":["hero"]}),
        ),
        tool(
            "wzry_get_hero_profiles",
            "Batch get complete hero profiles for lineup reasoning.",
            json!({"type":"object","properties":{"heroes":{"type":"array","items":{"type":"string"}}},"required":["heroes"]}),
        ),
        tool(
            "wzry_get_hero_skill",
            "Get one hero skill; skill accepts passive/被动/1/2/3/大招 or exact skill name.",
            json!({"type":"object","properties":{"hero":{"type":"string"},"skill":{"type":"string"}},"required":["hero","skill"]}),
        ),
        tool(
            "wzry_search_hero_skills",
            "Search across hero skill names/descriptions and return hero + skill hits.",
            json!({"type":"object","properties":{"query":{"type":"string"},"limit":{"type":"integer","minimum":1,"maximum":50}},"required":["query"]}),
        ),
        tool(
            "wzry_list_items",
            "List local items so agents can discover valid equipment names.",
            json!({"type":"object","properties":{"limit":{"type":"integer","minimum":1,"maximum":500}}}),
        ),
        tool(
            "wzry_search_items",
            "Search local item data.",
            json!({"type":"object","properties":{"query":{"type":"string"},"limit":{"type":"integer","minimum":1,"maximum":50}},"required":["query"]}),
        ),
        tool(
            "wzry_get_item",
            "Get one item by name.",
            json!({"type":"object","properties":{"item":{"type":"string"}},"required":["item"]}),
        ),
        tool(
            "wzry_get_summoner_skills",
            "List all summoner skills.",
            json!({"type":"object","properties":{}}),
        ),
        tool(
            "wzry_get_summoner_skill",
            "Get one summoner skill by name.",
            json!({"type":"object","properties":{"skill":{"type":"string"}},"required":["skill"]}),
        ),
        tool(
            "wzry_get_lineup_context",
            "Return allies/enemies/candidate hero profiles for model-side lineup recommendation. MCP does not score or choose lineups.",
            json!({"type":"object","properties":{"allies":{"type":"array","items":{"type":"string"}},"enemies":{"type":"array","items":{"type":"string"}},"candidate_pool":{"type":"array","items":{"type":"string"}}}}),
        ),
    ]
}

fn tool(name: &str, description: &str, input_schema: Value) -> Value {
    json!({"name": name, "description": description, "inputSchema": input_schema})
}

fn required_str<'a>(args: &'a Value, key: &str) -> Result<&'a str> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing string argument `{key}`"))
}

fn optional_usize(args: &Value, key: &str) -> Option<usize> {
    args.get(key).and_then(Value::as_u64).map(|x| x as usize)
}

fn required_string_array(args: &Value, key: &str) -> Result<Vec<String>> {
    optional_string_array(args, key)?
        .into_iter()
        .collect::<Vec<_>>()
        .pipe(Ok)
        .and_then(|v| {
            if v.is_empty() {
                Err(anyhow!("missing non-empty string array `{key}`"))
            } else {
                Ok(v)
            }
        })
}

fn optional_string_array(args: &Value, key: &str) -> Result<Vec<String>> {
    let Some(v) = args.get(key) else {
        return Ok(vec![]);
    };
    let arr = v
        .as_array()
        .ok_or_else(|| anyhow!("`{key}` must be an array"))?;
    arr.iter()
        .map(|x| {
            x.as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow!("`{key}` must contain strings"))
        })
        .collect()
}

trait Pipe: Sized {
    fn pipe<T>(self, f: impl FnOnce(Self) -> T) -> T {
        f(self)
    }
}
impl<T> Pipe for T {}

fn read_mcp_message<R: Read>(reader: &mut R) -> Result<Option<Value>> {
    loop {
        let Some(line) = read_line(reader)? else {
            return Ok(None);
        };
        let line = trim_line_ending(&line);
        if line.is_empty() {
            continue;
        }
        if is_header_line(line) {
            return read_content_length_message(reader, line);
        }
        return Ok(Some(
            serde_json::from_slice(line).context("parse newline-delimited MCP JSON body")?,
        ));
    }
}

fn read_line<R: Read>(reader: &mut R) -> Result<Option<Vec<u8>>> {
    let mut line = Vec::new();
    let mut buf = [0_u8; 1];
    loop {
        match reader.read(&mut buf)? {
            0 if line.is_empty() => return Ok(None),
            0 => return Ok(Some(line)),
            _ => {
                line.push(buf[0]);
                if buf[0] == b'\n' {
                    return Ok(Some(line));
                }
            }
        }
    }
}

fn trim_line_ending(mut line: &[u8]) -> &[u8] {
    if line.ends_with(b"\n") {
        line = &line[..line.len() - 1];
    }
    if line.ends_with(b"\r") {
        line = &line[..line.len() - 1];
    }
    line
}

fn split_header_line(line: &[u8]) -> Option<(&[u8], &[u8])> {
    let index = line.iter().position(|byte| *byte == b':')?;
    Some((&line[..index], &line[index + 1..]))
}

fn is_header_line(line: &[u8]) -> bool {
    let Some((name, _)) = split_header_line(line) else {
        return false;
    };
    !name.is_empty()
        && name
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
}

fn is_content_length_header(line: &[u8]) -> bool {
    let Some((name, _)) = split_header_line(line) else {
        return false;
    };
    name.eq_ignore_ascii_case(b"content-length")
}

fn parse_content_length(line: &[u8]) -> Option<usize> {
    std::str::from_utf8(line)
        .ok()?
        .split_once(':')?
        .1
        .trim()
        .parse::<usize>()
        .ok()
}

fn read_content_length_message<R: Read>(
    reader: &mut R,
    first_header: &[u8],
) -> Result<Option<Value>> {
    let mut len = parse_content_length(first_header);
    loop {
        let Some(line) = read_line(reader)? else {
            return Err(anyhow!("unexpected EOF while reading MCP headers"));
        };
        let line = trim_line_ending(&line);
        if line.is_empty() {
            break;
        }
        if is_content_length_header(line)
            && let Some(parsed) = parse_content_length(line)
        {
            len = Some(parsed);
        }
    }
    let len = len.ok_or_else(|| anyhow!("missing Content-Length header"))?;
    let mut body = vec![0_u8; len];
    reader.read_exact(&mut body)?;
    Ok(Some(
        serde_json::from_slice(&body).context("parse MCP JSON body")?,
    ))
}

fn write_mcp_message<W: Write>(writer: &mut W, value: &Value) -> Result<()> {
    serde_json::to_writer(&mut *writer, value)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{HeroBasic, HeroSkill, Item, SourceInfo, SummonerSkill};
    use tempfile::NamedTempFile;

    fn source(url: &str) -> SourceInfo {
        SourceInfo {
            url: url.to_string(),
            fetched_at: "2026-01-01T00:00:00Z".to_string(),
            content_hash: "hash".to_string(),
        }
    }

    fn fixture_db() -> (NamedTempFile, String) {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_string_lossy().to_string();
        let mut store = Store::open(&path).unwrap();
        let hero = HeroBasic {
            hero_id: 105,
            ename: 105,
            cname: "廉颇".to_string(),
            id_name: Some("lianpo".to_string()),
            title: Some("正义爆轰".to_string()),
            hero_type: Some(3),
            roles: vec!["坦克".to_string()],
            moss_id: Some(3627),
            source: source("https://pvp.qq.com/web201605/js/herolist.json"),
        };
        store.upsert_hero(&hero).unwrap();
        let skills = vec![
            HeroSkill {
                hero_id: 105,
                slot: "passive".to_string(),
                name: "勇士之魂".to_string(),
                cooldown: Some("0".to_string()),
                cost: Some("0".to_string()),
                description: "被动描述".to_string(),
                source: source("https://pvp.qq.com/web201605/herodetail/105.shtml"),
            },
            HeroSkill {
                hero_id: 105,
                slot: "skill_1".to_string(),
                name: "爆裂冲撞".to_string(),
                cooldown: Some("9".to_string()),
                cost: Some("0".to_string()),
                description: "一技能描述".to_string(),
                source: source("https://pvp.qq.com/web201605/herodetail/105.shtml"),
            },
        ];
        store.replace_hero_skills(105, &skills, &[]).unwrap();
        store
            .upsert_item(&Item {
                item_id: 1136,
                item_name: "破军".to_string(),
                item_type: Some(1),
                price: Some(1770),
                total_price: Some(2950),
                description_html: Some("<p>+180物理攻击</p>".to_string()),
                description_text: Some("+180物理攻击".to_string()),
                source: source("https://pvp.qq.com/web201605/js/item.json"),
            })
            .unwrap();
        store
            .upsert_summoner_skill(&SummonerSkill {
                skill_id: 80115,
                name: "闪现".to_string(),
                rank: Some(13),
                description: Some("向指定方向位移".to_string()),
                source: source("https://pvp.qq.com/web201605/js/summoner.json"),
            })
            .unwrap();
        drop(store);
        (file, path)
    }

    #[test]
    fn tool_specs_include_lineup_context() {
        let names = tool_specs()
            .into_iter()
            .map(|t| t["name"].as_str().unwrap().to_string())
            .collect::<Vec<_>>();
        assert!(names.contains(&"wzry_list_heroes".to_string()));
        assert!(names.contains(&"wzry_search_hero_skills".to_string()));
        assert!(names.contains(&"wzry_list_items".to_string()));
        assert!(names.contains(&"wzry_get_lineup_context".to_string()));
        assert!(names.contains(&"wzry_get_hero_profile".to_string()));
    }

    #[test]
    fn tool_call_returns_hero_profile_and_lineup_context() {
        let (_file, path) = fixture_db();
        let profile =
            call_tool_inner(&path, "wzry_get_hero_profile", &json!({"hero":"廉颇"})).unwrap();
        assert_eq!(profile["hero"]["cname"], "廉颇");
        assert_eq!(profile["skills"][0]["slot"], "passive");

        let ctx = call_tool_inner(
            &path,
            "wzry_get_lineup_context",
            &json!({"allies":["廉颇"], "candidate_pool":["廉颇"]}),
        )
        .unwrap();
        assert_eq!(ctx["recommendation_should_be_done_by_model"], true);
        assert_eq!(ctx["allies"][0]["hero"]["cname"], "廉颇");
    }

    #[test]
    fn discovery_tools_return_lists_and_skill_search_hits() {
        let (_file, path) = fixture_db();
        let heroes = call_tool_inner(&path, "wzry_list_heroes", &json!({"limit":5})).unwrap();
        assert_eq!(heroes.as_array().unwrap().len(), 1);
        assert_eq!(heroes[0]["cname"], "廉颇");

        let skills = call_tool_inner(
            &path,
            "wzry_search_hero_skills",
            &json!({"query":"冲撞", "limit":5}),
        )
        .unwrap();
        assert_eq!(skills[0]["hero"]["cname"], "廉颇");
        assert_eq!(skills[0]["skill"]["name"], "爆裂冲撞");

        let items = call_tool_inner(&path, "wzry_list_items", &json!({"limit":5})).unwrap();
        assert_eq!(items.as_array().unwrap().len(), 1);
        assert_eq!(items[0]["item_name"], "破军");
    }

    #[test]
    fn reads_newline_delimited_and_content_length_messages() {
        let mut newline = br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
"#
        .as_slice();
        let msg = read_mcp_message(&mut newline).unwrap().unwrap();
        assert_eq!(msg["method"], "initialize");

        let body = br#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
        let mut framed = format!(
            "Content-Type: application/vscode-jsonrpc; charset=utf-8\r\nContent-Length: {}\r\n\r\n",
            body.len()
        )
        .into_bytes();
        framed.extend_from_slice(body);
        let msg = read_mcp_message(&mut framed.as_slice()).unwrap().unwrap();
        assert_eq!(msg["method"], "tools/list");
    }

    #[test]
    fn reads_multiple_sequential_messages() {
        let mut newline = br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
"#
        .as_slice();
        let first = read_mcp_message(&mut newline).unwrap().unwrap();
        let second = read_mcp_message(&mut newline).unwrap().unwrap();
        assert_eq!(first["id"], 1);
        assert_eq!(second["id"], 2);

        let body = br#"{"jsonrpc":"2.0","id":3,"method":"ping"}"#;
        let mut mixed = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
        mixed.extend_from_slice(body);
        mixed.extend_from_slice(
            b"{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/list\",\"params\":{}}\n",
        );
        let mut mixed = mixed.as_slice();
        let first = read_mcp_message(&mut mixed).unwrap().unwrap();
        let second = read_mcp_message(&mut mixed).unwrap().unwrap();
        assert_eq!(first["method"], "ping");
        assert_eq!(second["method"], "tools/list");
    }

    #[test]
    fn writes_newline_delimited_message() {
        let mut out = Vec::new();
        write_mcp_message(&mut out, &json!({"jsonrpc":"2.0", "id":1, "result":{}})).unwrap();
        assert!(out.ends_with(b"\n"));
        let parsed: Value = serde_json::from_slice(trim_line_ending(&out)).unwrap();
        assert_eq!(parsed["id"], 1);
        assert!(
            !String::from_utf8(out)
                .unwrap()
                .starts_with("Content-Length:")
        );
    }

    #[test]
    fn ping_returns_empty_result() {
        let response = handle_message(
            "/tmp/not-needed.sqlite",
            json!({"jsonrpc":"2.0", "id": 9, "method":"ping"}),
        )
        .unwrap()
        .unwrap();
        assert_eq!(response["id"], 9);
        assert_eq!(response["result"], json!({}));
    }

    #[test]
    fn malformed_tools_call_returns_json_rpc_error() {
        let (_file, path) = fixture_db();
        let response = handle_message(
            &path,
            json!({"jsonrpc":"2.0", "id": 7, "method":"tools/call", "params": {}}),
        )
        .unwrap()
        .unwrap();
        assert_eq!(response["id"], 7);
        assert_eq!(response["error"]["code"], -32602);
    }

    #[test]
    fn unknown_tool_returns_json_rpc_error() {
        let (_file, path) = fixture_db();
        let response = handle_message(
            &path,
            json!({"jsonrpc":"2.0", "id": 8, "method":"tools/call", "params": {"name":"unknown_tool", "arguments": {}}}),
        )
        .unwrap()
        .unwrap();
        assert_eq!(response["id"], 8);
        assert_eq!(response["error"]["code"], -32602);
    }
}
