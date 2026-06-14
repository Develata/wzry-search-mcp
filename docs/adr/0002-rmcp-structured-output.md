# ADR 0002: RMCP Structured Output for v0.2.0

Date: 2026-06
Status: accepted

## Context

The initial MCP implementation manually handled stdio JSON-RPC details. For `v0.2.0`, the project needed closer alignment with the official Rust MCP SDK and a stronger typed contract for Agent clients.

Official SDK source: <https://github.com/modelcontextprotocol/rust-sdk>

## Decision

Use RMCP for the MCP server layer and expose structured output:

- tool methods return typed structures.
- `tools/list` includes `outputSchema`.
- successful `tools/call` responses include `structuredContent`.
- text JSON fallback remains available for older clients.

## Consequences

Positive:

- protocol behavior is delegated to SDK code.
- output schemas are generated from Rust types.
- Agent clients can consume structured data without parsing prose.

Trade-offs:

- RMCP SDK behavior becomes part of the dependency surface.
- manual compatibility with non-main-path framing styles is not preserved.

Follow-up docs:

- `docs/plan/03_mcp_contract.md`
- `docs/registry/tool-contracts.md`
- `docs/acceptance-cases/03_mcp_stdio_protocol.md`
