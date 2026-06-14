# Source Sync and Update Feature

This feature describes the observable sync/update behavior for maintainers.

## Sync behavior

A normal sync fetches public official sources, parses them, and writes a local SQLite dataset.

Expected user-visible outcomes:

- heroes, skills, items, and summoner skills become queryable through CLI and MCP.
- records include source evidence where available.
- parse warnings remain visible instead of being silently swallowed.
- generated exports can be recreated from SQLite.

## Update detection behavior

`check-updates` compares deterministic public source snapshots and reports whether tracked source content changed.

Expected behavior:

- first `--write-snapshots` run stores snapshots.
- a second check against unchanged sources reports no change.
- dynamic/news pages are not treated as deterministic canonical snapshots unless promoted through a contract update.

## Politeness behavior

Default sync is conservative and polite. Short delay and `--no-polite` modes are allowed for controlled smoke tests, CI source smoke, or local debugging. They are not the default production sync behavior.

## Non-goals

- Sync does not publish scraped datasets automatically.
- Sync does not mirror media resources.
- Update detection does not mutate MCP server behavior at query time.
