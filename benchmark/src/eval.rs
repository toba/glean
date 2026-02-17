use crate::{config, run, tasks};
use std::path::Path;

/// Default budget cap for eval runs (keep costs low).
const EVAL_MAX_BUDGET_USD: f64 = 0.10;

/// Run eval tasks with speed-tuned defaults.
pub fn eval(
    model: Option<&str>,
    task_filter: Option<&str>,
    mode_filter: Option<&str>,
    reps: Option<u32>,
    verbose: bool,
    output: Option<&Path>,
) {
    let eval_tasks = tasks::eval_tasks();
    let task_keys: Vec<&str> = eval_tasks.keys().copied().collect();
    let benchmark_dir = config::benchmark_dir();
    let mode_map = config::modes(&benchmark_dir);
    let mode_keys: Vec<&str> = mode_map.keys().copied().collect();

    let selected_tasks = if let Some(filter) = task_filter {
        run::parse_comma_list(filter, &task_keys, "tasks").unwrap_or_else(|e| {
            eprintln!("ERROR: {e}");
            std::process::exit(1);
        })
    } else {
        let mut keys = task_keys;
        keys.sort();
        keys
    };

    let selected_modes = if let Some(filter) = mode_filter {
        run::parse_comma_list(filter, &mode_keys, "modes").unwrap_or_else(|e| {
            eprintln!("ERROR: {e}");
            std::process::exit(1);
        })
    } else {
        vec!["baseline", "glean"]
    };

    let model_name = model.unwrap_or("sonnet");
    let all_models = config::models();
    if !all_models.contains_key(model_name) {
        eprintln!(
            "ERROR: Unknown model '{model_name}'. Valid: {}",
            all_models.keys().copied().collect::<Vec<_>>().join(", ")
        );
        std::process::exit(1);
    }

    let reps = reps.unwrap_or(1);

    let output_path = output.map(std::path::Path::to_path_buf).unwrap_or_else(|| {
        let results_dir = config::results_dir();
        std::fs::create_dir_all(&results_dir).expect("Failed to create results directory");
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        results_dir.join(format!("eval_{timestamp}_{model_name}.jsonl"))
    });

    println!("Budget cap: ${EVAL_MAX_BUDGET_USD:.2}/task");
    println!();

    run::run(
        &[model_name],
        &selected_tasks,
        &selected_modes,
        reps,
        None,
        verbose,
        &eval_tasks,
        Some(&output_path),
        Some(EVAL_MAX_BUDGET_USD),
    );
}
