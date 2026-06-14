# Release Artifact Acceptance Case

This case proves that a tagged release is usable from the public artifact surface.

## Public artifact smoke

For a release tag `<tag>`:

```bash
tmp="$(mktemp -d /tmp/wzry-release-check.XXXXXX)"
trap 'rm -rf "$tmp"' EXIT
cd "$tmp"

curl -L -O "https://github.com/Develata/wzry-search-mcp/releases/download/<tag>/wzry-search-mcp-linux-x86_64.tar.gz"
curl -L -O "https://github.com/Develata/wzry-search-mcp/releases/download/<tag>/wzry-search-mcp-linux-x86_64.tar.gz.sha256"
sha256sum -c wzry-search-mcp-linux-x86_64.tar.gz.sha256

tar -xzf wzry-search-mcp-linux-x86_64.tar.gz
./wzry-search-mcp-linux-x86_64/wzry-search-mcp --version
```

## Acceptance criteria

- checksum verification passes.
- extracted binary executes on Linux x86_64.
- `--version` prints the intended release version.
- archive contains README, SPEC, LICENSE, config example, docs, and schemas.
- docs-only releases do not claim behavior changes unless corresponding code/tests changed.

## Deployment path check

For AstrBot deployment, confirm the configured command still points to the container binary path and passes `--db <sqlite> serve` exactly as documented in `docs/plan/05_release_deployment.md`.
