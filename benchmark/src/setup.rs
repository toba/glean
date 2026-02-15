use crate::config;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Clone and pin a single repository.
fn setup_repo(name: &str, url: &str, commit_sha: &str, repo_path: &Path) {
    if repo_path.exists() {
        // Verify correct commit
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo_path)
            .output();
        if let Ok(o) = output {
            let current = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if current == commit_sha {
                println!("  {name}: already at {}", &commit_sha[..8]);
                return;
            }
            println!(
                "  {name}: at {}, need {}, re-cloning...",
                &current[..current.len().min(8)],
                &commit_sha[..8]
            );
        }
        fs::remove_dir_all(repo_path).ok();
    }

    println!("  {name}: cloning from {url}...");
    let status = Command::new("git")
        .args([
            "clone",
            "--no-checkout",
            url,
            &repo_path.display().to_string(),
        ])
        .output()
        .expect("Failed to run git clone");
    if !status.status.success() {
        eprintln!(
            "  ERROR: git clone failed: {}",
            String::from_utf8_lossy(&status.stderr)
        );
        return;
    }

    let status = Command::new("git")
        .args(["checkout", commit_sha])
        .current_dir(repo_path)
        .output()
        .expect("Failed to run git checkout");
    if !status.status.success() {
        eprintln!(
            "  ERROR: git checkout failed: {}",
            String::from_utf8_lossy(&status.stderr)
        );
        return;
    }
    println!("  {name}: checked out {}", &commit_sha[..8]);
}

/// Clone all real-world benchmark repos.
pub fn setup_repos() {
    let repos_dir = config::repos_dir();
    fs::create_dir_all(&repos_dir).expect("Failed to create repos directory");

    println!("Setting up benchmark repositories...");
    for (_, rc) in config::repos() {
        let path = rc.path(&repos_dir);
        setup_repo(rc.name, rc.url, rc.commit_sha, &path);
    }
    println!("Done.");
}

/// Generate the synthetic Python project for benchmarking.
pub fn setup_synthetic() {
    let repo_path = config::synthetic_repo();

    if repo_path.exists() {
        println!("Removing existing repo at {}", repo_path.display());
        fs::remove_dir_all(&repo_path).ok();
    }

    println!("Creating repo at {}", repo_path.display());
    fs::create_dir_all(&repo_path).expect("Failed to create repo directory");

    let dirs = [
        "src/auth",
        "src/api",
        "src/database",
        "src/models",
        "src/utils",
        "tests",
    ];
    for d in &dirs {
        fs::create_dir_all(repo_path.join(d)).unwrap();
    }

    let files = synthetic_files();
    let mut file_stats = Vec::new();

    for (path, content) in &files {
        let full = repo_path.join(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full, content).unwrap();
        let lines = content.lines().count();
        file_stats.push((path.as_str(), lines));
        println!("  Created {path} ({lines} lines)");
    }

    // Initialize git repo
    println!("\nInitializing git repository...");
    Command::new("git")
        .arg("init")
        .current_dir(&repo_path)
        .output()
        .expect("git init failed");
    Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()
        .expect("git add failed");
    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()
        .expect("git commit failed");

    println!("\n{}", "=".repeat(60));
    println!("Repository setup complete!");
    println!("{}", "=".repeat(60));
    println!("\nLocation: {}", repo_path.display());
    println!("Total files: {}", file_stats.len());
    println!(
        "Total lines: {}",
        file_stats.iter().map(|(_, n)| n).sum::<usize>()
    );
    println!("\nFile breakdown:");
    file_stats.sort_by(|a, b| b.1.cmp(&a.1));
    for (path, lines) in &file_stats {
        println!("  {path:40} {lines:4} lines");
    }
}

fn synthetic_files() -> Vec<(String, String)> {
    vec![
        (
            "src/auth/tokens.py".into(),
            include_str!("synthetic_content/tokens.py").into(),
        ),
        (
            "src/api/routes.py".into(),
            include_str!("synthetic_content/routes.py").into(),
        ),
        (
            "src/database/connection.py".into(),
            include_str!("synthetic_content/connection.py").into(),
        ),
        (
            "README.md".into(),
            include_str!("synthetic_content/readme.md").into(),
        ),
        (
            "src/auth/middleware.py".into(),
            include_str!("synthetic_content/middleware.py").into(),
        ),
        (
            "src/database/queries.py".into(),
            include_str!("synthetic_content/queries.py").into(),
        ),
        (
            "src/database/migrations.py".into(),
            include_str!("synthetic_content/migrations.py").into(),
        ),
        (
            "src/auth/__init__.py".into(),
            "\"\"\"Authentication module.\"\"\"\n".into(),
        ),
        (
            "src/database/__init__.py".into(),
            "\"\"\"Database module.\"\"\"\n".into(),
        ),
        (
            "src/api/__init__.py".into(),
            "\"\"\"API module.\"\"\"\n".into(),
        ),
        (
            "src/api/validators.py".into(),
            include_str!("synthetic_content/validators.py").into(),
        ),
        (
            "src/models/__init__.py".into(),
            "\"\"\"Data models module.\"\"\"\n".into(),
        ),
        (
            "src/models/user.py".into(),
            include_str!("synthetic_content/user.py").into(),
        ),
        (
            "src/models/order.py".into(),
            include_str!("synthetic_content/order.py").into(),
        ),
        (
            "src/utils/__init__.py".into(),
            "\"\"\"Utilities module.\"\"\"\n".into(),
        ),
        (
            "src/utils/logging.py".into(),
            include_str!("synthetic_content/logging.py").into(),
        ),
        (
            "src/utils/config.py".into(),
            include_str!("synthetic_content/synth_config.py").into(),
        ),
        (
            "tests/test_auth.py".into(),
            include_str!("synthetic_content/test_auth.py").into(),
        ),
        (
            "tests/test_database.py".into(),
            include_str!("synthetic_content/test_database.py").into(),
        ),
        (
            "pyproject.toml".into(),
            include_str!("synthetic_content/pyproject.toml").into(),
        ),
    ]
}
