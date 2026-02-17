---
# 160-plv
title: Benchmark MCP config has hardcoded glean binary path
status: completed
type: bug
priority: normal
created_at: 2026-02-15T19:47:52Z
updated_at: 2026-02-15T19:49:07Z
sync:
    github:
        issue_number: "5"
        synced_at: "2026-02-17T00:08:58Z"
---

## Problem

`benchmark/fixtures/glean_mcp.json` was checked into the repo with a hardcoded absolute path (`/Users/flysikring/.cargo/bin/glean`). This breaks on any other machine — `bench setup --repos` would leave a stale config, and glean benchmark modes would reference a nonexistent binary.

## Fix

- [x] `setup.rs`: `bench setup --repos` now generates `glean_mcp.json` dynamically by resolving `glean` from PATH (using bare `"glean"` command name for portability), falling back to absolute path for local `target/{release,debug}/glean` builds
- [x] `run.rs`: Validates MCP config exists and binary is present before running glean modes; clear error message if stale
- [x] `.gitignore`: `benchmark/fixtures/glean_mcp.json` is now gitignored (machine-specific, generated)
- [x] Removed the hardcoded fixture from the repo

## Summary of Changes

Replaced the checked-in `glean_mcp.json` fixture with dynamic generation during `bench setup --repos`. The setup resolves the actual glean binary location (PATH → project build artifacts) and writes the MCP config. The run command validates the config before starting glean modes and gives actionable error messages if it's missing or stale.
