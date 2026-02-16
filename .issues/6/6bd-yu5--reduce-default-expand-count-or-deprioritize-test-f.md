---
# 6bd-yu5
title: Reduce default expand count or deprioritize test file usages
status: ready
type: task
priority: normal
created_at: 2026-02-16T01:14:22Z
updated_at: 2026-02-16T01:14:22Z
parent: 4q3-09c
---

## Problem

glean_search with expand=2 (default) returns full function bodies for the top 2 definitions + 10 usage contexts. On small repos like gin, this produces 11x more context per query than grep (3.4KB vs 309 bytes for `ServeHTTP`).

Most of the excess comes from:
1. Expanded definition bodies (full source of matching functions)
2. Usage contexts from test files (auth_test.go, benchmarks_test.go) that are not useful for navigation

This causes +50-250% context regression on gin/simple-rg tasks where baseline solves with 2-3 targeted greps.

## Options

- [ ] Reduce default expand from 2 to 1 (fewer expanded definitions, still shows outline)
- [ ] Deprioritize test file usages in search results (rank.rs) — `*_test.go`, `*_test.rs`, `*.test.ts`, etc.
- [ ] Cap usage results shown (currently 10) to a lower number like 5
- [ ] Consider if the model could be guided to use expand=0 on small queries

## Impact

Would help: gin_binding_tag, gin_context_next, rg_flag_definition, rg_trait_implementors, gin_middleware_chain
