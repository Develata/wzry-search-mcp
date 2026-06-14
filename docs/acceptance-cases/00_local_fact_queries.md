# Local Fact Query Acceptance Case

This case proves that the local canonical dataset supports factual query flows without network access during query.

## Preconditions

A SQLite database has been created by `sync`, either full sync or a controlled small smoke database.

## Evidence commands

```bash
CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo run --quiet -- \
  --db /tmp/wzry-full-smoke.sqlite \
  list-heroes --limit 5

CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo run --quiet -- \
  --db /tmp/wzry-full-smoke.sqlite \
  hero 廉颇

CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo run --quiet -- \
  --db /tmp/wzry-full-smoke.sqlite \
  search-skills 护盾 --limit 10

CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo run --quiet -- \
  --db /tmp/wzry-full-smoke.sqlite \
  item 破军

CARGO_TARGET_DIR=/tmp/hermes-cargo-target cargo run --quiet -- \
  --db /tmp/wzry-full-smoke.sqlite \
  summoner 闪现
```

## Acceptance criteria

- hero queries return bound basic profile plus skills.
- skill search includes hero identity and matching skill facts.
- item and summoner-skill queries resolve by local data.
- no query command fetches remote official pages.
- ambiguous or unknown names return explicit errors rather than guessed records.
