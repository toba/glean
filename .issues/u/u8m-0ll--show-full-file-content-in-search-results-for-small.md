---
# u8m-0ll
title: Show full file content in search results for small files
status: completed
type: feature
priority: high
created_at: 2026-02-17T02:51:37Z
updated_at: 2026-02-17T03:00:53Z
---

## Problem

When searching symbols in small files (~20 lines), glean returns outlines with placeholders like `let <prop>` and `fn validate` — hiding the actual code. The model then needs a follow-up Read call to see method bodies, which wastes a round-trip.

For the eval_swift_chain benchmark, every file (Session.swift, Request.swift, Validation.swift) is under 30 lines. Glean's outlines forced 3 extra Read calls that baseline didn't need (baseline just `cat`d the files).

## Expected

If a file is small enough (e.g. ≤ the 3500-token threshold already used in `read/`), search results should include the full definition body instead of an outline placeholder.

## Approach

In symbol search result formatting, check the file's token count. If the whole file is under the outline threshold, emit the full matched range instead of the collapsed outline. This avoids the read pipeline's outline-vs-full decision being at odds with what search returns.

## Evidence

eval_swift_chain: glean 6 turns / 5 tool calls (Read=3, Bash=2) vs baseline 3 turns / 2 tool calls (Bash=2). Same correctness, 82% more context, 147% more duration.

## Summary of Changes\n\nModified `format_matches()` in `src/search/mod.rs` to bypass the `expand_remaining` budget for files under `EXPAND_FULL_FILE_THRESHOLD` (800 tokens):\n- Stat file before expand check, compute `is_small_file`\n- Changed gate from `expand_remaining > 0` to `expand_remaining > 0 || is_small_file`\n- Only decrement `expand_remaining` for non-small files\n- 1 integration test verifying small files get code blocks even with expand=0
