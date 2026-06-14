# Tools and Output Shapes

`wzry-search-mcp` exposes source-aware factual tools. It does not choose lineups or encode fixed tier lists.

## Core tools

- `wzry_list_heroes`
- `wzry_search_heroes`
- `wzry_get_hero_profile`
- `wzry_get_hero_profiles`
- `wzry_get_hero_skill`
- `wzry_search_hero_skills`
- `wzry_list_items`
- `wzry_search_items`
- `wzry_get_item`
- `wzry_get_summoner_skills`
- `wzry_get_summoner_skill`
- `wzry_get_lineup_context`

## Agent discovery workflow

Agents do not need a hard-coded prompt containing every hero/item name. Use the discovery tools when the user gives vague or partial names:

1. Call `wzry_list_heroes` or `wzry_search_heroes` to identify valid hero names.
2. Call `wzry_get_hero_profile` / `wzry_get_hero_profiles` for full hero + skill facts.
3. Call `wzry_search_hero_skills` when the user asks about mechanics such as 护盾、位移、控制、霸体, or only remembers a skill keyword.
4. Call `wzry_list_items` / `wzry_search_items` before equipment-specific reasoning.
5. Call `wzry_get_summoner_skills` to list all summoner skills, or `wzry_get_summoner_skill` for one skill.

`list` tools accept optional `limit` values in `[1, 500]`; search tools accept optional `limit` values in `[1, 50]` and default to small result sets. Lineup judgement remains model-side.

## 输出形态

`v0.2.0` 起，MCP 层基于官方 Rust MCP SDK（RMCP）实现。工具调用返回：

- `structuredContent`：机器可读结构化 JSON；
- `content[0].text`：同一 JSON 的文本序列化形式，用于兼容只能读取文本内容的旧客户端；
- `outputSchema`：每个工具均声明结构化输出 schema。

返回数组的 discovery 工具会用对象包一层，确保 `outputSchema` 根节点是 object：

- `wzry_list_heroes` / `wzry_search_heroes`：`{"heroes": [...]}`；
- `wzry_get_hero_profiles`：`{"heroes": [...]}`；
- `wzry_search_hero_skills`：`{"hits": [...]}`；
- `wzry_list_items` / `wzry_search_items`：`{"items": [...]}`；
- `wzry_get_summoner_skills`：`{"summoner_skills": [...]}`。

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
