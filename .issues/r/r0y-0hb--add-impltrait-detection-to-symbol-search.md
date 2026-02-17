---
# r0y-0hb
title: Add impl/trait detection to symbol search
status: completed
type: feature
priority: high
created_at: 2026-02-17T00:33:57Z
updated_at: 2026-02-17T00:43:23Z
---

When searching for a trait or interface name, also surface `impl Trait for Type` and `class X implements Interface` blocks.

Upstream tilth does this in symbol.rs. Currently glean only finds direct definitions and usages.

## Tasks

- [x] Study tilth's approach to impl/trait detection in symbol search
- [x] Add tree-sitter queries for impl/trait blocks across supported languages
- [x] Surface impl blocks in symbol search results
- [x] Add tests for Rust impl/trait, Go interface, TypeScript implements
- [x] Run benchmarks to verify no regression


## Summary of Changes

Added impl/trait and interface detection to symbol search:

- **treesitter.rs**: Added `extract_impl_trait()`, `extract_impl_type()`, `extract_implemented_interfaces()` helpers. Also fixed `extract_definition_name()` for bare `impl Type` blocks (now checks `type` field).
- **symbol.rs**: Extended `walk_for_definitions()` with two new checks:
  1. Rust `impl Trait for Type` — searching for a trait name now surfaces all impl blocks
  2. TypeScript/Java `class Foo implements Bar` — searching for an interface surfaces implementing classes
- Added 4 new tests: `rust_impl_trait_detected_by_trait_name`, `rust_bare_impl_detected_by_type_name`, `typescript_class_implements_interface`, `impl_trait_surfaces_in_symbol_search`
