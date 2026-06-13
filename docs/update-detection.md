# Update Detection

`check-updates` computes hashes for core official sources and compares them with the local `source_snapshots` table.

Sources checked:

- `herolist`: <https://pvp.qq.com/web201605/js/herolist.json>
- `items`: <https://pvp.qq.com/web201605/js/item.json>
- `summoner`: <https://pvp.qq.com/web201605/js/summoner.json>
- `news_index`: <https://pvp.qq.com/web201706/newsindex.shtml>

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

Do not commit generated `*.sqlite`, JSON export, or CSV export files to the repo.
