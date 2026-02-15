use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Model name → API model ID.
pub fn models() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("haiku", "claude-haiku-4-5-20251001"),
        ("sonnet", "claude-sonnet-4-5-20250929"),
        ("opus", "claude-opus-4-6"),
    ])
}

#[allow(dead_code)]
pub struct ModeConfig {
    pub name: &'static str,
    pub tools: Vec<&'static str>,
    pub mcp_config_path: Option<PathBuf>,
    pub description: &'static str,
}

pub fn modes(benchmark_dir: &Path) -> HashMap<&'static str, ModeConfig> {
    let glean_mcp = benchmark_dir.join("fixtures/glean_mcp.json");
    HashMap::from([
        (
            "baseline",
            ModeConfig {
                name: "baseline",
                tools: vec!["Read", "Edit", "Grep", "Glob", "Bash"],
                mcp_config_path: None,
                description: "Claude Code built-in tools",
            },
        ),
        (
            "glean",
            ModeConfig {
                name: "glean",
                tools: vec!["Read", "Edit", "Grep", "Glob", "Bash"],
                mcp_config_path: Some(glean_mcp.clone()),
                description: "Built-in tools + glean MCP (hybrid)",
            },
        ),
        (
            "glean_forced",
            ModeConfig {
                name: "glean_forced",
                tools: vec!["Read", "Edit"],
                mcp_config_path: Some(glean_mcp),
                description: "glean MCP only (no Bash/Grep/Glob)",
            },
        ),
    ])
}

#[allow(dead_code)]
pub struct RepoConfig {
    pub name: &'static str,
    pub url: &'static str,
    pub commit_sha: &'static str,
    pub language: &'static str,
    pub description: &'static str,
}

impl RepoConfig {
    pub fn path(&self, repos_dir: &Path) -> PathBuf {
        repos_dir.join(self.name)
    }
}

pub fn repos() -> HashMap<&'static str, RepoConfig> {
    HashMap::from([
        (
            "ripgrep",
            RepoConfig {
                name: "ripgrep",
                url: "https://github.com/BurntSushi/ripgrep.git",
                commit_sha: "0a88cccd5188074de96f54a4b6b44a63971ac157",
                language: "rust",
                description: "ripgrep line-oriented search tool",
            },
        ),
        (
            "fastapi",
            RepoConfig {
                name: "fastapi",
                url: "https://github.com/tiangolo/fastapi.git",
                commit_sha: "6fa573ce0bc16fe445f93db413d20146dd9ff35d",
                language: "python",
                description: "FastAPI web framework",
            },
        ),
        (
            "gin",
            RepoConfig {
                name: "gin",
                url: "https://github.com/gin-gonic/gin.git",
                commit_sha: "d7776de7d444935ea4385999711bd6331a98fecb",
                language: "go",
                description: "Gin HTTP web framework",
            },
        ),
        (
            "express",
            RepoConfig {
                name: "express",
                url: "https://github.com/expressjs/express.git",
                commit_sha: "1140301f6a0ed5a05bc1ef38d48294f75a49580c",
                language: "javascript",
                description: "Express.js web framework",
            },
        ),
    ])
}

pub const SYSTEM_PROMPT: &str = "You are a code assistant. Answer the user's question about the codebase in the current directory.\nUse the tools available to you to explore and understand the code.\nBe precise and show relevant code when asked.";

pub const DEFAULT_REPS: u32 = 5;
pub const DEFAULT_MAX_BUDGET_USD: f64 = 1.0;

/// Resolve benchmark directory from the executable location or current dir.
pub fn benchmark_dir() -> PathBuf {
    // Try to find benchmark dir relative to current dir
    let cwd = std::env::current_dir().expect("cannot get cwd");
    if cwd.join("Cargo.toml").exists() && cwd.join("src/main.rs").exists() {
        return cwd;
    }
    if cwd.join("benchmark/Cargo.toml").exists() {
        return cwd.join("benchmark");
    }
    cwd
}

pub fn fixtures_dir() -> PathBuf {
    benchmark_dir().join("fixtures")
}

pub fn repos_dir() -> PathBuf {
    fixtures_dir().join("repos")
}

pub fn synthetic_repo() -> PathBuf {
    fixtures_dir().join("repo")
}

pub fn results_dir() -> PathBuf {
    benchmark_dir().join("results")
}
