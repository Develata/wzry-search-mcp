# SPEC: wzry-search-mcp v0.4

## 1. Product Boundary

`wzry-search-mcp` is a local factual retrieval layer for AI agents. It builds and queries a local dataset for 王者荣耀 official game facts.

The core object is `hero_profile`, where basic hero metadata and skill data are inseparable at the API layer.

## 2. Included

- Hero profile:
  - basic hero metadata
  - passive skill
  - active skills
  - extra/form skills when official pages expose them
  - source URL, fetched timestamp, and content hash
- Item data
- Summoner skill data
- Local search over the synced dataset
- Manual or scheduled update detection
- One-shot sync-update orchestration
- JSON/CSV export from the local dataset
- Incremental sync when sources change
- Model-side lineup recommendation support through batch evidence retrieval

## 3. Excluded

- Skin data
- Image assets
- Skin image URLs
- Large static media resources
- Rune / 铭文 data
- Hard-coded lineup recommendation rules
- Hard-coded hero tier lists unless later added as a separate sourced extension

## 4. Data Sources

Priority: official public sources first.

- `https://pvp.qq.com/web201605/js/herolist.json`
- `https://pvp.qq.com/web201605/herodetail/{hero_id}.shtml`
- `https://pvp.qq.com/web201605/herodetail/{id_name}.shtml` as fallback when the numeric detail page is missing
- `https://pvp.qq.com/web201605/js/item.json`
- `https://pvp.qq.com/web201605/js/summoner.json`

Non-snapshot reference source:

- `https://pvp.qq.com/web201706/newsindex.shtml` may help manual diagnosis, but it is not part of deterministic hash snapshots because dynamic markup can change between immediate checks.

All stored facts must carry source URL and hash.

## 5. Hero Skill Slot Semantics

The parser maps skill blocks by order:

- index 0 -> `passive`
- index 1 -> `skill_1`
- index 2 -> `skill_2`
- index 3 -> `skill_3`
- index >= 4 and non-empty -> `extra_{index}`

The API accepts aliases:

- `passive`, `被动`
- `1`, `一技能`, `skill_1`
- `2`, `二技能`, `skill_2`
- `3`, `三技能`, `大招`, `skill_3`

## 6. Storage

SQLite is the canonical local store. JSON output is an export/query representation, not the primary store.

Tables:

- `heroes`
- `hero_skills`
- `items`
- `summoner_skills`
- `source_snapshots`
- `update_events`

## 7. MCP Tool Contract

### Discovery tools

- `wzry_list_heroes`: list bounded local hero names/metadata for agent-side discovery.
- `wzry_search_heroes`: search hero candidates by Chinese name, slug, title, or id.
- `wzry_search_hero_skills`: search skill names/descriptions across all heroes and return `{hero, skill}` hits.
- `wzry_list_items`: list bounded local item data for agent-side discovery.
- `wzry_search_items`: search equipment by name or description.
- `wzry_get_summoner_skills`: list all summoner skills.

Discovery tools are meant to reduce prompting friction: an agent can first discover valid names, then call precise detail tools.

### `wzry_get_hero_profile`

Returns a complete hero profile. It must include skills by default.

### `wzry_get_hero_profiles`

Returns complete hero profiles for a list of heroes. Intended for lineup reasoning.

### `wzry_get_lineup_context`

Returns grouped hero profiles for allies, enemies, and candidate pool. It does not choose heroes or score lineups.

The response includes:

```json
{
  "recommendation_should_be_done_by_model": true
}
```

## 8. Update Strategy

- `check-updates` fetches deterministic core source hashes for `herolist.json`, `item.json`, and `summoner.json`.
- If no deterministic source hash changed, do nothing.
- Hero detail pages contain skill text and are synchronized by `sync`; periodic polite `sync` is the v0.1 mechanism for catching skill-only text changes.
- If `herolist.json` changed, run `sync` to refresh hero list and details.
- If `item.json` changed, run `sync` to refresh items.
- If `summoner.json` changed, run `sync` to refresh summoner skills.
- News-based affected-hero incremental sync through CLI `sync-changed`; it analyzes official update-like news, detects mentioned local heroes, and refreshes only their detail pages.
- `sync-update` is the v0.4 sync-oriented one-shot orchestration command. It checks deterministic source hashes, runs news-based incremental sync, emits a compact summary or JSON result, and uses a lock file by default.
- `sync-update` does not run full `sync` by default. It may run full sync only when the operator explicitly passes `--fallback-full` and deterministic source hashes changed.
- `sync-update --dry-run` must not refresh hero details or run full sync; it reports the planned state and warnings only.

## 9. Polite Crawling

- Default serial requests.
- Random delay between hero detail requests.
- Bounded retries.
- No media crawling.
- User-agent identifies the project.

## 10. Failure Semantics

- Missing DB -> return actionable error: run `sync` first.
- Ambiguous hero -> return candidates.
- Missing source -> return error with source URL.
- Detail parse anomaly -> reject replacing that hero's stored skills, record an update event, and keep the last successful profile queryable.
- No model-side recommendation -> MCP still returns context successfully.
