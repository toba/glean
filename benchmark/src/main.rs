mod analyze;
mod compare;
mod config;
mod json_helpers;
mod parse;
mod run;
mod setup;
mod task;
mod tasks;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bench", about = "glean benchmark suite")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run benchmarks
    Run {
        /// Comma-separated model names or 'all'
        #[arg(long, default_value = "sonnet")]
        models: String,
        /// Comma-separated task names or 'all'
        #[arg(long, default_value = "all")]
        tasks: String,
        /// Comma-separated mode names or 'all'
        #[arg(long, default_value = "all")]
        modes: String,
        /// Number of repetitions
        #[arg(long, default_value_t = config::DEFAULT_REPS)]
        reps: u32,
        /// Filter tasks by repo (comma-separated or 'all')
        #[arg(long, default_value = "all")]
        repos: String,
        /// Print detailed output for debugging
        #[arg(long)]
        verbose: bool,
    },
    /// Generate markdown report from JSONL results
    Analyze {
        /// Path to JSONL results file
        results_file: PathBuf,
        /// Output path for markdown report (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Compare two JSONL result files
    Compare {
        /// Old results file
        old: PathBuf,
        /// New results file
        new: PathBuf,
    },
    /// Set up benchmark fixtures
    Setup {
        /// Clone real-world repos at pinned commits
        #[arg(long)]
        repos: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            models,
            tasks,
            modes,
            reps,
            repos,
            verbose,
        } => {
            let all_tasks = tasks::all_tasks();
            let model_keys: Vec<&str> = config::models().keys().copied().collect();
            let task_keys: Vec<&str> = all_tasks.keys().copied().collect();
            let benchmark_dir = config::benchmark_dir();
            let mode_map = config::modes(&benchmark_dir);
            let mode_keys: Vec<&str> = mode_map.keys().copied().collect();

            let selected_models = run::parse_comma_list(&models, &model_keys, "models")
                .unwrap_or_else(|e| {
                    eprintln!("ERROR: {e}");
                    std::process::exit(1);
                });
            let selected_tasks =
                run::parse_comma_list(&tasks, &task_keys, "tasks").unwrap_or_else(|e| {
                    eprintln!("ERROR: {e}");
                    std::process::exit(1);
                });
            let selected_modes =
                run::parse_comma_list(&modes, &mode_keys, "modes").unwrap_or_else(|e| {
                    eprintln!("ERROR: {e}");
                    std::process::exit(1);
                });

            let repo_filter = if repos.eq_ignore_ascii_case("all") {
                None
            } else {
                Some(repos.as_str())
            };

            run::run(
                &selected_models,
                &selected_tasks,
                &selected_modes,
                reps,
                repo_filter,
                verbose,
                &all_tasks,
            );
        }
        Commands::Analyze {
            results_file,
            output,
        } => {
            analyze::analyze(&results_file, output.as_deref());
        }
        Commands::Compare { old, new } => {
            compare::compare(&old, &new);
        }
        Commands::Setup { repos } => {
            if !repos {
                println!("Specify --repos to clone real-world repos at pinned commits");
                println!("  bench setup --repos");
                std::process::exit(1);
            }
            setup::setup_repos();
        }
    }
}
