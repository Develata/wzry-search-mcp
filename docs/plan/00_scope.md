# Scope Contract

`wzry-search-mcp` is a local factual retrieval MCP for 王者荣耀 public game data.

It is not a recommendation engine, not an online proxy to Tencent pages, and not a media mirror.

## Included scope

- Official public hero list data.
- Hero detail facts: basic profile plus passive/active/extra skill entries.
- Official public equipment data.
- Official public summoner-skill data.
- Local SQLite canonical dataset.
- CLI commands for sync, query, update detection, export, and MCP serving.
- MCP tools for local factual retrieval and model-side lineup evidence.
- Release artifacts containing a binary and documentation, not a prebuilt gameplay database.

## Excluded scope

- Skins and media assets.
- Large static image mirrors.
- Rune systems.
- Subjective tier lists, matchup scores, fixed lineup templates, or hard-coded recommendations.
- Live network calls inside MCP tools.
- Automatic mutation of external Agent configuration.

## Boundary invariant

The MCP server provides evidence. The calling model performs judgment.

`wzry_get_lineup_context` may gather allies, enemies, and candidate hero profiles, but the field `recommendation_should_be_done_by_model` remains true. Any future scoring feature must be sourced, timestamped, and versioned as evidence rather than silently becoming the system ontology.

## Sources

Current official public data entries are documented in `README.md` and `plan/02_source_sync.md`:

- Hero list: <https://pvp.qq.com/web201605/js/herolist.json>
- Hero detail page shape: `https://pvp.qq.com/web201605/herodetail/{hero_id}.shtml`
- Item list: <https://pvp.qq.com/web201605/js/item.json>
- Summoner skills: <https://pvp.qq.com/web201605/js/summoner.json>
