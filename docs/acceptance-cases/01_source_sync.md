# Source Sync Acceptance Case

This case proves the sync/update path can build and inspect the local dataset from public official sources.

## Limited smoke

```bash
rm -f /tmp/wzry-source-smoke.sqlite /tmp/wzry-source-smoke.sqlite-*
CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo run --quiet -- \
  --db /tmp/wzry-source-smoke.sqlite \
  sync --no-polite --limit-heroes 2
```

Acceptance criteria:

- command exits successfully.
- at least the limited hero set, their skills, item list, and summoner-skill list are stored.
- no parse warning silently replaces good previous data.

## Update detection smoke

```bash
rm -f /tmp/wzry-update-smoke.sqlite /tmp/wzry-update-smoke.sqlite-*
CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo run --quiet -- \
  --db /tmp/wzry-update-smoke.sqlite \
  check-updates --write-snapshots
CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo run --quiet -- \
  --db /tmp/wzry-update-smoke.sqlite \
  check-updates
```

Acceptance criteria:

- first run can write deterministic source snapshots.
- second run against unchanged deterministic list sources reports unchanged.
- behavior matches `docs/update-detection.md`.
