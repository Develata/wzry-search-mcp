# Hermes MCP Configuration Example

Build or install the binary first:

```bash
cd /opt/gitclone/wzry-search-mcp
CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo build --release
install -Dm755 /tmp/hermes-cargo-target/release/wzry-search-mcp /opt/data/bin/wzry-search-mcp
```

Create the local dataset:

```bash
/opt/data/bin/wzry-search-mcp --db /opt/data/wzry-search-mcp/wzry.sqlite sync
```

Configure the MCP server in Hermes config:

```yaml
mcp_servers:
  wzry-search:
    enabled: true
    command: /opt/data/bin/wzry-search-mcp
    args:
      - --db
      - /opt/data/wzry-search-mcp/wzry.sqlite
      - serve
```

Then verify with:

```bash
hermes mcp test wzry-search
```

After changing MCP config, restart or reload MCP in the host session.

## Local cron update

For Develata's Hermes deployment, the tracked cron template includes a weekly no-agent job:

```text
wzry-search-mcp-weekly-sync
schedule: 20 4 * * 1
script: /opt/data/scripts/wzry-search-mcp-update.py
```

The job updates the local SQLite DB used by agents:

```text
/opt/data/wzry-search-mcp/wzry.sqlite
```

Normal unchanged sync is silent. If core sources changed, it sends a short Telegram notice after successful sync. GitHub Actions remains only a remote source-smoke check and does not maintain this local DB.

## Tool use guidance

For discovery before detailed queries:

```text
wzry_list_heroes({"limit": 30})
wzry_search_heroes({"query": "廉", "limit": 10})
wzry_search_hero_skills({"query": "护盾", "limit": 10})
wzry_list_items({"limit": 30})
wzry_get_summoner_skills({})
```

For a single hero:

```text
wzry_get_hero_profile({"hero":"廉颇"})
```

For model-side lineup recommendation:

```text
wzry_get_lineup_context({
  "allies": ["廉颇", "小乔"],
  "enemies": ["兰陵王"],
  "candidate_pool": ["孙尚香", "马可波罗", "后羿"]
})
```

The MCP returns evidence only. The model should reason from passive/skills and produce the recommendation itself.
