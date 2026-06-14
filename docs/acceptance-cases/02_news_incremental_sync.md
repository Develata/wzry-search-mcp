# News Incremental Sync Acceptance Case

This case verifies v0.3.0 news-based affected-hero incremental sync.

## Parser/unit evidence

Automated tests should prove:

- official-style news index HTML yields entries with title and URL;
- update-like titles are selected;
- local hero names mentioned in title/detail text are detected;
- non-update or activity-only titles do not force a sync.

## CLI dry-run smoke

```bash
rm -f /tmp/wzry-news-smoke.sqlite /tmp/wzry-news-smoke.sqlite-*
CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo run --quiet -- \
  --db /tmp/wzry-news-smoke.sqlite \
  sync --min-delay-ms 200 --max-delay-ms 500 --limit-heroes 2
CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo run --quiet -- \
  --db /tmp/wzry-news-smoke.sqlite \
  sync-changed --news-limit 5 --dry-run
```

Acceptance criteria:

- output is valid JSON;
- output includes `checked_articles`, `matched_articles`, `affected_heroes`, `synced_heroes`, and `warnings`;
- dry-run does not mutate hero skill rows.

## Full command smoke

When the dry-run finds affected heroes that exist in the local DB:

```bash
CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo run --quiet -- \
  --db /tmp/wzry-news-smoke.sqlite \
  sync-changed --news-limit 5 --min-delay-ms 200 --max-delay-ms 500
```

Acceptance criteria:

- command exits successfully;
- affected hero detail pages are refreshed;
- update events record the news incremental path;
- unrelated heroes are not resynced by this command.
