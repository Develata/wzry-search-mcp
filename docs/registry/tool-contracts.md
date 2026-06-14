# Tool Contracts Registry

This registry is the controlled MCP tool-name and shape table. It is semver-relevant.

All tools are local SQLite queries. None of them may fetch remote pages during `tools/call`.

| Tool | Arguments | Success output root | Purpose |
| --- | --- | --- | --- |
| `wzry_list_heroes` | `limit?: integer` | `{ "heroes": [...] }` | list local hero candidates |
| `wzry_search_heroes` | `query: string`, `limit?: integer` | `{ "heroes": [...] }` | search hero candidates by name/id/title |
| `wzry_get_hero_profile` | `hero: string` | `HeroProfile` object | get one bound hero profile plus skills |
| `wzry_get_hero_profiles` | `heroes: string[]` | `{ "heroes": [...] }` | batch get hero profiles for lineup reasoning |
| `wzry_get_hero_skill` | `hero: string`, `skill: string` | `HeroSkill` object | get one passive/active/extra skill |
| `wzry_search_hero_skills` | `query: string`, `limit?: integer` | `{ "hits": [...] }` | search skill names and descriptions |
| `wzry_list_items` | `limit?: integer` | `{ "items": [...] }` | list local equipment records |
| `wzry_search_items` | `query: string`, `limit?: integer` | `{ "items": [...] }` | search equipment records |
| `wzry_get_item` | `item: string` | `Item` object | get one equipment record |
| `wzry_get_summoner_skills` | `limit?: integer` | `{ "summoner_skills": [...] }` | list summoner skills |
| `wzry_get_summoner_skill` | `skill: string` | `SummonerSkill` object | get one summoner skill |
| `wzry_get_lineup_context` | `allies?: string[]`, `enemies?: string[]`, `candidate_pool?: string[]` | `LineupContext` object | gather evidence for model-side lineup reasoning |

## Limit conventions

- list tools: `limit` is optional and bounded by implementation.
- search tools: `limit` is optional and defaults to a small result set.
- empty batch requests that cannot produce meaningful evidence should be domain errors, not successful empty recommendations.

## Structured output convention

Every tool declares `outputSchema` through RMCP structured output. Successful calls return `structuredContent` plus JSON text fallback.

Array-like results are wrapped in objects rather than using a raw array root. This keeps schema roots object-shaped and makes future additive fields possible.

## Error convention

- domain lookup failures: tool result `isError: true`, no `structuredContent`.
- schema/protocol/tool-dispatch failures: RMCP-level error behavior.
- no fabricated success payloads for failed lookups.

## Change protocol

Before changing this registry:

1. update `docs/plan/03_mcp_contract.md` and `docs/plan/04_error_semantics.md` if relevant.
2. update `docs/features/00_agent_usage.md` if user/agent workflow changes.
3. update `docs/acceptance-cases/03_mcp_stdio_protocol.md` and `tests/mcp_stdio.rs`.
4. only then change `src/mcp.rs`.
