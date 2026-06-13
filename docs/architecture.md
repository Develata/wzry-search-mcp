# Architecture

`wzry-search-mcp` follows a small layered design. The goal is to keep the backbone clean: source adapters, parsers, storage, query tools, and CLI/MCP shells stay separated.

This document applies Develata's engineering constitution to this project: define the skeleton first, keep external dependencies behind adapter boundaries, prefer module replacement over main-flow rewrites, and avoid letting first-version features define the system ontology.

Source: <https://develata.me/knowledge/sharing/Awesome-Ai/prompts/%E5%B7%A5%E7%A8%8B%E5%AE%AA%E6%B3%95>

## Core ontology

The project's stable objects are:

- `HeroBasic`
- `HeroSkill`
- `HeroProfile = HeroBasic + skills + parse_warnings`
- `Item`
- `SummonerSkill`
- `SourceInfo`
- `SourceSnapshot`
- `UpdateEvent`
- `LineupContext`

Skins, images, image URLs, runes, and subjective tier lists are intentionally outside the core ontology.

## Layers

### 1. Shell layer

Files:

- `src/main.rs`
- `src/mcp.rs`

Responsibilities:

- CLI argument parsing.
- JSON-RPC/MCP stdio framing.
- User-facing command routing.
- Text/JSON presentation.

Non-responsibilities:

- HTML parsing.
- Source-specific transformation.
- SQLite schema ownership.
- Lineup recommendation logic.

### 2. Coordination layer

File:

- `src/crawler.rs`

Responsibilities:

- HTTP fetching.
- Polite delay / retry boundary.
- Source hash snapshots.
- Sync orchestration: fetch -> decode -> parse -> store.

Non-responsibilities:

- Data model definitions.
- DOM parsing details.
- Query tool semantics.

### 3. Capability modules

Files:

- `src/parser.rs`
- `src/export.rs`

Responsibilities:

- `parser.rs`: transform official JSON/HTML text into typed model objects.
- `export.rs`: transform local typed objects into JSON/CSV export formats.

These are intended to be replaceable modules under stable interfaces. If another official source shape appears, add or replace parser functions instead of rewriting the sync backbone.

### 4. Storage/query adapter

Files:

- `src/db.rs` — module root and `Store` construction.
- `src/db/schema.rs` — SQLite schema migration.
- `src/db/write.rs` — upsert, replace, retention, snapshots, update events.
- `src/db/query.rs` — local search, resolution, and profile/item/summoner retrieval.
- `src/db/rows.rs` — row-to-model mapping helpers.

Responsibilities:

- SQLite schema.
- Upsert/replace transactions.
- Local query and resolution.
- Converting rows to typed models.

Non-responsibilities:

- HTTP fetching.
- DOM parsing.
- MCP JSON-RPC framing.

### 5. Object and utility layer

Files:

- `src/model.rs`
- `src/util.rs`

Responsibilities:

- Stable typed objects.
- Source URLs and small pure utilities.
- Hashing, decoding, text normalization.

## Design invariants

1. Hero profile queries must bind hero basic info and skills together.
2. Passive skills are first-class skills with slot `passive`.
3. Extra skill/form entries use `extra_n` rather than ad-hoc fields.
4. MCP tools query the local database only; they must not perform real-time network calls.
5. Lineup recommendation is model-side; MCP returns evidence only.
6. Generated datasets are artifacts, not source files.
7. External official source details enter through crawler/parser boundaries, not through storage/query or MCP layers.

## Extension rules

- New official data source shape: add parser capability and source adapter logic; do not change core model unless the ontology truly changes.
- New export format: extend `export.rs`; do not alter crawler or MCP.
- New query tool: prefer composing existing `Store` methods; avoid network calls in `mcp.rs`.
- New subjective feature such as tier/T度: store source-aware factual evidence separately from recommendation/ranking decisions. If T度 is imported, it must carry source and timestamp, and the model should still decide how to use it.

## Failure-first boundaries

- Fetch failure: retried inside crawler, then propagated as warning/update event during sync.
- Detail parse anomaly: rejected before replacing stored skills; sync records an update event and preserves the last successful profile.
- Source change: recorded through `source_snapshots` and `update_events`.
- Ambiguous local query: returns an explicit candidate list error rather than guessing.
