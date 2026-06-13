# GitHub Release Artifact Strategy

This project is designed to publish code and binaries, not large mirrored media assets.

## Release artifacts

Recommended artifacts per release:

- `wzry-search-mcp-${target}.tar.gz` or `.zip`: compiled binary plus README excerpt.
- `wzry-schema-${version}.json`: optional exported schema description.
- `sample-wzry.sqlite.zst`: optional tiny sample database for smoke tests only.

Avoid publishing:

- skin data
- image assets
- skin image URLs as a bundled catalog
- large static media resources
- unsourced tier lists

## GitHub Actions release flow

A future release workflow should:

1. Run `cargo fmt --all -- --check`.
2. Run `cargo clippy --all-targets --all-features -- -D warnings`.
3. Run `cargo test --all-features`.
4. Build release binaries for target platforms.
5. Upload binaries as GitHub release assets.
6. Optionally run a small smoke sync with `--limit-heroes 2`, then export JSON/CSV as CI artifacts, not committed data.

## Dataset policy

The canonical dataset is generated locally by users:

```bash
wzry-search-mcp sync --db ~/.local/share/wzry-search-mcp/wzry.sqlite
```

Generated full datasets should not be committed to the repository. If public distribution is desired later, prefer GitHub Release assets with clear timestamp, source URLs, and generation command.

## Tagging

Use semantic version tags:

```text
v0.1.0
v0.2.0
```

Do not tag until the working tree is clean and local validation has passed.
