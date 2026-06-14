# Data Model Contract

The canonical data model is defined in Rust structs under `src/model.rs` and persisted through SQLite tables owned by `src/db/schema.rs`.

## Stable objects

- `HeroBasic`: one hero identity and role record.
- `HeroSkill`: one passive, active, or extra skill entry.
- `HeroProfile`: `HeroBasic` plus ordered skills and parse warnings.
- `Item`: one equipment record.
- `SummonerSkill`: one summoner-skill record.
- `SourceInfo`: source URL, fetch time, and content hash evidence.
- `SourceSnapshot`: hash snapshot for update detection.
- `UpdateEvent`: local observation of source changes or parse anomalies.
- `LineupContext`: grouped hero profiles for model-side reasoning.

## Invariants

1. Hero profile queries bind basic hero data and skills together.
2. Passive skills are first-class skills with slot `passive`.
3. Extra skill/form entries use stable `extra_n` slots rather than ad-hoc fields.
4. Source evidence travels with model records where available.
5. Parse warnings are visible to callers instead of being silently discarded.
6. Generated JSON/CSV datasets are artifacts, not source-of-truth files.
7. MCP output schemas are generated from typed Rust return structs.

## Projection to code

- Rust model structs: `src/model.rs`.
- SQLite schema: `src/db/schema.rs`.
- Write path: `src/db/write.rs`.
- Read path and resolution: `src/db/query.rs`, `src/db/rows.rs`.
- Export projection: `src/export.rs`.
- MCP structured output projection: `src/mcp.rs`.

Any change to a stable object field that is visible through CLI export or MCP output requires updating:

- `docs/registry/tool-contracts.md` if MCP-visible.
- `docs/tools.md` examples if user-facing.
- `tests/mcp_stdio.rs` or export tests if shape-sensitive.
