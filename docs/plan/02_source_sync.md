# Source Sync Contract

Source sync builds a local SQLite dataset from public official 王者荣耀 pages and JSON files. The sync path is allowed to access the network; MCP tools are not.

## Current source set

- Hero list JSON: <https://pvp.qq.com/web201605/js/herolist.json>
- Hero detail pages:
  - primary: `https://pvp.qq.com/web201605/herodetail/{hero_id}.shtml`
  - fallback: `https://pvp.qq.com/web201605/herodetail/{id_name}.shtml`
- Item JSON: <https://pvp.qq.com/web201605/js/item.json>
- Summoner skill JSON: <https://pvp.qq.com/web201605/js/summoner.json>

## Sync pipeline

```text
fetch -> decode -> parse -> validate -> replace/upsert -> query/export/MCP
```

Layer ownership:

- `src/crawler.rs`: HTTP fetching, polite delay, retry boundary, hash snapshots, orchestration.
- `src/parser.rs`: source text to typed model objects.
- `src/db/write.rs`: transactions, replacement, retention, update events.
- `src/db/query.rs`: local query only.

## Invariants

1. Default full sync is polite; short/no-polite modes are for smoke tests or controlled local runs.
2. Source hash changes are recorded before they become assumptions in parser code.
3. Detail parse anomalies must not silently replace good existing skill data.
4. News/announcement pages are not deterministic canonical sync sources unless a stable machine-readable endpoint is established.
5. MCP tools never perform real-time network calls; they only read the local database.

## Update detection

`check-updates` is a CLI capability. It checks deterministic list JSON sources and writes source snapshots when requested. It is intentionally separate from MCP tools so that agent queries remain local, fast, and side-effect-light.

## Sync-update orchestration

`sync-update` is the v0.4 sync-update one-shot command. It exists to make manual and scheduled maintenance use one stable entrypoint instead of copy-pasting multiple subcommands.

Contract:

1. Open an existing SQLite database; first-time dataset creation remains `sync`.
2. Acquire an update lock by default at `<db>.sync-update.lock`; fail fast if another update is already running unless `--lock-timeout-ms` is set.
3. Run deterministic source checking. Unchanged source snapshots may be refreshed after a successful non-dry-run update; changed snapshots must not be advanced unless the matching full data refresh succeeds.
4. Run news-based incremental affected-hero sync with bounded `--news-limit`.
5. Emit a compact human summary by default or a machine-readable JSON result with `--json`.
6. Do not run full sync by default. Only run full sync when deterministic source hashes changed and `--fallback-full` is explicitly passed.
7. `--dry-run` must not refresh hero details and must not run full sync.

Non-goals:

- no binary self-update;
- no AstrBot config mutation;
- no service restart;
- no long-running daemon or embedded scheduler.
