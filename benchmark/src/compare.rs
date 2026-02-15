use crate::analyze::load_results;
use crate::json_helpers::{get_bool, get_str, get_u64};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

fn avg(runs: &[&Value], key: &str) -> f64 {
    if runs.is_empty() {
        return 0.0;
    }
    let vals: Vec<f64> = runs
        .iter()
        .filter_map(|r| r.get(key).and_then(Value::as_f64))
        .collect();
    if vals.is_empty() {
        return 0.0;
    }
    vals.iter().sum::<f64>() / vals.len() as f64
}

fn group_by_task_mode<'a>(runs: &'a [&Value]) -> HashMap<(String, String), Vec<&'a Value>> {
    let mut groups: HashMap<(String, String), Vec<&Value>> = HashMap::new();
    for r in runs {
        let key = (
            get_str(r, "task").to_string(),
            get_str(r, "mode").to_string(),
        );
        groups.entry(key).or_default().push(r);
    }
    groups
}

pub fn compare(old_path: &Path, new_path: &Path) {
    if !old_path.exists() {
        eprintln!("ERROR: File not found: {}", old_path.display());
        std::process::exit(1);
    }
    if !new_path.exists() {
        eprintln!("ERROR: File not found: {}", new_path.display());
        std::process::exit(1);
    }

    let old_results = load_results(old_path);
    let new_results = load_results(new_path);

    let old_valid: Vec<&Value> = old_results
        .iter()
        .filter(|r| r.get("error").is_none())
        .collect();
    let new_valid: Vec<&Value> = new_results
        .iter()
        .filter(|r| r.get("error").is_none())
        .collect();

    println!("{}", "=".repeat(80));
    println!("OLD vs NEW COMPARISON");
    println!("{}", "=".repeat(80));
    println!();
    println!("Old file: {}", old_path.display());
    println!("New file: {}", new_path.display());

    let old_groups = group_by_task_mode(&old_valid);
    let new_groups = group_by_task_mode(&new_valid);

    // Compare glean runs
    println!();
    println!("{}", "=".repeat(80));
    println!("GLEAN MODE COMPARISON");
    println!("{}", "=".repeat(80));

    let mut all_tasks: Vec<String> = old_groups
        .keys()
        .filter(|(_, m)| m == "glean")
        .map(|(t, _)| t.clone())
        .collect();
    all_tasks.sort();
    all_tasks.dedup();

    for task in &all_tasks {
        let old_glean = old_groups
            .get(&(task.clone(), "glean".into()))
            .cloned()
            .unwrap_or_default();
        let new_glean = new_groups
            .get(&(task.clone(), "glean".into()))
            .cloned()
            .unwrap_or_default();

        if old_glean.is_empty() || new_glean.is_empty() {
            continue;
        }

        println!();
        println!("{}", "=".repeat(80));
        println!("Task: {task}");
        println!("{}", "=".repeat(80));

        for (old, new) in old_glean.iter().zip(new_glean.iter()) {
            println!();
            println!("OLD: {}", get_str(old, "glean_version"));
            println!(
                "  Turns: {}, Tool calls: {}",
                get_u64(old, "num_turns"),
                get_u64(old, "num_tool_calls")
            );
            println!("  Tools: {}", old.get("tool_calls").unwrap_or(&Value::Null));
            println!("  Correct: {}", get_bool(old, "correct"));

            println!();
            println!("NEW: {}", get_str(new, "glean_version"));
            println!(
                "  Turns: {}, Tool calls: {}",
                get_u64(new, "num_turns"),
                get_u64(new, "num_tool_calls")
            );
            println!("  Tools: {}", new.get("tool_calls").unwrap_or(&Value::Null));
            println!("  Correct: {}", get_bool(new, "correct"));

            let turn_delta = get_u64(new, "num_turns") as i64 - get_u64(old, "num_turns") as i64;
            let tool_delta =
                get_u64(new, "num_tool_calls") as i64 - get_u64(old, "num_tool_calls") as i64;

            println!();
            println!("DELTA:");
            let turn_desc = if turn_delta > 0 {
                "more"
            } else if turn_delta < 0 {
                "fewer"
            } else {
                "same"
            };
            let tool_desc = if tool_delta > 0 {
                "more"
            } else if tool_delta < 0 {
                "fewer"
            } else {
                "same"
            };
            println!("  Turns: {turn_delta:+} ({turn_desc})");
            println!("  Tool calls: {tool_delta:+} ({tool_desc})");
            println!(
                "  Correctness: {}",
                if get_bool(old, "correct") == get_bool(new, "correct") {
                    "same"
                } else {
                    "CHANGED"
                }
            );
        }
    }

    // Summary statistics
    println!();
    println!("{}", "=".repeat(80));
    println!("SUMMARY STATISTICS");
    println!("{}", "=".repeat(80));

    let old_glean_sonnet: Vec<&Value> = old_valid
        .iter()
        .filter(|r| get_str(r, "mode") == "glean" && get_str(r, "model") == "sonnet")
        .copied()
        .collect();
    let new_glean_sonnet: Vec<&Value> = new_valid
        .iter()
        .filter(|r| get_str(r, "mode") == "glean" && get_str(r, "model") == "sonnet")
        .copied()
        .collect();

    println!();
    println!(
        "{:<30} {:>20} {:>20} {:>15}",
        "Metric", "Old", "New", "Delta"
    );
    println!("{}", "-".repeat(90));

    let metrics = [
        ("num_turns", "Avg turns"),
        ("num_tool_calls", "Avg tool calls"),
    ];
    for (key, label) in &metrics {
        let old_avg = avg(&old_glean_sonnet, key);
        let new_avg = avg(&new_glean_sonnet, key);
        let delta = new_avg - old_avg;
        println!("{label:<30} {old_avg:>20.2} {new_avg:>20.2} {delta:>15.2}");
    }

    // Correctness
    let old_correct = old_glean_sonnet
        .iter()
        .filter(|r| get_bool(r, "correct"))
        .count();
    let new_correct = new_glean_sonnet
        .iter()
        .filter(|r| get_bool(r, "correct"))
        .count();
    println!();
    println!(
        "{:<30} {:>17}/{} {:>17}/{} {:>15}",
        "Correctness",
        old_correct,
        old_glean_sonnet.len(),
        new_correct,
        new_glean_sonnet.len(),
        new_correct as i64 - old_correct as i64
    );

    // Tool mix
    println!();
    println!("{}", "=".repeat(80));
    println!("TOOL MIX ANALYSIS");
    println!("{}", "=".repeat(80));

    fn count_tools(runs: &[&Value]) -> HashMap<String, u64> {
        let mut counts: HashMap<String, u64> = HashMap::new();
        for r in runs {
            if let Some(tc) = r.get("tool_calls").and_then(Value::as_object) {
                for (tool, count) in tc {
                    *counts.entry(tool.clone()).or_insert(0) += count.as_u64().unwrap_or(0);
                }
            }
        }
        counts
    }

    let old_tools = count_tools(&old_glean_sonnet);
    let new_tools = count_tools(&new_glean_sonnet);

    let mut all_tool_names: Vec<String> =
        old_tools.keys().chain(new_tools.keys()).cloned().collect();
    all_tool_names.sort();
    all_tool_names.dedup();

    println!();
    println!("{:<40} {:>15} {:>15} {:>15}", "Tool", "Old", "New", "Delta");
    println!("{}", "-".repeat(90));

    for tool in &all_tool_names {
        let old_count = old_tools.get(tool).copied().unwrap_or(0);
        let new_count = new_tools.get(tool).copied().unwrap_or(0);
        let delta = new_count as i64 - old_count as i64;
        println!("{tool:<40} {old_count:>15} {new_count:>15} {delta:>15}");
    }
}
