# ADR 0001: Local Fact Store Instead of Live Query MCP

Date: 2026-06
Status: accepted

## Context

王者荣耀 official public pages are useful data sources, but Agent tool calls should be fast, deterministic, and side-effect-light. Live web fetches during an MCP tool call would mix networking, parsing, and reasoning latency into every query.

## Decision

`wzry-search-mcp` builds a local SQLite canonical dataset through explicit CLI sync/update commands. MCP tools query only that local dataset.

## Consequences

Positive:

- MCP tool calls are local and predictable.
- source sync failures are isolated from query-time reasoning.
- artifacts can be exported and tested independently.

Trade-offs:

- data may be stale until the next sync.
- update detection and sync require separate operational discipline.

Follow-up docs:

- `docs/plan/02_source_sync.md`
- `docs/features/01_source_sync_and_updates.md`
- `docs/acceptance-cases/01_source_sync.md`
