# News-based Incremental Sync Feature

This feature lets maintainers refresh likely affected hero detail pages after official update/news posts mention specific heroes.

## User workflow

1. Keep a normal local SQLite dataset through full sync or scheduled weekly sync.
2. Run a dry-run analysis:

```bash
wzry-search-mcp --db ./wzry.sqlite sync-changed --news-limit 10 --dry-run
```

3. Inspect matched news titles and affected hero names.
4. Run the actual incremental sync:

```bash
wzry-search-mcp --db ./wzry.sqlite sync-changed --news-limit 10
```

## Expected output semantics

The command prints JSON with:

- `checked_articles`: number of update-like articles considered;
- `matched_articles`: official article titles/URLs and detected hero names;
- `affected_heroes`: unique local hero names and ids;
- `synced_heroes`: names refreshed in non-dry-run mode;
- `warnings`: detail fetch/parse warnings that did not abort the whole analysis.

## Non-goals

This feature does not infer patch note semantics. It only says “these local hero records may need refresh because an official update-like article mentions them”. The local hero detail page remains the canonical source for skill text.
