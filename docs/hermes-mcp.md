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

## Tool use guidance

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
