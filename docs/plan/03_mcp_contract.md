# MCP Contract

The MCP server is the public Agent-facing contract of `wzry-search-mcp`.

## Transport

- Transport: standard input / standard output.
- Protocol: JSON-RPC messages over UTF-8 text lines, as implemented by official Rust MCP SDK RMCP.
- Standard output is reserved for protocol messages only.
- Logs and diagnostics must go to standard error.
- CLI entrypoint remains `wzry-search-mcp --db <sqlite-path> serve`.

RMCP source: <https://github.com/modelcontextprotocol/rust-sdk>
MCP stdio transport reference: <https://modelcontextprotocol.io/specification/2025-06-18/basic/transports#stdio>

## Capabilities

The server exposes `tools` capability and supports:

- `initialize`
- `notifications/initialized`
- `ping`
- `tools/list`
- `tools/call`

## Tool contract

The controlled list of public tool names and argument keys is in `docs/registry/tool-contracts.md`.

Public tool names are semver-relevant. Renaming, removing, or changing an input key must be treated as a contract change.

## Structured output

Since `v0.2.0`, each tool returns typed structured output through RMCP:

- `structuredContent`: machine-readable JSON object.
- `content[0].text`: JSON text fallback carrying the same semantic payload for older clients.
- `outputSchema`: schema declared in `tools/list` from Rust return types.

Array-like results are wrapped in an object so the output schema root is an object, for example `{ "heroes": [...] }`.

## No network in tools

MCP tools must query the local SQLite store only. Network access belongs to CLI sync/update code, not to `src/mcp.rs` tool handlers.

## Projection to verification

The minimum protocol acceptance case is `docs/acceptance-cases/03_mcp_stdio_protocol.md` and `tests/mcp_stdio.rs`:

1. initialize server.
2. list tools.
3. verify expected tool count and `outputSchema` presence.
4. call a successful tool and verify `structuredContent`.
5. call representative domain/protocol errors and verify error semantics.
