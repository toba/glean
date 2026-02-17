---
# sag-6xx
title: Review upstream tilth changes for incorporation
status: completed
type: task
priority: normal
created_at: 2026-02-17T00:01:32Z
updated_at: 2026-02-17T00:08:45Z
sync:
    github:
        issue_number: "13"
        synced_at: "2026-02-17T00:08:59Z"
---

Review recent upstream changes from jahala/tilth (30 commits, Feb 9–14 2026) for potential incorporation into glean.

## High-priority diffs to review

These files had the most activity and are most likely to contain improvements worth porting:

- [x] `src/mcp.rs` — reviewed
- [x] `src/search/symbol.rs` — reviewed
- [x] `src/search/mod.rs` — reviewed
- [x] `src/read/outline/code.rs` — reviewed (glean ahead)
- [x] `src/read/mod.rs` — reviewed (glean ahead)
- [x] `src/read/imports.rs` — reviewed (identical)

## Specific features to evaluate

- [x] Multi-symbol search — already in glean
- [x] Search all files regardless of .gitignore — already in glean
- [x] Bug fixes — glean is ahead (binary detection, is_code gating)

## How to review

For each file, run:
```bash
gh api "repos/jahala/tilth/contents/src/<path>?ref=main" --jq '.content' | base64 -d
```
Compare against glean's version. Look for:
- Bug fixes we're missing
- Performance improvements
- New edge case handling
- API improvements worth adopting

## Findings

### Worth incorporating from tilth

| Priority | Change | Effort |
|----------|--------|--------|
| **High** | Impl/trait + interface detection in symbol search — surfaces `impl Trait for Type` and `class X implements Interface` when searching for a trait/interface | Medium |
| **High** | Faceted search results (>5 matches grouped into Definitions/Implementations/Tests/Usages) | Medium |
| **High** | Relative paths in output (`rel()` helper strips scope prefix) | Small |
| **Medium** | Transitive callee resolution (2-hop call graph in expand footer) | Medium |
| **Medium** | Sibling surfacing (referenced fields/methods from same struct/impl) | Medium |
| **Medium** | Noise stripping + smart truncation in expanded code | Medium |
| **Medium** | Add "Replaces host Read/Grep/Glob tool" to MCP tool descriptions | Trivial |
| **Medium** | Bump default expand from 1 to 2 | Trivial |
| **Low** | Import-skipping in expand_match (skip leading use/import lines) | Small |
| **Low** | Consecutive blank line collapsing in expanded code | Small |
| **Low** | Small-file outline skip (<50 lines) | Trivial |
| **Low** | Separate early-quit thresholds for defs (50) vs usages (30) | Trivial |

### Glean is already ahead on

- Swift and Zig language support (tree-sitter outlines + definition detection + tests)
- `walk_collect` abstraction (tilth still inlines walker boilerplate)
- Binary detection in usage search (`BinaryDetection::convert`)
- Heuristic fallback gated on `is_code` (tilth allows Markdown false positives)
- `parse_tree` shared helper
- `resolve_scope` with helpful error messages (tilth silently falls back)
- `io_err` helper for cleaner error construction
- `using_namespace_declaration` recognition in outlines
- Comprehensive test coverage across all compared files

## Summary of Changes

Reviewed all 6 high-priority upstream files (mcp.rs, symbol.rs, search/mod.rs, outline/code.rs, read/mod.rs, imports.rs) plus content.rs. Glean is ahead on code quality and language support. The main items worth porting from tilth are: impl/trait detection in symbol search, faceted result grouping, relative path output, transitive callee resolution, sibling surfacing, and noise stripping in expanded code.
