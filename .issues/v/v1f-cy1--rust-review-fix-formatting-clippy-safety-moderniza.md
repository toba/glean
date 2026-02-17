---
# v1f-cy1
title: 'Rust review: fix formatting, clippy, safety, modernization, dedup'
status: completed
type: task
priority: normal
created_at: 2026-02-15T18:13:40Z
updated_at: 2026-02-15T18:20:07Z
sync:
    github:
        issue_number: "8"
        synced_at: "2026-02-17T00:08:58Z"
---

Implement all findings from rust-review skill

## Summary of Changes

All rust-review findings implemented:

1. **Edition upgrade**: 2021 → 2024 (enables let-chains, stricter match ergonomics)
2. **Formatting & clippy**: cargo fmt clean, zero clippy warnings
3. **SAFETY comments**: Added to both `Mmap::map` unsafe blocks in `read/mod.rs`
4. **cache.rs unwrap fix**: `checked_sub().unwrap()` → `let-else` with `clear()` fallback
5. **`#[allow]` → `#[expect]`**: 8 instances in `types.rs`, 1 in `mcp.rs` (session.rs kept as `#[allow]` since lint is unfulfilled)
6. **`io_err` helper**: New helper in `error.rs`, updated 5 call sites in `read/mod.rs` and `edit.rs`
7. **`parse_tree` helper**: New in `treesitter.rs`, updated 5 call sites across `symbol.rs`, `callers.rs`, `callees.rs`, `code.rs`
8. **`walk_collect` helper**: New in `search/mod.rs`, updated 4 call sites (`symbol.rs` ×2, `content.rs`, `callers.rs`)
9. **Let-chain rewrites**: 13 locations across `mod.rs`, `treesitter.rs`, `search/mod.rs`, `code.rs`, `rank.rs`, `main.rs`, `classify.rs`, `map.rs`, `imports.rs`, `test_file.rs`, `callees.rs`, `callers.rs`, `symbol.rs`
