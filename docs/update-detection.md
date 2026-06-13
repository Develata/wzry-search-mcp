# Update Detection

`check-updates` computes hashes for deterministic core JSON sources and compares them with the local `source_snapshots` table. It is a coarse change detector for list-level sources, not a replacement for periodic polite `sync`.

Hero detail pages contain skill text. In v0.1, skill-only text changes are caught by running `sync`, not by `check-updates`.

Sources checked:

- `herolist`: <https://pvp.qq.com/web201605/js/herolist.json>
- `items`: <https://pvp.qq.com/web201605/js/item.json>
- `summoner`: <https://pvp.qq.com/web201605/js/summoner.json>

The news index at <https://pvp.qq.com/web201706/newsindex.shtml> is intentionally not part of deterministic snapshots because its dynamic markup can change between immediate checks.

## Check without writing

```bash
wzry-search-mcp --db ./wzry.sqlite check-updates
```

## Persist snapshots

```bash
wzry-search-mcp --db ./wzry.sqlite check-updates --write-snapshots
```

When a source hash differs from the stored snapshot, the command records a `source_changed` event if `--write-snapshots` is set.

## Intended schedule

A weekly cron/GitHub Action can run:

```bash
wzry-search-mcp --db ./wzry.sqlite check-updates --write-snapshots
```

If changed, follow with a polite sync:

```bash
wzry-search-mcp --db ./wzry.sqlite sync
```

For local smoke tests only, use shorter random delays:

```bash
wzry-search-mcp --db /tmp/wzry-smoke.sqlite sync --min-delay-ms 200 --max-delay-ms 500
```

For a fast MCP fixture containing only the first few heroes:

```bash
wzry-search-mcp --db /tmp/wzry-mcp-smoke.sqlite sync --no-polite --limit-heroes 2
```

Do not use `--no-polite` for normal full sync against public sources.

Do not commit generated `*.sqlite`, JSON export, or CSV export files to the repo.
