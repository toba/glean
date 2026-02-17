---
# 4q3-09c
title: Improve glean benchmark results for Opus
status: in-progress
type: epic
priority: high
created_at: 2026-02-15T22:38:32Z
updated_at: 2026-02-16T01:14:01Z
sync:
    github:
        issue_number: "12"
        synced_at: "2026-02-17T00:08:58Z"
---

Based on analysis of benchmark run `benchmark_20260215_151922_opus.jsonl` (6 tasks, 1 rep, Opus 4.6).

## Current Results

| Task | Baseline | Glean | Cost Δ | B✓ | G✓ |
|------|----------|-------|--------|----|----|
| gin_middleware_chain | $1.05 | $0.96 | -9% | ✓ | ✓ |
| gin_servehttp_flow | $0.70 | $0.86 | +24% | ✓ | ✓ |
| rg_binary_detection_default | $1.99 | $0.89 | -55% | ✗ | ✓ |
| rg_search_dispatch | $1.71 | $0.61 | -64% | ✗ | ✗ |
| rg_walker_parallel | $0.74 | $0.63 | -15% | ✗ | ✗ |
| zod_error_fallback | $0.59 | $0.55 | -7% | ✓ | ✓ |

**Overall: 3/6 baseline correct, 4/6 glean correct. Glean -18% median cost.**

## Problems to Investigate

### 1. Over-exploration on small codebases (Gin tasks)

**gin_middleware_chain**: Glean uses 15 turns / 14 tool calls vs baseline's 6 turns / 5 calls. Makes 6 `glean_search` + 5 `Grep` calls for a task baseline solves with 3 file reads. Context balloons 119K → 391K tokens. Cost savings (-9%) are marginal given the overhead.

**gin_servehttp_flow**: Glean is 24% MORE expensive ($0.70 → $0.86), 7 extra turns. Baseline uses 13 targeted greps efficiently. Glean scatters across grep (9) + glean_search (2) + reads (7) + bash (3).

**Root cause hypothesis**: Opus uses glean tools aggressively even when the codebase is small enough that direct file reads suffice. The model doesn't adapt search strategy to repo size.

**Potential fixes**:
- [ ] Investigate whether glean's MCP tool descriptions encourage over-use on small repos
- [ ] Consider adding repo-size-aware hints to glean_search results (e.g., "this repo has N files")
- [ ] Review if glean_search returns are too granular for small codebases, prompting follow-up searches

### 2. Both ripgrep "hard" navigation tasks fail in both modes

**rg_search_dispatch** (missing `ReadByLine`): Both modes fail. Baseline reads `mod.rs` and `glue.rs` but misses the line-by-line reader. Glean mode doesn't even find `glue.rs` — it gets confused about whether ripgrep code is in the repo at all.

**rg_walker_parallel** (missing `walk.rs`): Both modes fail. The model finds the right crate but can't locate the specific `walk.rs` file containing the parallel walker implementation. Previous Opus run had glean solving this 2/3 times — regression from 67% → 0%.

**Root cause hypothesis**: These tasks require navigating a large Rust workspace (`crates/*/src/`) where file discovery is the bottleneck. The model struggles to enumerate and find the right source files in deeply nested structures.

**Potential fixes**:
- [ ] Investigate rg_walker_parallel regression — compare tool sequences from previous passing runs vs this failing run
- [ ] Check if glean_files or directory listing gives adequate visibility into crate structure
- [ ] Test whether glean_search with broader scope queries would surface `walk.rs` and `glue.rs`
- [ ] Consider improving outline/file discovery for Rust workspace layouts

### 3. rg_search_dispatch: glean mode gets confused about repo boundaries

In the glean run, the model first searches the glean repo itself for ripgrep symbols, then realizes ripgrep is a benchmark fixture. It wastes turns on `glean_search` queries scoped to `.` (the glean repo root) before narrowing to the fixture. This suggests the model doesn't understand the benchmark fixture repo layout.

**Potential fixes**:
- [ ] Review if the benchmark prompt provides adequate context about where to search
- [ ] Consider if glean_search scope parameter could be made clearer/more prominent

### 4. Statistical confidence is low (1 rep)

All per-task conclusions are drawn from single runs. The previous 3-rep Opus run showed different patterns (e.g., rg_walker_parallel was 0/3 baseline → 2/3 glean, now both fail).

- [x] Re-run with 3 reps to get reliable per-task accuracy and cost distributions
- [ ] Compare 3-rep results against previous Opus run to identify true regressions vs variance

### 5. Cost breakdown insights

Glean's biggest cost advantage comes from **reduced cache creation** (baseline wastes on broad file reads that get cached but aren't useful). Examples:
- rg_binary_detection_default: cache_create drops $0.81 → $0.39
- rg_search_dispatch: cache_create drops $1.32 → $0.29

