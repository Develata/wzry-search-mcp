# Coverage Matrix

This matrix is the doc/code/test alignment map for `wzry-search-mcp`.

Authority order:

1. `docs/plan/` defines engineering contracts.
2. `docs/registry/` controls stable public names and schema-facing contracts.
3. `docs/features/` describes observable user/agent behavior.
4. `docs/acceptance-cases/` defines the evidence needed before release.
5. Code and tests are projections of the documents above.

| Plan contract | Registry / feature projection | Acceptance case | Primary code surface | Verification evidence |
| --- | --- | --- | --- | --- |
| `plan/00_scope.md` | `features/00_agent_usage.md` | `acceptance-cases/00_local_fact_queries.md` | `src/main.rs`, `src/db/query.rs`, `src/mcp.rs` | CLI query smoke + MCP tool calls |
| `plan/01_data_model.md` | `registry/tool-contracts.md`, `docs/tools.md` | `acceptance-cases/00_local_fact_queries.md` | `src/model.rs`, `src/db/rows.rs`, `src/export.rs` | Rust tests + export smoke |
| `plan/02_source_sync.md`, `plan/06_news_incremental_sync.md` | `features/01_source_sync_and_updates.md`, `features/02_news_incremental_sync.md` | `acceptance-cases/01_source_sync.md`, `acceptance-cases/02_news_incremental_sync.md` | `src/crawler.rs`, `src/parser.rs`, `src/db/write.rs`, `src/main.rs` | limited sync + update check + `sync-changed --dry-run` smoke |
| `plan/03_mcp_contract.md` | `registry/tool-contracts.md`, `features/00_agent_usage.md` | `acceptance-cases/03_mcp_stdio_protocol.md` | `src/mcp.rs`, `tests/mcp_stdio.rs` | `tools/list`, `tools/call`, structured output checks |
| `plan/04_error_semantics.md` | `registry/tool-contracts.md` | `acceptance-cases/03_mcp_stdio_protocol.md` | `src/mcp.rs`, `src/db/query.rs` | domain error and protocol error tests |
| `plan/05_release_deployment.md` | `docs/release-artifacts.md`, `docs/hermes-mcp.md` | `acceptance-cases/04_release_artifacts.md` | `.github/workflows/release.yml`, README install docs | artifact download + checksum + version smoke |

## Drift rules

- If a public MCP tool name, input key, output shape, or error class changes, update `plan/03_mcp_contract.md`, `plan/04_error_semantics.md`, and `registry/tool-contracts.md` before code.
- If source URLs or parsing assumptions change, update `plan/02_source_sync.md` before parser/crawler code.
- If release artifact names, checksums, or deployment paths change, update `plan/05_release_deployment.md` and `docs/release-artifacts.md` before release.
- If only wording or comments change, this matrix may remain unchanged.
