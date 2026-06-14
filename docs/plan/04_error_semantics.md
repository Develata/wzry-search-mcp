# Error Semantics Contract

Errors must be useful to an Agent and must not corrupt the MCP transport.

## Error classes

### Domain errors

Domain errors are normal tool-call failures caused by valid protocol messages whose requested game data cannot be resolved or accepted.

Examples:

- unknown hero name.
- ambiguous hero query.
- unknown item name.
- invalid empty batch request.
- unsupported skill selector for an existing hero.

MCP projection:

- `tools/call` response is a successful JSON-RPC response object.
- tool result has `isError: true`.
- error text is placed under `content`.
- `structuredContent` is absent.

### Protocol / schema errors

Protocol errors are malformed MCP requests, missing required arguments, invalid JSON-RPC shape, or unknown tool names handled by RMCP.

MCP projection:

- JSON-RPC error response or SDK-level tool dispatch error, according to RMCP behavior.
- No fabricated structured success payload.

### Internal errors

Internal errors include SQLite failures, unexpected IO errors, and runtime failures.

Projection:

- surface as an error, not an empty success object.
- avoid leaking credentials or host-specific secrets.
- preserve enough message text for debugging.

## Invariants

1. A failed domain lookup must not be encoded as a successful empty object unless the tool contract explicitly says empty results are normal.
2. `structuredContent` is only for successful typed payloads.
3. Stdio framing remains valid even when a tool fails.
4. Tests must cover at least one successful structured output, one domain error, one missing-argument error, and one unknown-tool error.

## Code projection

- Tool implementation: `src/mcp.rs`.
- Lookup semantics: `src/db/query.rs`.
- Protocol acceptance tests: `tests/mcp_stdio.rs`.
