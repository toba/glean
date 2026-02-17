# Glean Benchmark

Automated evaluation of Glean's impact on LLM agent code navigation.

## Compared to tilth

[tilth](https://github.com/jahala/tilth) generally benchmarks navigation rather than edit tasks. That is important and helpful but I was also curious to see how I could help YOLO mode. Just go edit stuff, you fool!

Glean ditches `tilth`'s synthetic tasks and adds *edit* tasks against real codebases, such as

| Task | Lang | What it tests |
|------|--|---------------|
| `rg_binary_detection_default` | Rust | Trace binary detection from CLI flags through `HiArgs` to searcher construction, then change `quit` to `convert` |
| `zod_error_fallback` | TS | Trace the 5-level error message fallback chain, then change the ultimate fallback string |
| `af_acceptable_status` | Swift | Trace response validation to find where the status code range is defined, then widen it |
| `gin_binding_tag` | Go | Find where the validator tag name is configured and rename it |
| `rg_search_dispatch` | Rust | Trace generic type flow through `ReadByLine`/`MultiLine`/`Sink` across `glue.rs` |
| `af_request_chain` | Swift | Trace Session to DataRequest to adapter to URLSessionTask across multiple files |

## Token overhead

It is prudent to wonder if an MCP, plugin, skill or other buzzword is just more YouTube snake oil or might have actual benefits beyond their unavoidable token costs.

The fox in the henhouse (Claude) estimates that Glean will consume about 1,200 tokens per turn, which is a small fraction of what it's expected to save. But you should check your own context to verify. As always, trust your heart. I mean you're eyes.

## Metrics

Tilth has a clever formula for measuring benefits in terms of dollars, percentages, hit points and such. At least I assume its clever because it made me tired to read it.

Over here, I can't be bothered to rethink the existing, standard unit of LLM measurement, the token. 

| Model | Tasks | Runs | Baseline $/correct | glean $/correct | Change | Baseline acc | glean acc |
|---|---|---|---|---|---|---|---|
| Opus 4.6 | 26 | 156 | $0.61 | $0.66 | **+9%** | 85% | 76% |
| Sonnet 4.5 | 26 | — | — | — | — | — | — |
| Haiku 4.5 | 7 (forced) | 7 | $0.22 | $0.04 | **-82%** | 69% | 100% |

Opus is the only model with a full, statistically meaningful run. The Sonnet full run produced 148/156 errors (API/timeout), so no usable aggregate data. Haiku forced mode is a clear win but only 7 runs.

### Why "cost per correct answer"?

Raw cost comparison treats a wrong answer as a cheap success. It isn't — you paid for a response you can't use and still need the answer. The real question is: **how much do you expect to spend before you get a correct answer?**

This is a geometric retry model. If accuracy is `p`, you need `1/p` attempts on average before one succeeds. The expected cost is:

```
expected_cost = cost_per_attempt × (1 / accuracy)
```

**Cost per correct answer** (`total_spend / correct_answers`) computes this exactly. It's mathematically equivalent to `avg_cost / accuracy_rate` — not an arbitrary penalty, but the expected cost under retry.

## Sonnet 4.5

*These per-task results are from v0.3.2 with a smaller task set (11 tasks, Go + Rust only). A full 26-task Sonnet run was attempted but produced 148/156 errors (API timeouts), so no usable aggregate data exists for the current version.*

### Per-task results (v0.3.2, Go + Rust only)

Costs are 3-rep averages. Winner: accuracy difference first, then >=10% cost difference.

```
Task                                     Base    Glean   Delta  B✓  T✓  Winner
─────────────────────────────────────────────────────────────────────────────────
[E] gin_client_ip                        $0.18   $0.16    -12%  2/3 3/3  TILTH (acc)
[E] gin_context_next                     $0.05   $0.08    +55%  0/3 3/3  TILTH (acc)
[E] rg_flag_definition                   $0.09   $0.08     -9%  3/3 3/3  ~tie
[E] rg_lineiter_definition               $0.10   $0.09    -12%  1/3 1/3  TILTH ($)
[E] rg_lineiter_usage                    $0.21   $0.13    -38%  3/3 3/3  TILTH ($)
─────────────────────────────────────────────────────────────────────────────────
[M] gin_radix_tree                       $0.13   $0.15    +18%  0/3 1/3  TILTH (acc)
─────────────────────────────────────────────────────────────────────────────────
[H] gin_middleware_chain                 $0.38   $0.28    -27%  3/3 3/3  TILTH ($)
[H] gin_servehttp_flow                   $0.38   $0.32    -17%  3/3 3/3  TILTH ($)
[H] rg_search_dispatch                   $0.56   $0.49    -12%  3/3 3/3  TILTH ($)
[H] rg_trait_implementors                $0.23   $0.15    -38%  3/3 2/3  BASE (acc)
[H] rg_walker_parallel                   $0.25   $0.23     -6%  1/3 0/3  BASE (acc)
```

### By language

| Repo | Language | $/correct (B → T) | Accuracy (B → T) |
|---|---|---|---|
| ripgrep | Rust | $0.31 → $0.29 (-6%) | 78% → 67% |
| Gin | Go | $0.42 → $0.23 (-45%) | 53% → 87% |

Go sees the largest improvement: cost per correct answer drops 45% as accuracy jumps from 53% to 87%. Rust accuracy regresses (78% → 67%) driven by two tasks where the model flakes on specific reps, though cost per correct still improves slightly.

## Opus 4.6

26 tasks × 3 reps × 2 modes = 156 runs. The most statistically meaningful dataset.

**Aggregate:** Baseline 85% accuracy, glean 76%. Median cost $0.51 → $0.50 (-3%). Cost per correct answer $0.61 → $0.66 (**+9%**). Glean's small cost savings are wiped out by the accuracy regression.

Notable per-task results:

```
Task                                     Base    Glean   Delta  B✓  G✓  Winner
─────────────────────────────────────────────────────────────────────────────────
rg_search_dispatch                       $1.60   $1.74    +9%   2/3 3/3  GLEAN (acc)
zod_transform_pipe                       $0.60   $0.67   +13%   2/3 3/3  GLEAN (acc)
af_interceptor_protocol                  $0.41   $0.31   -25%   3/3 2/3  BASE (acc)
af_request_chain                         $1.44   $1.20   -17%   3/3 2/3  BASE (acc)
rg_binary_detection_default              $1.89   $1.10   -41%   3/3 1/3  BASE (acc)
rg_lineiter_definition                   $0.25   $0.18   -27%   3/3 0/3  BASE (acc)
rg_trait_implementors                    $0.18   $0.57  +219%   3/3 2/3  BASE (acc+$)
gin_servehttp_flow                       $0.69   $0.60   -13%   3/3 3/3  GLEAN ($)
zod_string_schema                        $0.97   $0.84   -13%   3/3 3/3  GLEAN ($)
af_session_config                        $0.68   $0.50   -26%   1/3 0/3  ~tie (both bad)
rg_walker_parallel                       $0.51   $0.66   +29%   0/3 0/3  ~tie (both fail)
```

Glean wins on 2 tasks by flipping accuracy (rg_search_dispatch, zod_transform_pipe). But it loses on 4 tasks where baseline was 3/3 and glean regressed (af_interceptor_protocol, rg_binary_detection_default, rg_lineiter_definition, rg_trait_implementors). The remaining 20 tasks are ties — mostly both 3/3 with cost within noise.

## Haiku 4.5 (71 runs)

Haiku was tested in three configurations: baseline (no glean), hybrid (glean available alongside built-in tools), and forced (built-in search tools removed).

| Mode | Runs | Accuracy | Avg cost | $/correct | Glean adoption |
|---|---|---|---|---|---|
| Baseline | 29 | 69% | $0.15 | $0.22 | — |
| Hybrid | 35 | 69% | $0.16 | $0.23 | 9% (3/35 runs) |
| **Forced** | **7** | **100%** | **$0.04** | **$0.04** | **100%** |

In hybrid mode, Haiku used glean tools in only 3 of 35 valid runs. It defaults to Bash/Grep/Read regardless of MCP instructions. Instruction tuning (moving directives to the top, using CRITICAL/MUST language) had no measurable effect on adoption.

In forced mode (`--disallowedTools "Bash,Grep,Glob"`), Haiku achieves 7/7 correct at $0.04 average — the cheapest correct answers in the entire benchmark. The same 7 tasks score 74% accuracy at $0.11 average in baseline.

## Cross-model analysis

### Tool adoption by model (glean mode)

| Model | glean_search/run | glean_read/run | Bash/run | Adoption rate |
|---|---|---|---|---|
| Haiku 4.5 | 0.2 | 0.1 | 5.9 | 9% |
| Sonnet 4.5 | 2.4 | 3.0 | 0.2 | 95% |
| Opus 4.6 | varies | varies | varies | ~mixed |

Sonnet and Opus adopt glean tools readily. Haiku ignores them unless forced. But adoption doesn't translate to better outcomes — Opus adopts glean aggressively and still regresses on accuracy.

### Where glean helps

**rg_search_dispatch (67% → 100% accuracy):** Glean helps Opus find the correct dispatch path on all 3 reps instead of 2/3. One of only two tasks where glean flips accuracy in the positive direction.

**zod_transform_pipe (67% → 100%):** Similar story — baseline flakes on 1 rep, glean gets all 3.

**Haiku forced mode (69% → 100%, -82% $/correct):** The clearest win in the entire benchmark, though with only 7 runs.

### Where glean hurts

**rg_lineiter_definition (100% → 0%):** The worst regression. Baseline gets it right every time; glean fails every time. glean_search returns a definition but the model doesn't extract the answer correctly from the structured output.

**rg_binary_detection_default (100% → 33%):** A hard task where glean's cost savings (-41%) are meaningless because accuracy craters.

**rg_trait_implementors (100% → 67%, +219% cost):** Glean costs 3x more and still loses accuracy. The model over-explores with glean tools on what should be a simple lookup.

### The honest picture

The full Opus benchmark — the most statistically meaningful data — shows glean slightly reduces raw cost but at the expense of accuracy. Cost per correct answer, the metric that matters, is 9% worse with glean. The Sonnet data is too old (v0.3.2) and incomplete to draw current conclusions. Haiku forced mode works well but is a narrow scenario with minimal data.



## Methodology

Each run invokes `claude -p` (Claude Code headless mode) with a code navigation question. Benchmarks are designed to run under a Claude Code subscription plan (no incremental API cost). Cost figures in reports are **estimated from token counts** using published Anthropic API pricing, not from actual API billing.

**Cost estimation:** The benchmark runner captures per-turn token breakdowns (input, output, cache creation, cache read) from `claude -p --output-format stream-json --verbose`. The analyzer multiplies these counts by published per-million-token rates:

| Model | Input | Output | Cache write | Cache read |
|---|---|---|---|---|
| Sonnet | $3.00 | $15.00 | $3.75 | $0.30 |
| Opus | $15.00 | $75.00 | $18.75 | $1.50 |
| Haiku | $0.80 | $4.00 | $1.00 | $0.08 |

To update pricing, edit the `Pricing` constants in `benchmark/src/analyze.rs`.

**Three modes:**
- **Baseline** — Claude Code built-in tools: Read, Edit, Grep, Glob, Bash
- **glean** — Built-in tools + glean MCP server (hybrid mode)
- **glean_forced** — glean MCP + Read/Edit only (Bash, Grep, Glob removed)

All modes use the same system prompt, $1.00 budget cap, and model. The agent explores the codebase and returns a natural-language answer. Correctness is checked against ground-truth strings that must appear in the response.

**Repos (pinned commits):**

| Repo | Language | Description |
|---|---|---|
| [Gin](https://github.com/gin-gonic/gin) | Go | HTTP framework |
| [ripgrep](https://github.com/BurntSushi/ripgrep) | Rust | Line-oriented search |
| [Alamofire](https://github.com/Alamofire/Alamofire) | Swift | HTTP networking library |
| [Zod](https://github.com/colinhacks/zod) | TypeScript | Schema validation |

**Difficulty tiers (7 tasks each, Sonnet only):**
- **Easy** — Single-file lookups, finding definitions, tracing short paths
- **Medium** — Cross-file tracing, understanding data flow, 2-3 hop chains
- **Hard** — Deep call chains, multi-file architecture, complex dispatch

### Running benchmarks

**Prerequisites:**
- Rust toolchain (for building the benchmark runner)
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) CLI (`claude`) installed and authenticated
- glean installed (`cargo install glean` or `brew install toba/tap/glean`)
- Git (for cloning benchmark repos)

**Build:**

```bash
cd benchmark && cargo build --release
```

**Setup:**

```bash
# Clone repos at pinned commits (~100MB total)
bench setup --repos

# Generate synthetic test repository
bench setup --synthetic
```

**Run:**

```bash
# All 26 tasks, baseline + glean, 3 reps, Sonnet
bench run --models sonnet --tasks all --modes all --reps 3

# Specific tasks
bench run --models sonnet --tasks gin_middleware_chain,rg_search_dispatch --reps 3

# Opus on hard tasks only
bench run --models opus --tasks all --repos ripgrep,gin --reps 3

# Haiku forced mode (built-in search tools removed)
bench run --models haiku --tasks all --modes glean_forced --reps 1

# Single mode only (skip baseline comparison)
bench run --models sonnet --tasks all --modes glean --reps 1
```

**Analyze:**

```bash
# Summarize results from a run
bench analyze results/benchmark_<timestamp>_<model>.jsonl

# Save report to file
bench analyze results/benchmark_<timestamp>.jsonl -o report.md

# Compare two runs (e.g. different versions)
bench compare results/old.jsonl results/new.jsonl
```

Results are written to `benchmark/results/benchmark_<timestamp>_<model>.jsonl`. Each line is a JSON object with task name, mode, cost, token counts, correctness, and tool sequence.

### Task definitions

Tasks are in `benchmark/src/tasks/`. Each implements the `Task` trait with `name()`, `prompt()`, `ground_truth()`, and optionally `repo()` and `task_type()`.

### Contributing benchmarks

We welcome benchmark contributions — more data makes the results more reliable.

**Adding results:** Run the benchmark suite on your machine and share the `.jsonl` file in a GitHub issue or PR. Different hardware, API regions, and model versions can all affect results.

**Adding tasks:** Add a new struct implementing the `Task` trait in the appropriate file under `benchmark/src/tasks/`, then register it in `benchmark/src/tasks/mod.rs`. Each task needs:
- `repo()`: which benchmark repo to use
- `prompt()`: the code navigation question
- `ground_truth()`: `GroundTruth` with required strings that must appear in a correct answer
- `task_type()`: `"read"` or `"navigate"`

Good tasks have unambiguous correct answers that can be verified by string matching. Avoid tasks where the answer depends on interpretation.

## Task design



## Version history

| Version | Changes | Cost/correct (Sonnet) |
|---|---|---|
| v0.2.1 | First benchmark | baseline |
| v0.3.0 | Callee footer, session dedup, multi-symbol search | -8% |
| v0.3.1 | Go same-package callees, map demotion | +12% (regression) |
| v0.3.2 | Map disabled, instruction tuning, multi-model benchmarks | **-26%** |

v0.3.1 regressed because the model overused glean_map (62% of losing tasks) and re-read files already shown in search results. v0.3.2 disabled map and added instruction guidance to stop re-reading expanded code.
