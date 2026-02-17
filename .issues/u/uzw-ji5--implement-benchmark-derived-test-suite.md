---
# uzw-ji5
title: Implement benchmark-derived test suite
status: completed
type: task
priority: normal
created_at: 2026-02-16T01:30:25Z
updated_at: 2026-02-16T01:35:27Z
sync:
    github:
        issue_number: "2"
        synced_at: "2026-02-17T00:08:59Z"
---

Add ~47 new tests: fixtures, unit tests for format/rank/session/budget/callees, fixture-dependent tests for content/symbol/callers, edit system tests, and integration tests.

## Summary of Changes

Added 47 new tests (25 → 72 total) covering previously untested code paths:

### Fixtures (15 files)
- mini-go: 4 files (router, context, middleware, go.mod)
- mini-rust: 4 files (lib, searcher, lines, Cargo.toml)
- mini-ts: 4 files (schemas, errors, parse, package.json)
- mini-swift: 3 files (Session, Request, Validation)

### Unit tests (no fixtures)
- format.rs: 7 tests (hash determinism, 12-bit range, hashlines format, parse_anchor, number_lines, search_header)
- rank.rs: 6 tests (definitions > usages, exact > inexact, vendor penalty, context boost, small file bonus, determinism)
- session.rs: 3 tests (expand dedup, summary counts, reset)
- budget.rs: 3 tests (under budget, over budget, header preservation)
- callees.rs: 5 tests (Rust/Go/TS extraction, dedup+sort, range filter)

### Fixture-dependent tests
- content.rs: 3 tests (literal search, regex search, no results)
- symbol.rs: 4 tests (Go/Rust definitions, usages, dedup)
- callers.rs: 2 tests (Go callers, no results)
- edit.rs: 5 tests (hash roundtrip, valid edit, hash mismatch, overlapping ranges, delete)

### Integration tests
- tests/integration.rs: 9 tests exercising full run() flow (symbol, content, glob, file path, fallthrough, not found across 4 languages)

### Other changes
- Added tempfile dev-dependency
- Fixed pre-existing clippy needless_raw_string_hashes warnings
