---
# kvr-0co
title: Fix af_session_config and gin_client_ip task correctness
status: ready
type: task
priority: normal
created_at: 2026-02-16T01:08:16Z
updated_at: 2026-02-16T01:08:16Z
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