Glean's cost disadvantage comes from **more turns** inflating cache_read costs:
- gin_middleware_chain: cache_read increases $0.04 → $0.15
- gin_servehttp_flow: cache_read increases $0.16 → $0.30

**Potential optimization**: Reduce unnecessary follow-up queries that re-read already-expanded context (session dedup should help but may not be sufficient for Opus's exploration pattern).

- [ ] Audit session dedup effectiveness for Opus — is it actually preventing re-expansion?
- [ ] Check if glean_search results include enough context to avoid follow-up reads

## Comparison to Previous Opus Run (README, 3 reps)

| Task | Previous delta | This run delta | Change |
|------|---------------|----------------|--------|
| gin_middleware_chain | glean -32% | glean -9% | worse |
| gin_servehttp_flow | base -29% | base +24% | consistent direction |
| rg_search_dispatch | glean -21% | glean -64% | better |
| rg_walker_parallel | glean 0/3→2/3 | both 0/1 | regression |

New tasks: rg_binary_detection_default (strong glean win), zod_error_fallback (tie).

## Partial 3-Rep Results (af + zod only, gin/rg lost to runner bugs)

43 valid runs recovered. gin and ripgrep results lost to concurrent-write corruption + runner panic.

### Per-Task Summary

| Task | Baseline ctx | Glean ctx | Ctx Δ | Cost Δ | B✓ | G✓ |
|------|-------------|-----------|-------|--------|----|-----|
| af_acceptable_status | 202K | 121K | -40% | -28% | 3/3 | 2/3 |
| af_upload_multipart | 59K | 74K | +27% | -8% | 3/3 | 3/3 |
| af_interceptor_protocol | — | 170K | — | — | — | 1/3 |
| zod_string_schema | 418K | 298K | -29% | **-30%** | 3/3 | 3/3 |
| zod_error_fallback | 334K | 298K | -11% | **-21%** | 3/3† | 3/3 |
| zod_parse_flow | 395K | 297K | -25% | -1% | 3/3 | 3/3† |
| zod_transform_pipe | 297K | 302K | +2% | -7% | 3/3 | 3/3 |
| zod_optional_nullable | 194K | 382K | **+97%** | **+41%** | 3/3 | 3/3 |
| zod_error_handling | — | — | — | — | — | — |
| zod_discriminated_union | — | — | — | — | 0/3† | 0/1† |

†partial reps due to corrupted lines

**Summary (median of medians): glean -18% cost, -17% turns, flat context**

### New Problems Identified

#### 6. zod_optional_nullable: glean regression (+97% context, +41% cost)

Glean used 19 turns vs baseline 12. Tool breakdown shows glean added `Glob` (2), `Bash` (4), and `mcp__glean__glean_search` (1) on top of the same Grep/Read pattern. The MCP tool did not help — the agent wandered.

- [ ] Investigate tool sequences for zod_optional_nullable glean runs to identify what triggered the extra exploration
- [ ] Check if glean_search returned unhelpful results that sent the agent down rabbit holes

#### 7. zod_discriminated_union: 0% correctness in both modes

Always fails with "Missing: schemas.ts". The task may be too hard, or the correctness check expectations may not match what Opus can realistically find.

- [ ] Review zod_discriminated_union task definition and correctness checks
- [ ] Determine if the task needs adjustment or if it reveals a real capability gap

#### 8. af_interceptor_protocol: low correctness (33% glean, no baseline data)

Fails with "Missing: RequestInterceptor" or "Missing: RequestAdapter". Only glean runs available.

- [ ] Review af_interceptor_protocol task definition and correctness checks

### Infrastructure Fixes Applied

- [x] Fixed run_full.sh bash 3.2 compatibility (declare -A → parallel arrays)
- [x] Fixed UTF-8 truncation panic in run.rs (byte index on char boundary)
- [x] Added --output flag to bench run; run_full.sh now uses per-repo temp files merged at end

### Analysis: Root Causes and Fixes Before Re-run

#### A. glean_search scope path mismatch (FIXED)

**Root cause**: When Claude runs with cwd set to the repo (e.g., `fixtures/repos/zod/`), it passes scope paths like `benchmark/fixtures/repos/zod/packages/...` to glean_search — a relative path from the wrong root. `resolve_scope` silently canonicalized this to a non-existent path, returning empty results. The model then spent 6-8 turns doing `ls`, `pwd`, `Glob` to figure out the directory layout.

**Fix applied**: `resolve_scope` now returns an error with the actual cwd when the scope path does not exist. This gives the model immediate feedback to correct the path.

**Impact**: Should eliminate the +97% context regression on `zod_optional_nullable` and similar overhead on other tasks that use `glean_search`.

#### B. Correctness checks require filenames the model does not echo (NEEDS FIX)

Tasks `zod_discriminated_union`, `zod_error_handling` have required_strings like `"schemas.ts"`, `"errors.ts"`. The prompts already tell the model where to look (e.g., "Find ... in packages/zod/src/v4/core/errors.ts"), so the model reads the file and describes it without repeating the filename. Fix: remove filename from prompt to make it a real discovery task.

#### C. af_interceptor_protocol: 1-turn failures (flaky, not fixable)

2 of 3 glean runs produced only 1 turn with minimal output. Likely a model flake — the 3rd run worked perfectly (9 turns, all checks passed). Not actionable.

- [x] Fix resolve_scope to error on non-existent paths (glean source)
- [x] Fix zod_discriminated_union: remove file path hint from prompt
- [x] Fix zod_error_handling: remove file path hint from prompt

## Full 3-Rep Results (156 runs, all 4 repos)

### Overall (median of medians)

| Metric | baseline | glean | Improvement |
|--------|----------|-------|-------------|
| Context tokens | 197K | 184K | -7% |
| Turns | 11 | 9 | -18% |
| Tool calls | 10 | 8 | -20% |
| Est. cost | $0.51 | $0.50 | -3% |

### Per-Task Results

| Task | Ctx Δ | Cost Δ | Turns Δ | B✓ | G✓ |
|------|-------|--------|---------|-----|-----|
| zod_string_schema | -37% | -13% | -40% | 3/3 | 3/3 |
| gin_client_ip | -36% | -5% | -55% | 0/3 | 1/3 |
| af_session_config | -25% | -26% | -18% | 1/3 | 0/3 |
| rg_binary_detection | -23% | -41% | -12% | 3/3 | 1/3 |
| gin_servehttp_flow | -20% | -13% | -50% | 3/3 | 3/3 |
| zod_parse_flow | -17% | -5% | -13% | 3/3 | 3/3 |
| rg_search_dispatch | -17% | +9% | -8% | 2/3 | 3/3 |
| af_interceptor_protocol | -14% | -25% | -57% | 3/3 | 2/3 |
| zod_discriminated_union | -13% | -21% | -21% | 3/3 | 3/3 |
| zod_error_fallback | +11% | +22% | -10% | 3/3 | 3/3 |
| af_acceptable_status | -8% | -13% | -20% | 3/3 | 3/3 |
| af_response_validation | -6% | +5% | -18% | 3/3 | 3/3 |
| rg_walker_parallel | -1% | +29% | -9% | 0/3 | 0/3 |
| gin_radix_tree | +8% | +8% | 0% | 1/3 | 1/3 |
| af_upload_multipart | +10% | +5% | 0% | 3/3 | 3/3 |
| rg_lineiter_definition | +14% | -27% | 0% | 3/3 | 0/3 |
| af_request_chain | +30% | -17% | -27% | 3/3 | 2/3 |
| zod_error_handling | +32% | +26% | -14% | 3/3 | 3/3 |
| zod_optional_nullable | +34% | +10% | +27% | 3/3 | 3/3 |
| zod_transform_pipe | +37% | +13% | -14% | 2/3 | 3/3 |
| rg_lineiter_usage | +39% | -24% | +17% | 3/3 | 2/3 |
| gin_context_next | +55% | +42% | +20% | 3/3 | 3/3 |
| gin_binding_tag | +61% | +65% | +40% | 3/3 | 3/3 |
| rg_flag_definition | +83% | +68% | +67% | 3/3 | 3/3 |
| gin_middleware_chain | +99% | +50% | 0% | 3/3 | 3/3 |
| rg_trait_implementors | +248% | +219% | +133% | 3/3 | 2/3 |

## Fixes Applied (Round 2)

### glean source changes
- [x] `resolve_scope` returns error on non-existent paths with cwd hint (src/mcp.rs)
- [x] Added `.build` to SKIP_DIRS — Swift Package Manager build artifacts excluded (src/search/mod.rs)
- [x] Added `.xcodeproj` / `.xcworkspace` to SKIP_DIRS — pbxproj no longer pollutes search (src/search/mod.rs)
- [x] Removed stale `.build/` directory from alamofire fixture

### Benchmark changes
- [x] zod_discriminated_union prompt: removed file path hint → now 100% correct (was 0%)
- [x] zod_error_handling prompt: removed file path hint
- [x] UTF-8 truncation panic fix (benchmark/src/run.rs)
- [x] Added --output flag to bench run (benchmark/src/{main,run}.rs)
- [x] run_full.sh: per-repo temp files, bash 3.2 compatible

## Remaining Issues

### Needs new issues
- Documentation Markdown examples ranked above actual source definitions in Alamofire search
- Glean expand=2 default too aggressive for small repos (11x context overhead vs grep)
- Test file usages not deprioritized in search results
