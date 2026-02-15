use crate::config;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Resolve the glean binary: PATH first (as bare "glean"), then project build artifacts.
fn find_glean_binary() -> Result<String, String> {
    // Try PATH first — use bare command name so the config is portable
    if let Ok(output) = Command::new("which").arg("glean").output()
        && output.status.success()
    {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Ok("glean".into());
        }
    }

    // Fall back to project build: release then debug (absolute path, not portable)
    let project_root = config::benchmark_dir()
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();
    for profile in ["release", "debug"] {
        let candidate = project_root.join(format!("target/{profile}/glean"));
        if candidate.exists() {
            eprintln!("  NOTE: using local build artifact (not in PATH)");
            return Ok(candidate.canonicalize().unwrap().display().to_string());
        }
    }

    Err("glean not found in PATH or target/. Build it first: cargo build --release".into())
}

/// Generate the MCP config JSON pointing to the actual glean binary.
pub fn generate_mcp_config() -> Result<(), String> {
    let glean_path = find_glean_binary()?;

    let mcp_json = serde_json::json!({
        "mcpServers": {
            "glean": {
                "command": glean_path,
                "args": ["--mcp", "--edit"]
            }
        }
    });

    let dest = config::fixtures_dir().join("glean_mcp.json");
    fs::create_dir_all(dest.parent().unwrap()).ok();
    fs::write(&dest, serde_json::to_string_pretty(&mcp_json).unwrap())
        .map_err(|e| format!("Failed to write {}: {e}", dest.display()))?;

    println!("  MCP config: {} -> {}", dest.display(), glean_path);
    Ok(())
}

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
    // Generate MCP config pointing to the real glean binary
    if let Err(e) = generate_mcp_config() {
        eprintln!("  WARNING: {e}");
        eprintln!("  glean modes will not work until this is resolved.");
    }

    println!("Done.");
}
