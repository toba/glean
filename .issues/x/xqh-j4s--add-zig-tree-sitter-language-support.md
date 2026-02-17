---
# xqh-j4s
title: Add Zig tree-sitter language support
status: completed
type: feature
priority: normal
created_at: 2026-02-15T20:30:24Z
updated_at: 2026-02-15T20:36:13Z
sync:
    github:
        issue_number: "7"
        synced_at: "2026-02-17T00:08:59Z"
---

Add Zig as a supported language in glean's tree-sitter pipeline.

## Todo
- [x] Add `tree-sitter-zig` dependency to Cargo.toml
- [x] Add `Zig` variant to `Lang` enum in types.rs
- [x] Add `.zig` extension mapping in read/mod.rs
- [x] Wire up grammar in outline_language() in read/outline/code.rs
- [x] Add outline extraction queries for Zig AST nodes
- [x] Add definition detection patterns in search/symbol.rs
- [x] Add callee extraction query in search/callees.rs
- [x] Verify it compiles and tests pass
