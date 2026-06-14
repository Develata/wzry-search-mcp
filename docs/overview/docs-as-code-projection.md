# Overview: Docs as Code Projection

`wzry-search-mcp` keeps documentation in a lightweight contract-first layout.

```text
docs/plan/              engineering contracts
docs/registry/          controlled public names and schemas
docs/features/          observable behavior
docs/acceptance-cases/  verification evidence
docs/adr/               decision history
docs/tasks/             implementation blueprints
```

The purpose is not to create a large documentation system. The purpose is to make the repository agent-readable:

- `plan/` tells an agent what must remain true.
- `features/` tells an agent what users observe.
- `acceptance-cases/` tells an agent how to prove it.
- code then becomes a projection of the docs, rather than docs being an after-the-fact explanation.

For this repository, the most important public contracts are:

1. MCP stdio transport and structured output.
2. Stable tool names and input keys.
3. Local factual retrieval only; no hidden recommendation engine.
4. Public official source sync into local SQLite.
5. Stable release/deployment paths for AstrBot usage.
