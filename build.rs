use std::process::Command;

fn main() {
    // Embed git commit hash at build time
    let hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".into());

    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .is_some_and(|o| !o.stdout.is_empty());

    let commit = if dirty { format!("{hash}-dirty") } else { hash };

    println!("cargo:rustc-env=GLEAN_BUILD_COMMIT={commit}");

    // Re-run if HEAD changes (new commit)
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
}
