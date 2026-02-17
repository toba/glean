---
# l4i-5cz
title: 'Fix eval correctness checker: check all turns, not just last'
status: completed
type: bug
priority: high
created_at: 2026-02-17T02:44:07Z
updated_at: 2026-02-17T02:46:53Z
---

The eval correctness checker has multiple bugs causing false negatives.

## Problems

### 1. `result_text` only captures the last assistant turn
`parse.rs:145` overwrites `result_text` each turn, so only the final assistant message is checked for `required_strings`. If the model mentions required terms in earlier turns but not the last one, correctness fails.

Affected tasks:
- `eval_rust_rename_trait/glean`: Model did the rename but "PatternMatcher" wasn't in the final text
- `eval_rust_trait_impls` (both modes): `LiteralMatcher` found but not mentioned in last turn

**Fix:** Accumulate all assistant text across all turns for correctness checking. Either concatenate all turn texts into `result_text`, or add a separate `all_text` field.

### 2. Go fixture tasks: model gets confused about working directory
Go tasks (`eval_go_rename_method`, `eval_go_request_flow`) use `fixture_dir("mini-go")` and `work_dir()` sets `current_dir` correctly, but the model responds as if it's in the glean root. Need to verify:
- That `tests/fixtures/mini-go` exists and has the expected files
- That the system prompt or task prompt doesn't leak the parent project context

### 3. Edit task correctness only checks git diff, not tool call content
For edit tasks like `eval_go_rename_method`, the model may successfully edit files but the required string ("Continue") only needs to appear in the diff — which it does check. But `required_strings` is also checked against `result_text`, which may not mention it. The two checks (text + diff) should be independent.

## TODO

- [x] Fix `parse.rs` to accumulate all assistant text across turns (not just last turn)
- [x] Add `all_text` or change `result_text` semantics so correctness checks all turns
- [x] Verify Go fixtures exist and work correctly (`tests/fixtures/mini-go/`) — files exist and are correct
- [x] Investigate why models think they're in wrong directory for fixture tasks — Claude Code inherits parent `.git`/`CLAUDE.md` context; not a code bug, design constraint of in-repo fixtures
- [x] For edit tasks, check required_strings against diff output too (not just assistant text)


## Summary of Changes

### `benchmark/src/parse.rs`
- Changed `result_text` from last-turn-only to accumulating all assistant text across all turns via `all_text_parts: Vec<String>` joined at the end.

### `benchmark/src/task.rs`
- Moved forbidden-strings check before required-strings check.
- For edit tasks, `required_strings` are now checked against both assistant text AND `git diff` output — a match in either counts.
- This fixes false negatives where the model performed the edit correctly but didn't repeat the pattern name in its final response.

### Go fixture directory issue
- Verified `tests/fixtures/mini-go/` exists with correct files (`context.go`, `router.go`, `middleware.go`, `go.mod`).
- The model confusion is because Claude Code reads `.git` and `CLAUDE.md` from the parent glean project. This is inherent to in-repo fixtures, not a code bug.

### Upstream (jahala/tilth)
- Has the exact same bugs in `benchmark/parse.py` and `benchmark/tasks/base.py`.
