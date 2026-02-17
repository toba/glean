---
# ev6-226
title: Implement mini eval harness
status: completed
type: feature
priority: normal
created_at: 2026-02-17T02:15:14Z
updated_at: 2026-02-17T02:18:13Z
---

Add bench eval subcommand with 6 eval tasks targeting mini fixtures

## Summary of Changes

Implemented mini eval harness as a `bench eval` subcommand in the benchmark crate.

### Files modified:
- `tests/fixtures/mini-rust/src/lib.rs` — added `LiteralMatcher` implementor
- `benchmark/src/task.rs` — added `work_dir()` trait method
- `benchmark/src/run.rs` — use `work_dir()` in `run_single()`, skip repo validation for fixture tasks, accept budget parameter
- `benchmark/src/tasks/mod.rs` — added `mod eval` and `eval_tasks()` registry
- `benchmark/src/tasks/eval.rs` — **new**: 6 eval task definitions
- `benchmark/src/eval.rs` — **new**: eval runner with adjusted defaults (haiku, 1 rep, $0.10 budget)
- `benchmark/src/main.rs` — added `Eval` subcommand
