---
# s1j-47u
title: Replace Python benchmark suite with non-Python alternative
status: completed
type: task
priority: normal
created_at: 2026-02-15T18:10:57Z
updated_at: 2026-02-15T19:34:11Z
---

The `benchmark/` directory currently contains Python scripts for benchmarking glean against real repos (Express, FastAPI, Gin, ripgrep). Replace this with a non-Python approach.

## Current Python files
- `run.py` — main benchmark runner
- `analyze.py` — results analysis
- `analyze_exploration.py` — exploration analysis
- `compare_versions.py` — version comparison
- `config.py` — benchmark configuration
- `parse.py` — output parsing

Also has: `fixtures/`, `results/`, `tasks/`, `README.md`

## TODO
- [ ] Decide on replacement approach (shell scripts, Rust bench harness, hyperfine, etc.)
- [ ] Reimplement benchmark suite
- [ ] Remove all Python files from `benchmark/`
- [ ] Update `CLAUDE.md` benchmarks section

## Summary of Changes\n\nAlready completed in commit 539edd1. Python benchmark suite was fully replaced with a Rust implementation in `benchmark/src/`. All Python files removed.
