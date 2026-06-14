# Update Detection

`check-updates` computes hashes for deterministic core JSON sources and compares them with the local `source_snapshots` table. It is a coarse change detector for list-level sources, not a replacement for periodic polite `sync`.

Hero detail pages contain skill text. Starting in v0.3, `sync-changed` can inspect official update-like news and refresh only locally known heroes mentioned in those articles. Periodic polite `sync` remains the conservative full-refresh mechanism.

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

## News-based affected hero sync

`sync-changed` is separate from deterministic snapshots. It reads the official news index, filters update-like article titles, fetches a bounded number of detail pages, detects locally known hero names in the article text, and refreshes only those hero detail pages.

Dry-run:

```bash
wzry-search-mcp --db ./wzry.sqlite sync-changed --news-limit 10 --dry-run
```

Actual affected-hero refresh:

```bash
wzry-search-mcp --db ./wzry.sqlite sync-changed --news-limit 10
```

The news index is not written to `source_snapshots`; this path records `update_events` for diagnostics only.

## Intended schedule

The actual database used by local agents should be updated by local cron/Hermes cron, not by GitHub Actions:

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

## GitHub source smoke

The scheduled GitHub workflow `.github/workflows/source-smoke.yml` is intentionally limited to a small upstream-shape smoke:

- build the binary;
- sync only a tiny hero-detail fixture with `--limit-heroes 2` and short polite delays;
- exercise CLI discovery commands;
- initialize MCP over stdio, list tools, and call representative discovery tools.

It does not update, commit, publish, or upload generated datasets. GitHub Actions is for remote health checks; local cron/Hermes cron is the data-maintenance path for the SQLite database that agents actually query.

Do not commit generated `*.sqlite`, JSON export, or CSV export files to the repo.
