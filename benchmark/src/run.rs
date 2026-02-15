use crate::config::{self, ModeConfig};
use crate::parse::{self, RunResult};
use crate::task::Task;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// Get installed glean version via `glean --version`.
fn glean_version() -> Option<String> {
    Command::new("glean")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                Some(s.strip_prefix("glean ").unwrap_or(&s).to_string())
            } else {
                None
            }
        })
}

/// Resolve working directory for a task's repo.
fn get_repo_path(repo_name: &str) -> PathBuf {
    let repos = config::repos();
    repos[repo_name].path(&config::repos_dir())
}

/// Reset a repo to its clean state (undo edits, remove untracked files).
fn reset_repo(repo_path: &Path) {
    let _ = Command::new("git")
        .args(["checkout", "--", "."])
        .current_dir(repo_path)
        .output();
    let _ = Command::new("git")
        .args(["clean", "-fd"])
        .current_dir(repo_path)
        .output();
}

/// Extract ordered tool call names + key args from all turns.
fn compact_tool_sequence(result: &RunResult) -> Vec<Value> {
    let mut seq = Vec::new();
    for turn in &result.turns {
        for tc in &turn.tool_calls {
            let mut entry = serde_json::Map::new();
            entry.insert("name".into(), json!(tc.name));
            let mut args = serde_json::Map::new();
            for (k, v) in &tc.input {
                match k.as_str() {
                    "command" => {
                        let s = v.as_str().unwrap_or("");
                        args.insert(k.clone(), json!(&s[..s.len().min(80)]));
                    }
                    "file_path" => {
                        let s = v.as_str().unwrap_or("");
                        let fname = s.rsplit('/').next().unwrap_or(s);
                        args.insert(k.clone(), json!(fname));
                    }
                    "pattern" | "query" | "path" | "scope" | "kind" | "section" | "expand" => {
                        let s = v.as_str().unwrap_or("");
                        args.insert(k.clone(), json!(&s[..s.len().min(60)]));
                    }
                    _ => {}
                }
            }
            if !args.is_empty() {
                entry.insert("args".into(), Value::Object(args));
            }
            seq.push(Value::Object(entry));
        }
    }
    seq
}

/// Run a single benchmark iteration.
#[expect(clippy::too_many_arguments)]
fn run_single(
    task: &dyn Task,
    task_name: &str,
    mode: &ModeConfig,
    mode_name: &str,
    model_id: &str,
    model_name: &str,
    repetition: u32,
    verbose: bool,
) -> Result<Value, String> {
    let repo_path = get_repo_path(task.repo());

    let mut cmd_args = vec![
        "claude".to_string(),
        "-p".into(),
        "--output-format".into(),
        "stream-json".into(),
        "--verbose".into(),
        "--model".into(),
        model_id.to_string(),
        "--max-budget-usd".into(),
        config::DEFAULT_MAX_BUDGET_USD.to_string(),
        "--no-session-persistence".into(),
        "--dangerously-skip-permissions".into(),
        "--strict-mcp-config".into(),
        "--system-prompt".into(),
        config::SYSTEM_PROMPT.to_string(),
    ];

    if !mode.tools.is_empty() {
        cmd_args.push("--tools".into());
        cmd_args.push(mode.tools.join(","));
    }

    if let Some(ref mcp_path) = mode.mcp_config_path {
        cmd_args.push("--mcp-config".into());
        cmd_args.push(mcp_path.display().to_string());
    }

    cmd_args.push("--".into());
    cmd_args.push(task.prompt().to_string());

    if verbose {
        eprintln!("    Running: {}", cmd_args.join(" "));
    }

    // Unset CLAUDECODE to allow nested claude -p
    let env: HashMap<String, String> = std::env::vars()
        .filter(|(k, _)| k != "CLAUDECODE")
        .collect();

    let start = Instant::now();
    let output = Command::new(&cmd_args[0])
        .args(&cmd_args[1..])
        .current_dir(&repo_path)
        .envs(&env)
        .output()
        .map_err(|e| format!("Failed to spawn claude: {e}"))?;
    let elapsed_ms = start.elapsed().as_millis() as u64;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "claude -p failed with code {:?}\nstderr: {stderr}\nstdout: {}",
            output.status.code(),
            &stdout[..stdout.len().min(500)]
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut run_result = parse::parse_stream_json(&stdout);
    run_result.task_name = task_name.to_string();
    run_result.mode_name = mode_name.to_string();
    run_result.model_name = model_name.to_string();
    run_result.repetition = repetition;

    if run_result.duration_ms == 0 {
        run_result.duration_ms = elapsed_ms;
    }

    let (correct, reason) = task.check_correctness(&run_result.result_text, &repo_path);
    run_result.correct = correct;
    run_result.correctness_reason = reason.clone();

    let tool_breakdown = parse::tool_call_counts(&run_result);
    let per_turn_context: Vec<u64> = run_result
        .turns
        .iter()
        .map(|t| t.context_tokens())
        .collect();
    let total_context: u64 = per_turn_context.iter().sum();
    let num_tool_calls: u64 = tool_breakdown.values().sum();

    let result_text_truncated = if run_result.result_text.len() > 5000 {
        &run_result.result_text[..5000]
    } else {
        &run_result.result_text
    };

    Ok(json!({
        "task": task_name,
        "repo": task.repo(),
        "mode": mode_name,
        "model": model_name,
        "repetition": repetition,
        "glean_version": if mode_name.contains("glean") { glean_version() } else { None },
        "num_turns": run_result.num_turns,
        "num_tool_calls": num_tool_calls,
        "tool_calls": tool_breakdown,
        "duration_ms": run_result.duration_ms,
        "context_tokens": total_context,
        "output_tokens": run_result.total_output_tokens,
        "input_tokens": run_result.total_input_tokens,
        "cache_creation_tokens": run_result.total_cache_creation_tokens,
        "cache_read_tokens": run_result.total_cache_read_tokens,
        "per_turn_context_tokens": per_turn_context,
        "correct": correct,
        "correctness_reason": reason,
        "result_text": result_text_truncated,
        "tool_sequence": compact_tool_sequence(&run_result),
    }))
}

