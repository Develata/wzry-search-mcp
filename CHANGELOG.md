# Changelog

## v0.3.0 — 2026-06-15

### Added

- Added CLI `sync-changed` for news-based affected-hero incremental sync.
  - Reads the official news index first page.
  - Filters update-like article titles.
  - Fetches a bounded number of article detail pages.
  - Detects locally known hero names mentioned in title/detail text.
  - Refreshes only affected hero detail pages in non-dry-run mode.
  - Supports `--dry-run`, `--news-limit`, and polite delay flags.
- Added docs-as-code contracts for the v0.3.0 feature:
  - `docs/plan/06_news_incremental_sync.md`
  - `docs/features/02_news_incremental_sync.md`
  - `docs/acceptance-cases/02_news_incremental_sync.md`

### Changed

- Updated project version to `0.3.0`.
- Updated README install URLs and expected version output to `v0.3.0`.
- Documented manual update commands and an external cron schedule for `sync-changed`, `check-updates`, and polite full `sync`.
- Updated `SPEC.md`, `docs/update-detection.md`, and `docs/coverage-matrix.md` for `sync-changed`.
- Updated crawler user-agent to `wzry-search-mcp/0.3`.

### Fixed / Hardened

- Added boundary-aware matching for single-character hero names such as `镜`, reducing false-positive incremental sync matches.
- Added polite delay between fetched news article detail pages when `sync-changed` runs in polite mode.
- Kept MCP tool contract unchanged from v0.2.0.

### Verification

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`
- `cargo build --all-features`
- CLI smoke: `sync-changed --dry-run` preserves hero skill row count.
- CLI smoke: non-dry-run `sync-changed` refreshes affected hero detail pages.
- Codex + Reasonix review: final blocker-only pass.

## v0.2.0 — 2026-06-14

### Added

- Migrated MCP server layer to official Rust MCP SDK / RMCP.
- Added structured output support:
  - `structuredContent`
  - JSON text fallback in `content`
  - `outputSchema` in `tools/list`

### Changed

- Preserved public MCP tool names and input semantics while moving protocol ownership to RMCP.
- Array-like tool outputs use object roots such as `{ "heroes": [...] }` for schema compatibility.

### Fixed / Hardened

- Domain lookup failures return MCP tool errors instead of fabricated empty success payloads.
- Stdio integration tests cover initialize, ping, tools/list, successful tool call, domain error, malformed args, and unknown tools.

## v0.1.1 — 2026-06-14

### Fixed

- Aligned stdio MCP transport behavior with standard newline-delimited JSON-RPC clients.
- Hardened AstrBot/Python MCP SDK compatibility.

## v0.1.0 — 2026-06-13

### Added

- Initial local SQLite factual dataset for 王者荣耀 public official sources.
- Hero profile, hero skill, item, summoner skill, update detection, export, and MCP query support.
- Linux x86_64 release artifact workflow.
