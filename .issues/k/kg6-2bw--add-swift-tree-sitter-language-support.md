---
# kg6-2bw
title: Add Swift tree-sitter language support
status: completed
type: feature
priority: normal
created_at: 2026-02-15T17:47:59Z
updated_at: 2026-02-15T18:01:06Z
---

Add Swift as a supported language in glean.

## Context

glean uses tree-sitter for AST parsing, outline extraction, and symbol/definition detection. Swift is not currently supported.

## Tasks

- [ ] Add `Swift` variant to `Lang` enum in `types.rs`
- [ ] Add Swift file extension mapping (`.swift`)
- [ ] Initialize tree-sitter-swift grammar
- [ ] Add Swift outline extraction (functions, classes, structs, enums, protocols, extensions)
- [ ] Add Swift definition detection patterns for symbol search
- [ ] Add Swift caller detection patterns
- [ ] Add tests for Swift parsing and outline generation
- [ ] Add a Swift fixture file for testing

## Summary of Changes

Added full tree-sitter support for Swift, enabling AST-based outlines, symbol search, and callee detection.

### Files modified:
- **Cargo.toml** — Added `tree-sitter-swift = "0.7"` dependency
- **src/read/outline/code.rs** — Registered Swift grammar; added `swift_class_kind()` to disambiguate `class_declaration` (class/struct/enum/extension/actor); added `protocol_declaration`, `init_declaration`, `typealias_declaration`, `property_declaration` handling; expanded child collection to include Enum and Interface kinds; added 3 tests
- **src/search/treesitter.rs** — Added Swift definition kinds to `DEFINITION_KINDS`
- **src/search/callees.rs** — Added Swift callee query patterns for `call_expression`
- **src/search/symbol.rs** — Added Swift symbol search test
