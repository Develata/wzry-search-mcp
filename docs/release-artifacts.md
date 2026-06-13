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

The repository includes `.github/workflows/release.yml`. It runs on manual dispatch and semantic-version tags (`v*.*.*`).

The workflow:

1. Run `cargo fmt --all -- --check`.
2. Run `cargo clippy --all-targets --all-features -- -D warnings`.
3. Run `cargo test --all-features`.
4. Build the Linux x86_64 release binary.
5. Package the binary with README/SPEC/LICENSE/config/docs/schemas.
6. Generate a SHA-256 checksum.
7. Verify the archive contains `config.example.toml` and the hero profile schema.
8. Upload the archive and checksum as GitHub Actions workflow artifacts.
9. On semantic-version tag pushes only, create a GitHub Release and attach the archive plus checksum via `gh release create`; if the release already exists, upload the assets with `--clobber`.

Manual `workflow_dispatch` builds artifacts but does not create a release unless it is run on a tag ref.

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
