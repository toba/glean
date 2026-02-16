---
# r5a-ioi
title: Rewrite tests for search quality over existence
status: completed
type: task
priority: normal
created_at: 2026-02-16T01:44:41Z
updated_at: 2026-02-16T01:49:58Z
---

Replace 'contains any match' assertions with quality assertions: definition ranked first, callee resolution points to correct next file, result counts are tight, ranking order is correct.

## Summary of Changes

Rewrote fixture-dependent tests and integration tests to assert **search quality** rather than mere existence:

**Symbol search (symbol.rs)**: definition must be matches[0], def_range must be populated for expand, cross-file usages must appear as navigation breadcrumbs, result count must be tight, context must not demote definitions

**Content search (content.rs)**: most relevant file must rank first (context.go before middleware.go), regex search must return the actual signature, unique strings must have tight result counts

**Callers (callers.rs)**: calling_function populated, caller_range populated for expand, callers found across multiple files

**Callees (callees.rs)**: complete call chains (all callees found), no noise, sorted+deduped, def_range isolation works

**Integration (integration.rs)**: definition-first in formatted output, cross-file breadcrumbs visible, line range in definition headers for expand, file read shows [full] mode for small files, fallthrough resolves on first try, budget constrains output, glob completeness (no false includes)

**Ranking (rank.rs)**: added doc comments explaining each signal's benchmark impact
