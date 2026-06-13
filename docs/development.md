# Development and Verification

This document is the local pre-review checklist for `wzry-search-mcp`.

## Disk preflight

Before local Rust builds:

```bash
df -h / /opt/data /opt/gitclone 2>/dev/null || df -h
du -sh /usr/local/cargo /usr/local/rustup /tmp/hermes-cargo-target target 2>/dev/null || true
```

Use a temporary target dir in this Hermes environment:

```bash
export CARGO_TARGET_DIR=/tmp/hermes-cargo-target
```

Clean it after verification:

```bash
rm -rf /tmp/hermes-cargo-target
```

## Local validation gate

```bash
CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo fmt --all -- --check
CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo clippy --all-targets --all-features -- -D warnings
CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo test --all-features
CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo build --all-features
```

## Full data smoke

Use short polite delay only for smoke testing; default sync remains slower and more conservative.

```bash
rm -f /tmp/wzry-full-smoke.sqlite /tmp/wzry-full-smoke.sqlite-*
CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo run --quiet -- \
  --db /tmp/wzry-full-smoke.sqlite \
  sync --min-delay-ms 200 --max-delay-ms 500
```

Expected current smoke shape:

```text
heroes 130
skills 528
items 115
summoner 16
warnings 0
```

## Export smoke

```bash
rm -rf /tmp/wzry-export-smoke
mkdir -p /tmp/wzry-export-smoke
CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo run --quiet -- \
  --db /tmp/wzry-full-smoke.sqlite \
  export --format json --out /tmp/wzry-export-smoke/wzry.json
CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo run --quiet -- \
  --db /tmp/wzry-full-smoke.sqlite \
  export --format csv --out /tmp/wzry-export-smoke/csv
```

Expected row counts include CSV headers:

```text
json heroes/items/summoner: 130/115/16
heroes.csv: 131
hero_skills.csv: 529
items.csv: 116
summoner_skills.csv: 17
```

## Update detection smoke

```bash
rm -f /tmp/wzry-update-smoke.sqlite /tmp/wzry-update-smoke.sqlite-*
CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo run --quiet -- \
  --db /tmp/wzry-update-smoke.sqlite \
  check-updates --write-snapshots
CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo run --quiet -- \
  --db /tmp/wzry-update-smoke.sqlite \
  check-updates
```

Expected: first run changed, second run unchanged, three snapshots stored.

## MCP stdio smoke

Start from a small database:

```bash
rm -f /tmp/wzry-mcp-smoke.sqlite /tmp/wzry-mcp-smoke.sqlite-*
CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo run --quiet -- \
  --db /tmp/wzry-mcp-smoke.sqlite \
  sync --no-polite --limit-heroes 2
```

Then send JSON-RPC initialize, tools/list, and `wzry_get_lineup_context` frames. Expected:

```text
serverInfo.name = wzry-search-mcp
tools = 12
recommendation_should_be_done_by_model = true
```

## GitHub source smoke

`.github/workflows/source-smoke.yml` is a scheduled/dispatchable remote health check, not a data updater. It runs a tiny `--limit-heroes 2` sync plus CLI/MCP discovery calls and should not commit or upload generated datasets.

## Pre-review packet

Before Codex + Reasonix review, include:

- `git status --short --branch`
- `git log --oneline --decorate -8`
- `git diff --stat HEAD~N..HEAD` for the review scope
- `SPEC.md`
- README/docs summaries
- relevant source excerpts or full files
- real validation output
- full sync/export/update/MCP smoke output

Do not push before review unless Develata explicitly asks.
