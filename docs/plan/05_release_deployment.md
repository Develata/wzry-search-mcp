# Release and Deployment Contract

Release work must prove the binary artifact from the public surface, not only a local build.

## Release artifact shape

GitHub release artifacts are documented in `docs/release-artifacts.md` and currently include a Linux x86_64 archive plus checksum.

Expected archive content:

- `wzry-search-mcp` executable.
- `README.md`.
- `SPEC.md`.
- `LICENSE`.
- `config.example.toml`.
- `docs/`.
- `schemas/`.

## Stable deployment paths

For Develata's AstrBot deployment, these paths are intentionally stable across `v0.2.x` documentation hardening:

- host binary: `/opt/1panel/docker/compose/astrbot/data/wzry-search-mcp/bin/wzry-search-mcp`
- container binary: `/AstrBot/data/wzry-search-mcp/bin/wzry-search-mcp`
- host database: `/opt/1panel/docker/compose/astrbot/data/wzry-search-mcp/wzry.sqlite`
- container database: `/AstrBot/data/wzry-search-mcp/wzry.sqlite`

AstrBot MCP config should keep the command and args shape:

```json
{
  "command": "/AstrBot/data/wzry-search-mcp/bin/wzry-search-mcp",
  "args": ["--db", "/AstrBot/data/wzry-search-mcp/wzry.sqlite", "serve"]
}
```

## Release gates

Before tagging a release that changes behavior or artifacts:

1. local format/lint/test/build gate passes.
2. source sync smoke passes or is explicitly scoped out for docs-only releases.
3. MCP stdio acceptance passes.
4. independent review has no blocking finding for code-bearing releases.
5. public artifact is downloaded, checksum-verified, and `--version` checked.

Docs-only releases may skip full data sync when no code, schema, workflow, or packaging file changed, but must still verify the committed scope and docs links.
