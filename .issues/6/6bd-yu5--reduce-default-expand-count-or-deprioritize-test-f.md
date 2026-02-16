---
# 6bd-yu5
title: Reduce default expand count or deprioritize test file usages
status: completed
type: task
priority: normal
created_at: 2026-02-16T01:14:22Z
updated_at: 2026-02-16T02:23:45Z
parent: 4q3-09c
---

## Problem

glean_search with expand=2 (default) returns full function bodies for the top 2 definitions + 10 usage contexts. On small repos like gin, this produces 11x more context per query than grep (3.4KB vs 309 bytes for `ServeHTTP`).

Most of the excess comes from:
1. Expanded definition bodies (full source of matching functions)
2. Usage contexts from test files (auth_test.go, benchmarks_test.go) that are not useful for navigation

This causes +50-250% context regression on gin/simple-rg tasks where baseline solves with 2-3 targeted greps.

## Options

- [x] Reduce default expand from 2 to 1 (fewer expanded definitions, still shows outline)
- [ ] Deprioritize test file usages in search results (rank.rs) — `*_test.go`, `*_test.rs`, `*.test.ts`, etc.
- [ ] Cap usage results shown (currently 10) to a lower number like 5
- [ ] Consider if the model could be guided to use expand=0 on small queries

## Impact

Would help: gin_binding_tag, gin_context_next, rg_flag_definition, rg_trait_implementors, gin_middleware_chain

## Summary of Changes

**Test file deprioritization** (src/search/rank.rs):
- Added is_test_file() recognizing test naming conventions across Go, Rust, Python, JS/TS, Java, Kotlin, Swift, and tests/test/__tests__/ directories
- Added -100 penalty for non-definition matches in test files
- Two new tests: test_files_deprioritized, test_file_definition_still_ranks_above_source_usage

**Expand default reduced** (src/mcp.rs):
- Default expand changed from 2 to 1
- Updated tool description and JSON schema
- Models can still pass expand=2 when needed
