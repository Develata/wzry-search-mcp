use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
};

use rusqlite::Connection;
use serde_json::{Value, json};
use tempfile::NamedTempFile;

fn fixture_db() -> NamedTempFile {
    let file = NamedTempFile::new().unwrap();
    let conn = Connection::open(file.path()).unwrap();
    conn.execute_batch(
        r#"
        CREATE TABLE heroes (
          hero_id INTEGER PRIMARY KEY,
          ename INTEGER NOT NULL UNIQUE,
          cname TEXT NOT NULL,
          id_name TEXT,
          title TEXT,
          hero_type INTEGER,
          roles_json TEXT NOT NULL,
          moss_id INTEGER,
          source_url TEXT NOT NULL,
          fetched_at TEXT NOT NULL,
          content_hash TEXT NOT NULL
        );
        CREATE TABLE hero_skills (
          hero_id INTEGER NOT NULL,
          slot TEXT NOT NULL,
          name TEXT NOT NULL,
          cooldown TEXT,
          cost TEXT,
          description TEXT NOT NULL,
          source_url TEXT NOT NULL,
          fetched_at TEXT NOT NULL,
          content_hash TEXT NOT NULL,
          PRIMARY KEY(hero_id, slot)
        );
        CREATE TABLE hero_parse_warnings (
          hero_id INTEGER NOT NULL,
          warning TEXT NOT NULL,
          fetched_at TEXT NOT NULL
        );
        CREATE TABLE items (
          item_id INTEGER PRIMARY KEY,
          item_name TEXT NOT NULL,
          item_type INTEGER,
          price INTEGER,
          total_price INTEGER,
          description_html TEXT,
          description_text TEXT,
          source_url TEXT NOT NULL,
          fetched_at TEXT NOT NULL,
          content_hash TEXT NOT NULL
        );
        CREATE TABLE summoner_skills (
          skill_id INTEGER PRIMARY KEY,
          name TEXT NOT NULL,
          rank INTEGER,
          description TEXT,
          source_url TEXT NOT NULL,
          fetched_at TEXT NOT NULL,
          content_hash TEXT NOT NULL
        );
        CREATE TABLE source_snapshots (
          source_key TEXT PRIMARY KEY,
          url TEXT NOT NULL,
          fetched_at TEXT NOT NULL,
          content_hash TEXT NOT NULL,
          byte_len INTEGER NOT NULL
        );
        CREATE TABLE update_events (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          event_type TEXT NOT NULL,
          source_key TEXT,
          message TEXT NOT NULL,
          created_at TEXT NOT NULL
        );
        INSERT INTO heroes VALUES (
          105, 105, '廉颇', 'lianpo', '正义爆轰', 3, '["坦克"]', 3627,
          'https://pvp.qq.com/web201605/js/herolist.json', '2026-01-01T00:00:00Z', 'hash'
        );
        INSERT INTO hero_skills VALUES (
          105, 'passive', '勇士之魂', '0', '0', '被动描述',
          'https://pvp.qq.com/web201605/herodetail/105.shtml', '2026-01-01T00:00:00Z', 'hash'
        );
        "#,
    )
    .unwrap();
    drop(conn);
    file
}

fn send(stdin: &mut std::process::ChildStdin, value: Value) {
    writeln!(stdin, "{}", serde_json::to_string(&value).unwrap()).unwrap();
    stdin.flush().unwrap();
}

fn read_message(stdout: &mut BufReader<std::process::ChildStdout>) -> Value {
    let mut line = String::new();
    stdout.read_line(&mut line).unwrap();
    assert!(!line.trim().is_empty(), "empty MCP stdout line");
    serde_json::from_str(line.trim_end()).unwrap()
}

#[test]
fn stdio_initialize_ping_list_tools_and_tool_calls() {
    let db = fixture_db();
    let bin = env!("CARGO_BIN_EXE_wzry-search-mcp");
    let mut child = Command::new(bin)
        .args(["--db", db.path().to_str().unwrap(), "serve"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    send(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "stdio-test", "version": "0.0.0"}
            }
        }),
    );
    let init = read_message(&mut stdout);
    assert_eq!(init["id"], 1);
    assert_eq!(init["result"]["serverInfo"]["name"], "wzry-search-mcp");
    assert_eq!(
        init["result"]["serverInfo"]["version"],
        env!("CARGO_PKG_VERSION")
    );

    send(
        &mut stdin,
        json!({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}}),
    );

    send(
        &mut stdin,
        json!({"jsonrpc": "2.0", "id": 2, "method": "ping"}),
    );
    let ping = read_message(&mut stdout);
    assert_eq!(ping["id"], 2);
    assert_eq!(ping["result"], json!({}));

    send(
        &mut stdin,
        json!({"jsonrpc": "2.0", "id": 3, "method": "tools/list", "params": {}}),
    );
    let tools = read_message(&mut stdout);
    assert_eq!(tools["id"], 3);
    let tools_array = tools["result"]["tools"].as_array().unwrap();
    assert_eq!(tools_array.len(), 12);
    let profile = tools_array
        .iter()
        .find(|tool| tool["name"] == "wzry_get_hero_profile")
        .unwrap();
    assert!(profile.get("outputSchema").is_some());

    send(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {"name": "wzry_get_hero_profile", "arguments": {"hero": "廉颇"}}
        }),
    );
    let success = read_message(&mut stdout);
    assert_eq!(success["id"], 4);
    assert_eq!(success["result"]["isError"], false);
    assert_eq!(
        success["result"]["structuredContent"]["hero"]["cname"],
        "廉颇"
    );
    assert!(
        success["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("廉颇")
    );

    send(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {"name": "wzry_get_hero_profile", "arguments": {"hero": "不存在英雄"}}
        }),
    );
    let domain_error = read_message(&mut stdout);
    assert_eq!(domain_error["id"], 5);
    assert_eq!(domain_error["result"]["isError"], true);
    assert!(domain_error["result"].get("structuredContent").is_none());
    assert!(
        domain_error["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("hero not found")
    );

    send(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": {"name": "not_a_tool", "arguments": {}}
        }),
    );
    let unknown = read_message(&mut stdout);
    assert_eq!(unknown["id"], 6);
    assert!(unknown.get("error").is_some());

    send(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "tools/call",
            "params": {"name": "wzry_get_hero_profile", "arguments": {}}
        }),
    );
    let invalid_args = read_message(&mut stdout);
    assert_eq!(invalid_args["id"], 7);
    assert!(
        invalid_args["error"]["message"]
            .as_str()
            .unwrap()
            .contains("missing field `hero`")
    );

    send(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 8,
            "method": "tools/call",
            "params": {"name": "wzry_get_hero_profiles", "arguments": {"heroes": []}}
        }),
    );
    let empty_batch = read_message(&mut stdout);
    assert_eq!(empty_batch["id"], 8);
    assert!(
        empty_batch["error"]["message"]
            .as_str()
            .unwrap()
            .contains("missing non-empty string array `heroes`")
    );

    send(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 9,
            "method": "tools/call",
            "params": {"name": "wzry_get_hero_profiles", "arguments": {"heroes": [" "]}}
        }),
    );
    let blank_batch = read_message(&mut stdout);
    assert_eq!(blank_batch["id"], 9);
    assert!(
        blank_batch["error"]["message"]
            .as_str()
            .unwrap()
            .contains("missing non-empty string array `heroes`")
    );

    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();
}
