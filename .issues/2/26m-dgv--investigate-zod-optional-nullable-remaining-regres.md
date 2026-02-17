---
# 26m-dgv
title: Investigate zod_optional_nullable remaining regression
status: completed
type: task
priority: low
created_at: 2026-02-16T01:08:25Z
updated_at: 2026-02-16T02:28:22Z
parent: 4q3-09c
sync:
    github:
        issue_number: "22"
        synced_at: "2026-02-17T00:09:01Z"
---

## Problem

After the resolve_scope fix, zod_optional_nullable improved from +97% to +34% context regression. But it still regresses — glean uses 19 turns vs baseline 15, adding Glob (2), Bash (2), and glean_search (1) calls that baseline doesn't need.

## Investigation

- [ ] Check the new tool sequences — is the scope error now being returned and corrected, or is the model still using bad paths?
- [ ] Check if the glean_search call is actually useful or if the model ignores its results
- [ ] The baseline pattern (10 Greps + 4 Reads) is already efficient for this task. Can glean match it?

## Summary of Changes

No code changes needed. Root cause confirmed:

1. Model passes scope="benchmark/fixtures/repos/zod/..." — a relative path from the wrong root (cwd is already the zod repo root)
2. glean_search gets empty/wrong results
3. Model wastes 4-5 turns on Glob/Bash to discover directory structure
4. Falls back to Grep (same as baseline) but with 34% more context from wasted turns

The resolve_scope fix (already applied in parent epic) should give immediate error feedback on bad paths. Combined with expand=1 (just applied in 6bd-yu5), the overhead should drop. Need a benchmark re-run to verify.
