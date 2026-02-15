use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Anthropic Claude pricing (per million tokens).
const PRICE_CACHE_CREATION: f64 = 3.75;
const PRICE_CACHE_READ: f64 = 0.30;
const PRICE_OUTPUT: f64 = 15.00;
const PRICE_INPUT: f64 = 3.00;

pub fn load_results(path: &Path) -> Vec<Value> {
    let content = fs::read_to_string(path).expect("Failed to read results file");
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

fn get_f64(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn get_u64(v: &Value, key: &str) -> u64 {
    v.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn get_str<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

fn get_bool(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}

struct CostBreakdown {
    cache_creation_cost: f64,
    cache_read_cost: f64,
    output_cost: f64,
    input_cost: f64,
}

fn compute_cost_breakdown(run: &Value) -> CostBreakdown {
    CostBreakdown {
        cache_creation_cost: get_u64(run, "cache_creation_tokens") as f64 * PRICE_CACHE_CREATION
            / 1_000_000.0,
        cache_read_cost: get_u64(run, "cache_read_tokens") as f64 * PRICE_CACHE_READ / 1_000_000.0,
        output_cost: get_u64(run, "output_tokens") as f64 * PRICE_OUTPUT / 1_000_000.0,
        input_cost: get_u64(run, "input_tokens") as f64 * PRICE_INPUT / 1_000_000.0,
    }
}

fn format_cost_breakdown(c: &CostBreakdown) -> String {
    format!(
        "  cache_create=${:.3} cache_read=${:.3} output=${:.3} input=${:.3}",
        c.cache_creation_cost, c.cache_read_cost, c.output_cost, c.input_cost
    )
}

fn format_cost_delta(b: &CostBreakdown, g: &CostBreakdown) -> String {
    let dc = g.cache_creation_cost - b.cache_creation_cost;
    let dr = g.cache_read_cost - b.cache_read_cost;
    let do_ = g.output_cost - b.output_cost;
    let di = g.input_cost - b.input_cost;
    format!(
        "  \u{0394}cache_create={:+.3} \u{0394}cache_read={:+.3} \u{0394}output={:+.3} \u{0394}input={:+.3}",
        dc, dr, do_, di
    )
}

fn group_by<'a>(results: &'a [Value], keys: &[&str]) -> HashMap<Vec<String>, Vec<&'a Value>> {
    let mut groups: HashMap<Vec<String>, Vec<&Value>> = HashMap::new();
    for r in results {
        if r.get("error").is_some() {
            continue;
        }
        let key: Vec<String> = keys.iter().map(|k| get_str(r, k).to_string()).collect();
        groups.entry(key).or_default().push(r);
    }
    groups
}

struct Stats {
    median: f64,
    _mean: f64,
    _stdev: f64,
    _min: f64,
    _max: f64,
}

fn compute_stats(values: &[f64]) -> Stats {
    if values.is_empty() {
        return Stats {
            median: 0.0,
            _mean: 0.0,
            _stdev: 0.0,
            _min: 0.0,
            _max: 0.0,
        };
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = sorted[sorted.len() / 2];
    let mean = sorted.iter().sum::<f64>() / sorted.len() as f64;
    let stdev = if sorted.len() > 1 {
        let variance =
            sorted.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (sorted.len() - 1) as f64;
        variance.sqrt()
    } else {
        0.0
    };
    Stats {
        median,
        _mean: mean,
        _stdev: stdev,
        _min: sorted[0],
        _max: *sorted.last().unwrap(),
    }
}

fn ascii_sparkline(values: &[u64]) -> String {
    if values.is_empty() {
        return String::new();
    }
    let lo = *values.iter().min().unwrap();
    let hi = *values.iter().max().unwrap();
    if lo == hi {
        return "\u{2584}".repeat(values.len());
    }
    let chars = [
        ' ', '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}',
        '\u{2588}',
    ];
    values
        .iter()
        .map(|&v| {
            let idx = ((v - lo) as f64 / (hi - lo) as f64 * 8.0) as usize;
            chars[idx.min(8)]
        })
        .collect()
}

fn format_delta(baseline: f64, glean: f64) -> String {
    if baseline == 0.0 {
        return "\u{2014}".into();
    }
    let pct = ((glean - baseline) / baseline) * 100.0;
    if pct > 0.0 {
        format!("+{pct:.0}%")
    } else {
        format!("{pct:.0}%")
    }
}

fn find_median_run<'a>(runs: &'a [&Value], metric: &str) -> &'a Value {
    if runs.is_empty() {
        return &Value::Null;
    }
    let mut sorted: Vec<&Value> = runs.to_vec();
    sorted.sort_by(|a, b| get_f64(a, metric).partial_cmp(&get_f64(b, metric)).unwrap());
    sorted[sorted.len() / 2]
}

