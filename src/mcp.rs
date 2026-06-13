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
    let id = msg.get("id").cloned();
    let method = msg
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if id.is_none() {
        return Ok(None);
    }
    let id = id.unwrap();
    let result = match method {
        "initialize" => json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {"listChanged": false}},
            "serverInfo": {"name": "wzry-search-mcp", "version": env!("CARGO_PKG_VERSION")}
        }),
        "tools/list" => json!({"tools": tool_specs()}),
        "tools/call" => {
            let params = msg.get("params").cloned().unwrap_or_else(|| json!({}));
            let name = params
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("tools/call missing name"))?;
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
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
        "wzry_search_heroes" => {
            let query = required_str(args, "query")?;
            let limit = optional_usize(args, "limit").unwrap_or(10);
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
        "wzry_search_items" => {
            let query = required_str(args, "query")?;
            let limit = optional_usize(args, "limit").unwrap_or(10);
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

fn tool_specs() -> Vec<Value> {
    vec![
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
            "Get one hero skill; skill accepts passive/被动/1/2/3/大招.",
            json!({"type":"object","properties":{"hero":{"type":"string"},"skill":{"type":"string"}},"required":["hero","skill"]}),
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
    let mut header = Vec::new();
    let mut buf = [0_u8; 1];
    loop {
        match reader.read(&mut buf)? {
            0 if header.is_empty() => return Ok(None),
            0 => return Err(anyhow!("unexpected EOF while reading MCP headers")),
            _ => {
                header.push(buf[0]);
                if header.ends_with(b"\r\n\r\n") || header.ends_with(b"\n\n") {
                    break;
                }
            }
        }
    }
    let header_text = String::from_utf8_lossy(&header);
    let len = header_text
        .lines()
        .find_map(|line| {
            line.strip_prefix("Content-Length:")
                .or_else(|| line.strip_prefix("content-length:"))
        })
        .and_then(|s| s.trim().parse::<usize>().ok())
        .ok_or_else(|| anyhow!("missing Content-Length header"))?;
    let mut body = vec![0_u8; len];
    reader.read_exact(&mut body)?;
    Ok(Some(
        serde_json::from_slice(&body).context("parse MCP JSON body")?,
    ))
}

fn write_mcp_message<W: Write>(writer: &mut W, value: &Value) -> Result<()> {
    let body = serde_json::to_vec(value)?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()?;
    Ok(())
}
