---
# irr-1wq
title: Port Python benchmark suite to Rust
status: completed
type: task
priority: normal
created_at: 2026-02-15T18:16:57Z
updated_at: 2026-02-15T18:29:26Z
---

Replace ~1200 lines of Python benchmark code with a standalone Rust project at benchmark/. Create Cargo.toml, src/main.rs, config.rs, task.rs, tasks/*.rs, parse.rs, run.rs, analyze.rs, compare.rs. Delete all Python files. Keep fixtures and results.

## Summary of Changes

Ported the entire Python benchmark suite (~1,200 lines) to Rust as a standalone project at `benchmark/`.

### Created
- `benchmark/Cargo.toml` — standalone Rust project (not a workspace member)
- `benchmark/src/main.rs` — CLI entry point with clap subcommands: run, analyze, compare, setup
- `benchmark/src/config.rs` — models, repos, modes, paths
- `benchmark/src/task.rs` — Task trait + GroundTruth struct
- `benchmark/src/tasks/` — 26 task definitions across 5 files (synthetic, ripgrep, fastapi, gin, express)
- `benchmark/src/parse.rs` — stream-json output parser
- `benchmark/src/run.rs` — benchmark runner (subprocess management, JSONL output)
- `benchmark/src/analyze.rs` — markdown report generation from JSONL
- `benchmark/src/compare.rs` — diff two JSONL result files
- `benchmark/src/setup.rs` — repo cloning + synthetic repo generation
- `benchmark/src/synthetic_content/` — 15 embedded Python files for synthetic repo

### Deleted
- All Python files: run.py, analyze.py, analyze_exploration.py, compare_versions.py, config.py, parse.py
- All Python task files: tasks/*.py
- Fixture scripts: fixtures/setup_repos.py, fixtures/setup.py, fixtures/reset.py

### Kept
- fixtures/glean_mcp.json, results/, README.md (updated with new commands)

### Verification
- `cargo build` — compiles
- `cargo clippy -- -D warnings` — clean
- `cargo fmt --check` — clean
