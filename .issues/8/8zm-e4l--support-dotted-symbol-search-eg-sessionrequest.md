---
# 8zm-e4l
title: Support dotted symbol search (e.g. Session.request)
status: completed
type: feature
priority: high
created_at: 2026-02-17T02:51:25Z
updated_at: 2026-02-17T03:00:53Z
---

## Problem

`glean "Session.request"` returns 0 matches. The model has to search for `Session` and `request` separately and mentally join them, wasting a round-trip.

## Expected

`Session.request` should be parsed as "method `request` on type `Session`" and return the definition of `func request(_ url: String) -> DataRequest` scoped to the `Session` class.

## Approach

In `classify.rs` or symbol search, detect dotted identifiers (`Foo.bar`) and split into type + member. Search for the member name, then filter results to those inside the matching type's scope.

## Evidence

eval_swift_chain benchmark: glean took 6 turns vs baseline's 3, partly because the model couldn't directly query `Session.request`.

## Summary of Changes\n\nAdded dotted symbol search (e.g. `Session.request`) to `src/search/symbol.rs`:\n- `split_dotted_query()` — splits on single dot, rejects empty parts or multiple dots\n- `is_inside_type()` — walks parent chain for type container nodes\n- `search_dotted()` — dispatched from `search()` when query contains a dot\n- `find_definitions_dotted()` + `find_defs_treesitter_dotted()` + `walk_for_definitions_dotted()` — tree-sitter definition detection filtered by enclosing type\n- 5 unit tests + 1 integration test against mini-swift fixture
