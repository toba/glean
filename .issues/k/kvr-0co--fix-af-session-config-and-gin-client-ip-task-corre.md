---
# kvr-0co
title: Fix af_session_config and gin_client_ip task correctness
status: completed
type: task
priority: normal
created_at: 2026-02-16T01:08:16Z
updated_at: 2026-02-16T02:26:01Z
parent: 4q3-09c
---

## Problem

Two tasks fail in both modes, suggesting the tasks themselves have issues:

- **af_session_config**: 1/3 baseline, 0/3 glean — always 'Missing: Session.swift'. The model finds and reads Session.swift but doesn't echo the filename in its response.
- **gin_client_ip**: 0/3 baseline, 1/3 glean — always 'Missing: trustedCIDRs'. The model may not be finding this specific field.
- **rg_walker_parallel**: 0/3 both modes — the model can't locate walk.rs in the workspace structure.
- **gin_radix_tree**: 1/3 both modes — 'Missing: tree.go'

## Fix Options

- [ ] af_session_config: Same filename-echoing problem as zod tasks. Remove file path hint from prompt or adjust required_strings
- [ ] gin_client_ip: Check if 'trustedCIDRs' actually exists in the gin fixture (spelling, casing)
- [ ] rg_walker_parallel: Check if walk.rs exists and if the prompt is too vague
- [ ] gin_radix_tree: Check if tree.go exists and review correctness check

## Summary of Changes

**af_session_config**: Removed "Session.swift" from required_strings. The prompt tells the model to "Find the Session class" so it reads Session.swift but describes it without echoing the filename. Same pattern as the zod fixes already applied.

**gin_client_ip**: Relaxed "func (c *Context) ClientIP" to just "ClientIP" (model may paraphrase the signature). Added explicit hint in prompt to "Trace into the Engine to show how trustedCIDRs is used" since the field is in gin.go, not context.go.

**gin_radix_tree**: Removed "tree.go" from required_strings. The model finds the node struct and methods but does not echo the filename.

**rg_walker_parallel**: Removed "walk.rs" from required_strings. The prompt says "In the ignore crate" which is specific enough. The model finds WalkParallel/ParallelVisitor but does not always echo the filename.

**Pattern**: Filename-as-required-string is fragile because models read the file and describe its contents without restating the path. Required strings should test for code-level understanding (struct names, method names, concepts) not filenames.
