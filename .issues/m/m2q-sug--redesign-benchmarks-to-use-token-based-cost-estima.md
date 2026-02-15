---
# m2q-sug
title: Redesign benchmarks to use token-based cost estimation instead of live API calls
status: completed
type: task
priority: normal
created_at: 2026-02-15T19:14:42Z
updated_at: 2026-02-15T19:20:02Z
---

## Problem

The current benchmark suite runs actual Claude API calls via `claude -p`, which costs real money per run. This makes it expensive to iterate on benchmarks or run them frequently.

## Proposed Approach

Instead of making live API calls, measure tokens expended for different scenarios and estimate cost based on token counts. Use the existing Claude Code plan (subscription) rather than paying per-API-call.

Key ideas:
- [x] Design a token measurement approach that works with Claude Code plan usage
- [x] Capture input/output token counts per scenario without incurring API costs
- [x] Estimate cost based on token pricing tables rather than actual spend
- [x] Maintain existing task definitions and ground-truth validation
- [x] Update benchmark runner and analysis tooling
- [x] Update benchmark/README.md with new methodology

## Open Questions

- How exactly to capture token counts — intercept from Claude Code output, parse JSONL logs, or instrument the MCP server?
- Should we still validate correctness (ground truth) or focus purely on token efficiency?
- What token pricing to use for cost estimation (published Anthropic API rates)?


## Summary of Changes

- Added model-aware pricing table in `analyze.rs` (Sonnet, Opus, Haiku) with per-million-token rates for input, output, cache creation, and cache read
- Reports now use token-estimated cost (`estimated_cost`) instead of API-reported `total_cost_usd`
- Added `find_median_run_by` helper to find median run by computed value
- Updated README methodology section to explain that costs are estimated from token counts, not live billing
- `total_cost_usd` is retained in JSONL output for reference but no longer used in reports
