# v0.3.0 News-based Incremental Sync

Status: planned / active

## Objective

Add a conservative news-based incremental sync path that can inspect official news titles/details and refresh only affected hero detail pages.

This is the `SPEC.md` future work item: news-based affected-hero incremental sync.

## Scope

Included:

- parse official news index first page entries from <https://pvp.qq.com/web201706/newsindex.shtml>;
- filter update-like news by title keywords;
- fetch a bounded number of matching news detail pages;
- detect affected heroes by matching local hero names in title/detail text;
- optionally sync only those heroes' detail pages;
- expose this through CLI, not MCP.

Excluded:

- no model-side summary;
- no recommendation/tier logic;
- no skin/media/rune sync;
- no news source deterministic snapshot, because the page can be dynamic;
- no MCP tool change in v0.3.0.

## Public CLI contract

New command:

```bash
wzry-search-mcp --db ./wzry.sqlite sync-changed --news-limit 10 --dry-run
wzry-search-mcp --db ./wzry.sqlite sync-changed --news-limit 10 --min-delay-ms 300 --max-delay-ms 800
```

Behavior:

- Requires an existing local DB with heroes already synced.
- `--dry-run` prints affected hero candidates and matched news entries without writing hero skill data.
- Non-dry-run refreshes detail pages for the affected hero set only.
- The command writes `update_events` for the analysis/sync path but does not update deterministic `source_snapshots` for news pages.

## Failure semantics

- Missing local DB: actionable error to run full `sync` first.
- Empty local hero list: actionable error to run full `sync` first.
- News page fetch failure: command fails; no partial sync claim.
- Article detail fetch failure: report a warning in the output and continue analysis for other articles.
- Zero affected heroes: successful no-op.

## Verification

- parser unit tests for news index entries and hero-name detection.
- CLI smoke with `--dry-run` against a small synced DB.
- Rust gate: fmt, clippy, tests, build.
- Existing MCP stdio tests must remain green, proving MCP contract unchanged.
