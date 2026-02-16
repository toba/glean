---
# 26m-dgv
title: Investigate zod_optional_nullable remaining regression
status: ready
type: task
priority: low
created_at: 2026-02-16T01:08:25Z
updated_at: 2026-02-16T01:08:25Z
parent: 4q3-09c
---

## Problem

After the resolve_scope fix, zod_optional_nullable improved from +97% to +34% context regression. But it still regresses — glean uses 19 turns vs baseline 15, adding Glob (2), Bash (2), and glean_search (1) calls that baseline doesn't need.

## Investigation

- [ ] Check the new tool sequences — is the scope error now being returned and corrected, or is the model still using bad paths?
- [ ] Check if the glean_search call is actually useful or if the model ignores its results
- [ ] The baseline pattern (10 Greps + 4 Reads) is already efficient for this task. Can glean match it?
