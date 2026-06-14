# MCP Stdio Protocol Acceptance Case

This case proves the public Agent-facing MCP contract.

## Preconditions

- a small SQLite test database exists.
- server is launched as:

```bash
wzry-search-mcp --db <sqlite-path> serve
```

## Required protocol checks

1. Send `initialize`; expect `serverInfo.name = wzry-search-mcp` and tools capability.
2. Send `notifications/initialized`.
3. Send `ping`; expect a valid response.
4. Send `tools/list`; expect exactly 12 tools.
5. Confirm each public tool has stable name and schema shape listed in `docs/registry/tool-contracts.md`.
6. Confirm at least one tool, currently `wzry_get_hero_profile`, includes `outputSchema`.
7. Call `wzry_get_hero_profile` with `{ "hero": "廉颇" }`; expect `structuredContent.hero.cname = "廉颇"`.
8. Confirm `content[0].text` contains JSON fallback for the same semantic payload.
9. Call a known domain failure such as an unknown hero; expect tool result `isError: true` and no `structuredContent`.
10. Call an unknown tool and a missing-argument request; expect RMCP/protocol-level errors, not fake structured success.

## Automated mapping

Primary test file: `tests/mcp_stdio.rs`.

Any change to this acceptance case should be mirrored in that test or in a named SDK smoke script before release.
