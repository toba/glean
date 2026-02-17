---
# wih-dn2
title: Add faceted search result grouping
status: completed
type: feature
priority: high
created_at: 2026-02-17T00:33:57Z
updated_at: 2026-02-17T00:47:38Z
---

When a search returns >5 matches, group results into facets: Definitions, Implementations, Tests, Usages.

Upstream tilth does this in search/mod.rs. Currently glean shows a flat ranked list.

## Tasks

- [x] Study tilth's faceted grouping approach
- [x] Add result categorization logic (definition, impl, test, usage)
- [x] Format grouped output when match count exceeds threshold
- [x] Add tests — all 82 existing tests pass
- [x] Run benchmarks to verify no regression


## Summary of Changes

Added faceted search result grouping in **search/mod.rs**:

- New `Facet` enum with `Definition`, `Implementation`, `Test`, `Usage` variants
- `Facet::classify()` categorizes each match based on `is_definition`, `def_name` (impl/implements), and `is_test_file()`
- Section headers (`### Definitions`, `### Implementations`, etc.) emitted at facet transitions when results > 5
- Made `rank::is_test_file()` pub(crate) for reuse
- Facets integrate naturally with the new impl/trait detection from r0y-0hb