fn merge_tool_calls(runs: &[&Value]) -> HashMap<String, f64> {
    let mut all_tools: Vec<String> = Vec::new();
    for run in runs {
        if let Some(tc) = run.get("tool_calls").and_then(Value::as_object) {
            for k in tc.keys() {
                if !all_tools.contains(k) {
                    all_tools.push(k.clone());
                }
            }
        }
    }
    let mut result = HashMap::new();
    for tool in &all_tools {
        let mut counts: Vec<f64> = runs
            .iter()
            .map(|r| {
                r.get("tool_calls")
                    .and_then(|tc| tc.get(tool.as_str()))
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0)
            })
            .collect();
        counts.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = counts[counts.len() / 2];
        result.insert(tool.clone(), median);
    }
    result
}

pub fn generate_report(results: &[Value]) -> String {
    let valid: Vec<&Value> = results
        .iter()
        .filter(|r| r.get("error").is_none())
        .collect();
    let error_count = results.len() - valid.len();

    if valid.is_empty() {
        return if results.is_empty() {
            "# Error\n\nNo valid results found in file.\n".into()
        } else {
            format!("# Error\n\nAll {} runs failed.\n", results.len())
        };
    }

    let mut all_models: Vec<&str> = valid.iter().map(|r| get_str(r, "model")).collect();
    all_models.sort();
    all_models.dedup();
    let mut all_tasks: Vec<&str> = valid.iter().map(|r| get_str(r, "task")).collect();
    all_tasks.sort();
    all_tasks.dedup();
    let mut all_modes: Vec<&str> = valid.iter().map(|r| get_str(r, "mode")).collect();
    all_modes.sort();
    all_modes.dedup();
    let mut all_repos: Vec<&str> = valid
        .iter()
        .map(|r| {
            let s = get_str(r, "repo");
            if s.is_empty() { "synthetic" } else { s }
        })
        .collect();
    all_repos.sort();
    all_repos.dedup();

    let max_rep = valid
        .iter()
        .map(|r| get_u64(r, "repetition"))
        .max()
        .unwrap_or(0);
    let num_reps = max_rep + 1;

    let mut lines = Vec::new();

    lines.push("# glean Benchmark Results".into());
    lines.push(String::new());
    lines.push(format!(
        "**Generated:** {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
    lines.push(String::new());
    let mut runs_line = format!("**Runs:** {} valid", valid.len());
    if error_count > 0 {
        runs_line.push_str(&format!(" ({error_count} errors)"));
    }
    lines.push(runs_line);
    lines.push(format!(
        " | **Models:** {} | **Repos:** {} | **Reps:** {num_reps}",
        all_models.join(", "),
        all_repos.join(", ")
    ));
    lines.push(String::new());
    lines.push("## Context Efficiency".into());
    lines.push(String::new());
    lines.push("The primary metric. Context tokens (input + cached) represent the actual context processed each turn. This compounds because each turn re-sends conversation history.".into());
    lines.push(String::new());
    lines.push("### Per-task comparison".into());
    lines.push(String::new());

    let valid_owned: Vec<Value> = valid.iter().copied().cloned().collect();
    let task_groups = group_by(&valid_owned, &["task"]);

    for &task_name in &all_tasks {
        let key = vec![task_name.to_string()];
        let task_results: Vec<&Value> = match task_groups.get(&key) {
            Some(v) => v.to_vec(),
            None => continue,
        };

        lines.push(format!("#### {task_name}"));
        lines.push(String::new());

        let task_repo = get_str(task_results[0], "repo");
        if !task_repo.is_empty() && task_repo != "synthetic" {
            lines.push(format!("*Repo: {task_repo}*"));
            lines.push(String::new());
        }

        let mode_groups = {
            let mut m: HashMap<&str, Vec<&Value>> = HashMap::new();
            for r in &task_results {
                m.entry(get_str(r, "mode")).or_default().push(r);
            }
            m
        };

        let has_baseline = mode_groups.contains_key("baseline");
        let has_glean = mode_groups.contains_key("glean");

        if has_baseline && has_glean {
            let baseline_runs = &mode_groups["baseline"];
            let glean_runs = &mode_groups["glean"];

            let metrics: &[(&str, &str)] = &[
                ("Context tokens", "context_tokens"),
                ("Output tokens", "output_tokens"),
                ("Turns", "num_turns"),
                ("Tool calls", "num_tool_calls"),
                ("Cost USD", "total_cost_usd"),
                ("Duration ms", "duration_ms"),
            ];

            lines.push("| Metric | baseline | glean | delta |".into());
            lines.push("|--------|----------|-------|-------|".into());

            for &(label, key) in metrics {
                let b_vals: Vec<f64> = baseline_runs.iter().map(|r| get_f64(r, key)).collect();
                let g_vals: Vec<f64> = glean_runs.iter().map(|r| get_f64(r, key)).collect();
                let b_stats = compute_stats(&b_vals);
                let g_stats = compute_stats(&g_vals);
                let delta = format_delta(b_stats.median, g_stats.median);

                let (b_fmt, g_fmt) = if key == "total_cost_usd" {
                    (
                        format!("${:.4}", b_stats.median),
                        format!("${:.4}", g_stats.median),
                    )
                } else {
                    (
                        format!("{:.0}", b_stats.median),
                        format!("{:.0}", g_stats.median),
                    )
                };

                lines.push(format!(
                    "| {label} (median) | {b_fmt} | {g_fmt} | {delta} |"
                ));
            }

            // Correctness
            let b_correct = baseline_runs
                .iter()
                .filter(|r| get_bool(r, "correct"))
                .count();
            let g_correct = glean_runs.iter().filter(|r| get_bool(r, "correct")).count();
            let b_pct = b_correct as f64 / baseline_runs.len() as f64 * 100.0;
            let g_pct = g_correct as f64 / glean_runs.len() as f64 * 100.0;
            lines.push(format!(
                "| Correctness | {b_pct:.0}% | {g_pct:.0}% | \u{2014} |"
            ));
            lines.push(String::new());

            // Cost breakdown
            let b_median_run = find_median_run(baseline_runs, "total_cost_usd");
            let g_median_run = find_median_run(glean_runs, "total_cost_usd");
            let b_costs = compute_cost_breakdown(b_median_run);
            let g_costs = compute_cost_breakdown(g_median_run);
            let b_total = get_f64(b_median_run, "total_cost_usd");
            let g_total = get_f64(g_median_run, "total_cost_usd");
            let total_delta = g_total - b_total;
            let b_turns = get_u64(b_median_run, "num_turns");
            let g_turns = get_u64(g_median_run, "num_turns");
            let turns_delta = g_turns as i64 - b_turns as i64;
            let b_correct_str = if get_bool(b_median_run, "correct") {
                "correct"
            } else {
                "incorrect"
            };
            let g_correct_str = if get_bool(g_median_run, "correct") {
                "correct"
            } else {
                "incorrect"
            };

            lines.push("**Cost breakdown (median run):**".into());
            lines.push(String::new());
            lines.push(format!(
                "  baseline: {b_turns} turns, ${b_total:.2}, {b_correct_str}"
            ));
            lines.push(format_cost_breakdown(&b_costs));
            lines.push(format!(
                "  glean:    {g_turns} turns, ${g_total:.2}, {g_correct_str}"
            ));
            lines.push(format_cost_breakdown(&g_costs));
            lines.push(format!(
                "  delta:    {:+} turns, {:+.2}",
                turns_delta, total_delta
            ));
            lines.push(format_cost_delta(&b_costs, &g_costs));
            lines.push(String::new());

            // Per-turn sparklines
            let b_median_ctx = find_median_run(baseline_runs, "context_tokens");
            let g_median_ctx = find_median_run(glean_runs, "context_tokens");
            let b_per_turn: Vec<u64> = b_median_ctx
                .get("per_turn_context_tokens")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_u64).collect())
                .unwrap_or_default();
            let g_per_turn: Vec<u64> = g_median_ctx
                .get("per_turn_context_tokens")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_u64).collect())
                .unwrap_or_default();

            if !b_per_turn.is_empty() && !g_per_turn.is_empty() {
                lines.push("**Per-turn context tokens (median run):**".into());
                lines.push(String::new());
                let b_spark = ascii_sparkline(&b_per_turn);
                let g_spark = ascii_sparkline(&g_per_turn);
                let b_min = b_per_turn.iter().min().unwrap();
                let b_max = b_per_turn.iter().max().unwrap();
                let g_min = g_per_turn.iter().min().unwrap();
                let g_max = g_per_turn.iter().max().unwrap();
                lines.push(format!("  baseline: {b_spark} ({b_min} \u{2192} {b_max})"));
                lines.push(format!("  glean:    {g_spark} ({g_min} \u{2192} {g_max})"));
                lines.push(String::new());
            }

            // Tool breakdown
            let b_tools = merge_tool_calls(baseline_runs);
            let g_tools = merge_tool_calls(glean_runs);
            if !b_tools.is_empty() || !g_tools.is_empty() {
                lines.push("**Tool breakdown (median counts):**".into());
                lines.push(String::new());
                if !b_tools.is_empty() {
                    let strs: Vec<String> =
                        b_tools.iter().map(|(k, v)| format!("{k}={v:.0}")).collect();
                    lines.push(format!("  baseline: {}", strs.join(", ")));
                }
                if !g_tools.is_empty() {
                    let strs: Vec<String> =
                        g_tools.iter().map(|(k, v)| format!("{k}={v:.0}")).collect();
                    lines.push(format!("  glean:    {}", strs.join(", ")));
                }
                lines.push(String::new());
            }
        } else {
            // Only one mode available
            for &mode_name in &all_modes {
                let mode_results = match mode_groups.get(mode_name) {
                    Some(v) => v,
                    None => continue,
                };

                lines.push(format!("**Mode: {mode_name}**"));
                lines.push(String::new());
                lines.push("| Metric | Median |".into());
                lines.push("|--------|--------|".into());

                let metrics: &[(&str, &str)] = &[
                    ("Context tokens", "context_tokens"),
                    ("Output tokens", "output_tokens"),
                    ("Turns", "num_turns"),
                    ("Tool calls", "num_tool_calls"),
                    ("Cost USD", "total_cost_usd"),
                    ("Duration ms", "duration_ms"),
                ];

                for &(label, key) in metrics {
                    let vals: Vec<f64> = mode_results.iter().map(|r| get_f64(r, key)).collect();
                    let stats = compute_stats(&vals);
                    let fmt = if key == "total_cost_usd" {
                        format!("${:.4}", stats.median)
                    } else {
                        format!("{:.0}", stats.median)
                    };
                    lines.push(format!("| {label} | {fmt} |"));
                }

                let correct = mode_results
                    .iter()
                    .filter(|r| get_bool(r, "correct"))
                    .count();
                let pct = correct as f64 / mode_results.len() as f64 * 100.0;
                lines.push(format!("| Correctness | {pct:.0}% |"));
                lines.push(String::new());
            }
        }
        lines.push(String::new());
    }

    // Summary section
    let baseline_all: Vec<&Value> = valid
        .iter()
        .filter(|r| get_str(r, "mode") == "baseline")
        .copied()
        .collect();
    let glean_all: Vec<&Value> = valid
        .iter()
        .filter(|r| get_str(r, "mode") == "glean")
        .copied()
        .collect();

    if !baseline_all.is_empty() && !glean_all.is_empty() {
        lines.push("## Summary".into());
        lines.push(String::new());
        lines.push("Averaged across all tasks (median of medians):".into());
        lines.push(String::new());
        lines.push("| Metric | baseline | glean | Improvement |".into());
        lines.push("|--------|----------|-------|-------------|".into());

        let metrics: &[(&str, &str)] = &[
            ("Context tokens", "context_tokens"),
            ("Turns", "num_turns"),
            ("Tool calls", "num_tool_calls"),
            ("Cost USD", "total_cost_usd"),
        ];

        for &(label, key) in metrics {
            let b_by_task = {
                let mut m: HashMap<&str, Vec<f64>> = HashMap::new();
                for r in &baseline_all {
                    m.entry(get_str(r, "task"))
                        .or_default()
                        .push(get_f64(r, key));
                }
                m
            };
            let g_by_task = {
                let mut m: HashMap<&str, Vec<f64>> = HashMap::new();
                for r in &glean_all {
                    m.entry(get_str(r, "task"))
                        .or_default()
                        .push(get_f64(r, key));
                }
                m
            };

            let b_medians: Vec<f64> = b_by_task
                .values()
                .map(|v| compute_stats(v).median)
                .collect();
            let g_medians: Vec<f64> = g_by_task
                .values()
                .map(|v| compute_stats(v).median)
                .collect();

            if !b_medians.is_empty() && !g_medians.is_empty() {
                let b_val = compute_stats(&b_medians).median;
                let g_val = compute_stats(&g_medians).median;
                let improvement = format_delta(b_val, g_val);

                let (b_fmt, g_fmt) = if key == "total_cost_usd" {
                    (format!("${b_val:.4}"), format!("${g_val:.4}"))
                } else {
                    (format!("{b_val:.0}"), format!("{g_val:.0}"))
                };

                lines.push(format!("| {label} | {b_fmt} | {g_fmt} | {improvement} |"));
            }
        }

        lines.push(String::new());
    }

    lines.join("\n")
}

pub fn analyze(results_path: &Path, output_path: Option<&Path>) {
    if !results_path.exists() {
        eprintln!("ERROR: File not found: {}", results_path.display());
        std::process::exit(1);
    }

    let results = load_results(results_path);
    let report = generate_report(&results);

    if let Some(out) = output_path {
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(out, &report).expect("Failed to write report");
        println!("Report written to: {}", out.display());
    } else {
        println!("{report}");
    }
}
