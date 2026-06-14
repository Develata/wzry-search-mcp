use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
};

use serde_json::{Value, json};

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
fn stdio_initialize_ping_list_tools_and_unknown_tool_error() {
    let bin = env!("CARGO_BIN_EXE_wzry-search-mcp");
    let mut child = Command::new(bin)
        .args(["--db", "/tmp/wzry-search-mcp-stdio-test.sqlite", "serve"])
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
            "params": {"name": "not_a_tool", "arguments": {}}
        }),
    );
    let unknown = read_message(&mut stdout);
    assert_eq!(unknown["id"], 4);
    assert!(unknown.get("error").is_some());

    send(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {"name": "wzry_get_hero_profile", "arguments": {}}
        }),
    );
    let invalid_args = read_message(&mut stdout);
    assert_eq!(invalid_args["id"], 5);
    assert!(
        invalid_args["error"]["message"]
            .as_str()
            .unwrap()
            .contains("missing field `hero`")
    );

    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();
}
