---
# ka4-zv7
title: Investigate glean over-exploration on small codebases
status: completed
type: task
priority: high
created_at: 2026-02-16T01:07:46Z
updated_at: 2026-02-16T01:12:17Z
parent: 4q3-09c
---

## Problem

On small codebases (gin, simple rg tasks), glean consistently adds overhead without benefit:
- gin_binding_tag: +61% ctx, +65% cost (baseline: 5 turns, glean: 7 turns)
- gin_context_next: +55% ctx, +42% cost
- rg_flag_definition: +83% ctx, +68% cost (baseline: 3 turns, glean: 5 turns)
- rg_trait_implementors: +248% ctx, +219% cost (baseline: 3 turns, glean: 7 turns)
- gin_middleware_chain: +99% ctx, +50% cost (glean uses 9 glean_search calls vs 12 greps)

## Root Cause Hypothesis

Opus uses glean MCP tools even when the codebase is small enough that direct file reads/greps would be more efficient. The glean_search results include expanded definitions with full source bodies, which bloat context compared to targeted grep matches.

## Investigation

- [ ] Compare glean_search output size vs grep output size for typical gin queries
- [ ] Check if glean_search expand=2 (default) is too aggressive for small repos
- [ ] Look at gin_middleware_chain tool sequences: 9 glean_search calls vs 12 greps — are the glean results redundant?
- [ ] Consider whether glean_search should include repo size hints (file count) so the model can adapt strategy
- [ ] Test whether reducing default expand count (2→1) helps on small repos without hurting large ones

## Findings

### Root cause confirmed: glean search results are too verbose for small repos

For gin_middleware_chain, each glean_search returns ~3.4KB (definitions + 10 usage contexts) vs grep returning ~300 bytes. Most of the glean output is test-file usages that are not useful.

Example: \`glean "ServeHTTP"\` returns 3.4KB — 2 definitions and 8 test-file usages. The model only needs the 14-line definition in gin.go.

### Scope path issue partially resolved

The resolve_scope fix works — bad scope paths now return errors. But the model still wastes 1-2 turns trying bad paths before correcting. In gin_middleware_chain rep1, the model tried scope="benchmark/fixtures/repos/gin" (wrong), got an error, then retried without scope.

### Quantified overhead

For the "ServeHTTP" symbol in gin:
- glean_search output: 3,403 bytes (definitions + usages with context)
- grep output: 309 bytes (matching lines only)
- **11x context overhead per query**

### Possible fixes (ranked by feasibility)

1. **Reduce default expand count**: Change expand default from 2→1. Definitions still get shown but fewer usages. Low risk.
2. **Rank usages better**: Deprioritize test files in usage results. Would help for all repos.
3. **Budget parameter**: The budget parameter already exists but the model rarely uses it. Could reduce output size.
4. **Repo-size-aware hints**: Not practical — the model would need to adapt its strategy, which is unreliable.

Recommendation: Start with (1) reducing expand default, then investigate (2) test file deprioritization.
