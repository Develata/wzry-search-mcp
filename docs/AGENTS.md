# docs/

## Purpose

Project documentation for `wzry-search-mcp`. The docs are organized so both humans and agents can understand the contract before changing code.

Authoritative layers:

- `plan/`: engineering contracts — data model, source sync, MCP contract, error semantics, release/deployment boundaries.
- `features/`: observable behavior and agent/human usage flows.
- `acceptance-cases/`: verification cases mapped to CLI commands, SDK smoke tests, integration tests, and release checks.
- `registry/`: controlled tables for tool contracts, source URLs, schema names, and stable public names.
- `adr/`: decision history; useful context, but current `plan/` files win if they differ.
- `tasks/`: implementation plans for active or historical work; lower authority than `plan/`.
- `overview/`: cross-layer explanations for humans.
- `report/`: dated audits and snapshots; never authoritative over current contracts.

## Agent reading order

Before changing MCP behavior, source sync, data shape, release packaging, or AstrBot deployment:

1. Read this file.
2. Read the relevant file under `docs/plan/`.
3. Read `docs/registry/tool-contracts.md` if tool names, schemas, or error semantics are involved.
4. Read matching files under `docs/features/` and `docs/acceptance-cases/`.
5. Modify docs first when the contract changes.
6. Modify code/tests as a projection of the docs.
7. Run smoke + tests + review before release.

## Rules

- Do not add hard-coded recommendations, tier lists, or lineup scores to MCP docs or code unless a sourced extension explicitly defines them.
- Keep source URLs explicit when data-source behavior changes.
- Public MCP tool names are stable contract names; changing or removing one requires a versioned contract note.
- `structuredContent`, `outputSchema`, text JSON fallback, and error semantics must stay aligned across `plan/`, `registry/`, tests, and README.
- Files under `report/` and `tasks/` may explain history, but they do not override `plan/` or `registry/`.
