# Agent Usage Feature

This feature describes how an Agent should use `wzry-search-mcp` after the local dataset exists.

## Goal

Let the calling model retrieve source-aware factual context and then perform its own reasoning.

The MCP server does not decide lineups, rank heroes, or hide uncertainty.

## Discovery-first workflow

When a user gives a vague hero/item/skill reference:

1. Discover candidates.
   - heroes: `wzry_list_heroes` or `wzry_search_heroes`
   - items: `wzry_list_items` or `wzry_search_items`
   - hero skill text: `wzry_search_hero_skills`
   - summoner skills: `wzry_get_summoner_skills`
2. Resolve the concrete target.
   - hero: `wzry_get_hero_profile` or `wzry_get_hero_profiles`
   - item: `wzry_get_item`
   - hero skill: `wzry_get_hero_skill`
   - summoner skill: `wzry_get_summoner_skill`
3. Reason in the calling model.
   - for lineup questions, call `wzry_get_lineup_context` and explain that recommendation is model-side.
   - cite retrieved facts when useful.
   - do not pretend MCP has a hidden tier-list score.

## Expected observable behavior

- `tools/list` exposes 12 tools.
- each tool has an `outputSchema`.
- successful calls include `structuredContent`.
- text fallback contains JSON for clients that only consume `content[0].text`.
- domain failures are tool errors, not fabricated empty successes.

## Example prompts

User asks: “廉颇适合配谁？”

Agent flow:

1. call `wzry_get_lineup_context` with known allies/enemies/candidates if supplied.
2. if candidates are missing, call discovery/profile tools to gather enough facts.
3. answer from skill texts and roles; state that MCP supplies evidence, while the model makes the recommendation.

User asks: “有护盾的英雄技能有哪些？”

Agent flow:

1. call `wzry_search_hero_skills` with query `护盾`.
2. group hits by hero and skill slot.
3. summarize source-aware results without inventing unstored mechanics.
