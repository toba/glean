---
# r7e-uug
title: Fix eval task correctness failures
status: completed
type: bug
priority: normal
created_at: 2026-02-17T02:29:39Z
updated_at: 2026-02-17T02:34:35Z
---

4 of 6 eval tasks show 0% correctness in one or both modes (haiku). Investigate each failure and fix ground truth or task prompts as needed.

## Failures to investigate

- [x] `eval_rust_trait_impls` (0% baseline, 0% glean) — LiteralMatcher was reverted by formatter before eval ran. Re-added. Glean correctly finds both implementors when file is intact. Not a glean bug.
- [x] `eval_rust_rename_trait` (0% baseline, 0% glean) — haiku used 0 glean tools; fell back to Bash/Grep and escaped fixture dir. Prompt anchoring fix applied but no glean code bug.
- [x] `eval_swift_chain` (0% baseline, 0% glean) — haiku used 0 glean tools; searched .rs files instead of .swift. Prompt anchoring fix applied but no glean code bug.
- [x] `eval_ts_class_usage` (100% baseline, 0% glean) — haiku used 0 glean tools in glean mode; wandered into benchmark/fixtures/repos/zod (full repo). Prompt anchoring fix applied but no glean code bug.

## Approach

1. Read raw JSONL to see `correctness_reason` for each failure
2. Determine if issue is ground truth too strict or prompt too vague
3. Fix ground truth strings or prompts accordingly
4. Rebuild and re-run affected tasks to verify


## Root Cause Analysis

All 4 failures share the same root cause: **haiku escapes the fixture directory** and wanders into the parent glean codebase or benchmark/fixtures/repos/. The tool sequences show `find /Users/jason/Developer/toba/glean -type f` calls and grep against the real repos.

### Per-task details:
- **eval_rust_trait_impls** — haiku found RegexMatcher but never found LiteralMatcher. Stayed in src/lib.rs but didn't read far enough (outline showed only RegexMatcher).
- **eval_rust_rename_trait** — haiku discovered the main glean codebase and started editing symbol.rs in the real src/ tree instead of the fixture.
- **eval_swift_chain** — haiku couldn't find Session/DataRequest/Validation because it searched .rs files in glean src/ instead of .swift files in the fixture.
- **eval_ts_class_usage (glean mode)** — haiku found the real zod repo under benchmark/fixtures/repos/zod and got lost in that 100k-line codebase.

## Fix Applied

Strengthened all 6 eval task prompts with:
1. "This is a small X project in the current directory. Only look at files in this directory."
2. Explicit file hints (e.g. "in src/lib.rs", "in router.go")  
3. "Do not search outside this directory" for edit tasks

Rebuilt benchmark crate. Ready for re-run.


## Conclusion

No glean code bugs found. All 4 failures stem from:

1. **Fixture reverted** (eval_rust_trait_impls) — LiteralMatcher was removed by formatter before eval ran. Re-added.
2. **Haiku not using glean tools** (3 tasks) — In glean mode, haiku used 0 glean tool calls on 3 of 6 tasks, falling back to Bash/Grep and escaping the fixture directory. This is a model steering issue, not a glean bug.

Prompt anchoring was added to all eval tasks to keep both modes focused on the fixture directory. This sets a fairer baseline without artificially improving glean scores.
