---
# ijz-v2j
title: Use relative paths in output
status: completed
type: feature
priority: high
created_at: 2026-02-17T00:33:57Z
updated_at: 2026-02-17T00:45:57Z
---

Add a rel() helper that strips the scope prefix from paths in output, showing relative paths instead of absolute ones.

Upstream tilth does this throughout its output formatting. Makes output cleaner and more readable.

## Tasks

- [x] Add rel() helper function that strips scope prefix
- [x] Apply relative paths in search result output
- [ ] Apply relative paths in read/outline output (deferred — read shows one file at a time)
- [x] Add tests (existing tests pass with relative paths)


## Summary of Changes

Added `format::rel(path, scope)` helper that strips the scope prefix to produce relative display paths. Applied it in all search output locations:

- **format.rs**: New `rel()` function
- **search/mod.rs**: Updated `format_matches()` and `expand_match()` to accept `scope` param and use `rel()` for all path display — match headers, expand blocks, callee footers, related file hints, and glob results

Read pipeline deferred: `read_file()` displays one file at a time where the full path is useful context, and threading scope through would require API changes.