/// Parse a comma-separated list, validating against valid options.
pub fn parse_comma_list<'a>(
    value: &str,
    valid: &[&'a str],
    name: &str,
) -> Result<Vec<&'a str>, String> {
    if value.eq_ignore_ascii_case("all") {
        return Ok(valid.to_vec());
    }
    let mut result = Vec::new();
    for item in value.split(',') {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        if let Some(found) = valid.iter().find(|&&v| v == item) {
            result.push(*found);
        } else {
            return Err(format!(
                "Invalid {name}: {item}. Valid options: {}",
                valid.join(", ")
            ));
        }
    }
    Ok(result)
}

/// Main benchmark runner.
pub fn run(
    model_names: &[&str],
    task_names: &[&str],
    mode_names: &[&str],
    reps: u32,
    repo_filter: Option<&str>,
    verbose: bool,
    tasks: &HashMap<&str, Box<dyn Task>>,
) {
    let all_models = config::models();
    let benchmark_dir = config::benchmark_dir();
    let all_modes = config::modes(&benchmark_dir);
    let all_repos = config::repos();
    let repos_dir = config::repos_dir();

    // Filter tasks by repo if requested
    let filtered_tasks: Vec<&str> = if let Some(repo_filter) = repo_filter {
        if repo_filter.eq_ignore_ascii_case("all") {
            task_names.to_vec()
        } else {
            let requested: Vec<&str> = repo_filter.split(',').map(str::trim).collect();
            task_names
                .iter()
                .filter(|&&t| {
                    let repo = tasks[t].repo();
                    requested.contains(&repo)
                })
                .copied()
                .collect()
        }
    } else {
        task_names.to_vec()
    };

    if filtered_tasks.is_empty() {
        eprintln!("ERROR: No tasks match the specified filters.");
        std::process::exit(1);
    }

    // Validate real-world repos exist
    let selected_repos: Vec<&str> = filtered_tasks
        .iter()
        .map(|&t| tasks[t].repo())
        .collect();
    for repo_name in &selected_repos {
        if let Some(rc) = all_repos.get(repo_name) {
            let path = rc.path(&repos_dir);
            if !path.exists() {
                eprintln!("ERROR: Repo '{repo_name}' not cloned.");
                eprintln!("Expected at: {}", path.display());
                eprintln!("Run: bench setup --repos");
                std::process::exit(1);
            }
        }
    }

    // Validate MCP config for glean modes
    let needs_mcp = mode_names.iter().any(|&m| m.contains("glean"));
    if needs_mcp {
        let mcp_path = config::fixtures_dir().join("glean_mcp.json");
        if !mcp_path.exists() {
            eprintln!("ERROR: MCP config not found at {}", mcp_path.display());
            eprintln!("Run: bench setup --repos  (this generates glean_mcp.json from your PATH)");
            std::process::exit(1);
        }
        // Verify the referenced binary exists
        if let Ok(contents) = fs::read_to_string(&mcp_path)
            && let Ok(json) = serde_json::from_str::<Value>(&contents)
            && let Some(cmd) = json
                .pointer("/mcpServers/glean/command")
                .and_then(|v| v.as_str())
        {
            let found = if cmd.contains('/') {
                // Absolute path — check file exists
                std::path::Path::new(cmd).exists()
            } else {
                // Bare command — check PATH
                Command::new("which")
                    .arg(cmd)
                    .output()
                    .is_ok_and(|o| o.status.success())
            };
            if !found {
                eprintln!("ERROR: glean binary not found: {cmd}");
                eprintln!("The path in {} is stale.", mcp_path.display());
                eprintln!("Run: bench setup --repos  (to regenerate)");
                std::process::exit(1);
            }
        }
    }

    // Create results directory
    let results_dir = config::results_dir();
    fs::create_dir_all(&results_dir).expect("Failed to create results directory");

    // Create timestamped output file
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let model_suffix = if model_names.len() == 1 {
        format!("_{}", model_names[0])
    } else {
        String::new()
    };
    let output_file = results_dir.join(format!("benchmark_{timestamp}{model_suffix}.jsonl"));

    // Print configuration
    println!("{}", "=".repeat(70));
    println!("glean Benchmark Runner");
    println!("{}", "=".repeat(70));
    println!("Models:      {}", model_names.join(", "));
    println!("Tasks:       {}", filtered_tasks.join(", "));
    println!("Modes:       {}", mode_names.join(", "));
    let repos_used: Vec<&str> = {
        let mut r: Vec<&str> = filtered_tasks.iter().map(|&t| tasks[t].repo()).collect();
        r.sort();
        r.dedup();
        r
    };
    println!("Repos:       {}", repos_used.join(", "));
    println!("Repetitions: {reps}");
    println!("Output:      {}", output_file.display());
    println!("{}", "=".repeat(70));
    println!();

    let total_runs = filtered_tasks.len() * mode_names.len() * model_names.len() * reps as usize;
    let mut current_run = 0;

    let file = File::create(&output_file).expect("Failed to create output file");
    let mut writer = BufWriter::new(file);

    let mut prev_task: Option<&str> = None;
    let mut prev_mode: Option<&str> = None;

    for &task_name in &filtered_tasks {
        let task = &*tasks[task_name];
        for &mode_name in mode_names {
            let mode = &all_modes[mode_name];
            for &model_name in model_names {
                let model_id = all_models[model_name];
                for rep in 0..reps {
                    current_run += 1;
                    let run_id = format!("{task_name}/{mode_name}/{model_name}/rep{rep}");

                    // Reset repo if needed (for edit tasks, reset before each run;
                    // for others, reset when mode changes)
                    let repo_path = get_repo_path(task.repo());
                    let mut needs_reset = false;
                    if task.task_type() == "edit" {
                        if rep > 0
                            || prev_mode != Some(mode_name)
                            || prev_task != Some(task_name)
                        {
                            needs_reset = true;
                        }
                    } else if prev_mode != Some(mode_name) {
                        needs_reset = true;
                    }
                    if needs_reset {
                        if verbose {
                            eprintln!("  Resetting repo {}...", task.repo());
                        }
                        reset_repo(&repo_path);
                    }
                    prev_task = Some(task_name);
                    prev_mode = Some(mode_name);

                    println!("[{current_run}/{total_runs}] {run_id}");

                    match run_single(
                        task, task_name, mode, mode_name, model_id, model_name, rep, verbose,
                    ) {
                        Ok(result) => {
                            writeln!(writer, "{}", serde_json::to_string(&result).unwrap())
                                .unwrap();
                            writer.flush().unwrap();

                            let correct = result["correct"].as_bool().unwrap_or(false);
                            let status = if correct { "\u{2713}" } else { "\u{2717}" };
                            let num_turns = result["num_turns"].as_u64().unwrap_or(0);
                            let ctx = result["context_tokens"].as_u64().unwrap_or(0);
                            let out = result["output_tokens"].as_u64().unwrap_or(0);
                            let dur = result["duration_ms"].as_u64().unwrap_or(0);

                            println!(
                                "  {status} {num_turns}t {ctx}ctx {out}out {dur}ms"
                            );

                            if !correct {
                                let reason =
                                    result["correctness_reason"].as_str().unwrap_or("unknown");
                                println!("  \u{2192} {reason}");
                            }
                        }
                        Err(e) => {
                            if e.contains("timeout") || e.contains("Timeout") {
                                println!("  \u{2717} TIMEOUT (>300s)");
                            } else {
                                println!("  \u{2717} ERROR: {e}");
                            }
                            let error_result = json!({
                                "task": task_name,
                                "mode": mode_name,
                                "model": model_name,
                                "repetition": rep,
                                "error": e,
                                "correct": false,
                                "correctness_reason": format!("Exception: {e}"),
                            });
                            writeln!(writer, "{}", serde_json::to_string(&error_result).unwrap())
                                .unwrap();
                            writer.flush().unwrap();
                        }
                    }
                }
            }
        }
    }

    println!();
    println!("{}", "=".repeat(70));
    println!("Benchmark complete!");
    println!("Results saved to: {}", output_file.display());
    println!("{}", "=".repeat(70));
    println!();
    println!("To generate a report, run:");
    println!("  bench analyze {}", output_file.display());
    println!();
}
