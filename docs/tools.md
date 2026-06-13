# Tools and Output Shapes

`wzry-search-mcp` exposes source-aware factual tools. It does not choose lineups or encode fixed tier lists.

## Core tools

- `wzry_search_heroes`
- `wzry_get_hero_profile`
- `wzry_get_hero_profiles`
- `wzry_get_hero_skill`
- `wzry_search_items`
- `wzry_get_item`
- `wzry_get_summoner_skills`
- `wzry_get_summoner_skill`
- `wzry_get_lineup_context`

## Example: hero profile

Input:

```json
{"hero":"廉颇"}
```

Output shape:

```json
{
  "hero": {
    "hero_id": 105,
    "ename": 105,
    "cname": "廉颇",
    "id_name": "lianpo",
    "title": "正义爆轰",
    "hero_type": 3,
    "roles": ["坦克"],
    "moss_id": 3627,
    "source": {
      "url": "https://pvp.qq.com/web201605/js/herolist.json",
      "fetched_at": "...",
      "content_hash": "..."
    }
  },
  "skills": [
    {
      "hero_id": 105,
      "slot": "passive",
      "name": "勇士之魂",
      "cooldown": "0",
      "cost": "0",
      "description": "...",
      "source": {
        "url": "https://pvp.qq.com/web201605/herodetail/105.shtml",
        "fetched_at": "...",
        "content_hash": "..."
      }
    }
  ],
  "parse_warnings": []
}
```

## Example: lineup context

Input:

```json
{
  "allies": ["廉颇", "小乔"],
  "enemies": ["兰陵王"],
  "candidate_pool": ["孙尚香", "马可波罗"]
}
```

Output shape:

```json
{
  "allies": [{"hero": {}, "skills": [], "parse_warnings": []}],
  "enemies": [{"hero": {}, "skills": [], "parse_warnings": []}],
  "candidate_pool": [{"hero": {}, "skills": [], "parse_warnings": []}],
  "recommendation_should_be_done_by_model": true
}
```

The calling model should reason from passive/skill texts, not from hard-coded MCP scores.
