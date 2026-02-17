use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::types::Match;

const VENDOR_DIRS: &[&str] = &[
    "node_modules",
    "vendor",
    "dist",
    "build",
    ".git",
    "target",
    "__pycache__",
    ".venv",
    "venv",
    "pkg",
    "out",
];

/// Sort matches by score (highest first). Deterministic: same inputs, same order.
/// When `context` is provided, matches near the context file are boosted.
pub fn sort(matches: &mut [Match], query: &str, scope: &Path, context: Option<&Path>) {
    // Pre-compute context's package root once (same for entire batch)
    let ctx_parent = context.and_then(|c| c.parent());
    let ctx_pkg_root = context
        .and_then(package_root)
        .map(std::path::Path::to_path_buf);

    // Cache package roots for match paths — avoids repeated stat walks
    let mut pkg_cache: HashMap<PathBuf, Option<PathBuf>> = HashMap::new();

    matches.sort_by(|a, b| {
        let sa = score(
            a,
            query,
            scope,
            ctx_parent,
            ctx_pkg_root.as_ref(),
            &mut pkg_cache,
        );
        let sb = score(
            b,
            query,
            scope,
            ctx_parent,
            ctx_pkg_root.as_ref(),
            &mut pkg_cache,
        );
        sb.cmp(&sa)
            .then_with(|| a.path.cmp(&b.path))
            .then_with(|| a.line.cmp(&b.line))
    });
}

/// Ranking function. Each match gets a score — no floating point, no randomness.
fn score(
    m: &Match,
    _query: &str,
    scope: &Path,
    ctx_parent: Option<&Path>,
    ctx_pkg_root: Option<&PathBuf>,
    pkg_cache: &mut HashMap<PathBuf, Option<PathBuf>>,
) -> i32 {
    let mut s = 0i32;

    if m.is_definition {
        s += 1000;
    }
    if m.exact {
        s += 500;
    }

    s += scope_proximity(&m.path, scope) as i32;
    s += recency(m.mtime) as i32;

    if m.file_lines > 0 && m.file_lines < 200 {
        s += 50;
    }

    // Context-aware boosts
    if ctx_parent.is_some() || ctx_pkg_root.is_some() {
        s += context_proximity(&m.path, ctx_parent, ctx_pkg_root, pkg_cache);
    }

    // Vendor penalty (always active)
    if is_vendor_path(&m.path) {
        s -= 200;
    }

    // Test file penalty — deprioritize test usages (not definitions) so agents
    // see real implementations first. Weaker than vendor (-100 vs -200) because
    // test files are still legitimate navigation targets.
    if !m.is_definition && is_test_file(&m.path) {
        s -= 100;
    }

    s
}

/// 0-200, closer to scope root = higher.
fn scope_proximity(path: &Path, scope: &Path) -> u32 {
    let rel = path.strip_prefix(scope).unwrap_or(path);
    let depth = rel.components().count();
    200u32.saturating_sub(depth as u32 * 20)
}

/// Context-aware proximity boost with cached package roots.
fn context_proximity(
    match_path: &Path,
    ctx_parent: Option<&Path>,
    ctx_pkg_root: Option<&PathBuf>,
    pkg_cache: &mut HashMap<PathBuf, Option<PathBuf>>,
) -> i32 {
    // Same directory as context file
    if let Some(cp) = ctx_parent
        && match_path.parent() == Some(cp)
    {
        return 100;
    }

    // Same package root (cached)
    if let Some(cp_root) = ctx_pkg_root {
        let match_dir = match match_path.parent() {
            Some(d) => d.to_path_buf(),
            None => return 0,
        };
        let match_root = pkg_cache
            .entry(match_dir)
            .or_insert_with_key(|dir| package_root(dir).map(std::path::Path::to_path_buf));
        if let Some(mr) = match_root
            && mr == cp_root
        {
            return 75;
        }
    }

    0
}

/// Walk up to find the nearest Cargo.toml, package.json, pyproject.toml, go.mod, etc.
fn package_root(path: &Path) -> Option<&Path> {
    const MANIFESTS: &[&str] = &[
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "go.mod",
        "pom.xml",
        "build.gradle",
    ];
    let mut dir = path.parent()?;
    loop {
        for m in MANIFESTS {
            if dir.join(m).exists() {
                return Some(dir);
            }
        }
        dir = dir.parent()?;
    }
}

/// Check if path contains a vendor directory component.
fn is_vendor_path(path: &Path) -> bool {
    path.components().any(|c| {
        c.as_os_str()
            .to_str()
            .is_some_and(|s| VENDOR_DIRS.contains(&s))
    })
}

/// Check if a path looks like a test file by filename convention.
pub(crate) fn is_test_file(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    // Go: foo_test.go
    // Rust: tests.rs, test.rs, or in tests/ directory
    // JS/TS: foo.test.ts, foo.spec.ts, foo.test.js, foo.spec.js
    // Python: test_foo.py, foo_test.py
    // Java/Kotlin: FooTest.java, FooTest.kt
    // Go: foo_test.go, Rust: foo_test.rs, Python: foo_test.py / test_foo.py
    // JS/TS: foo.test.ts, foo.spec.ts, etc.
    // Java/Kotlin: FooTest.java, FooTest.kt
    // Swift: FooTests.swift
    name.ends_with("_test.go")
        || name.ends_with("_test.rs")
        || name.ends_with("_test.py")
        || (name.starts_with("test_") && name.len() > 8) // test_foo.py (min: test_X.py)
        || name.contains(".test.")
        || name.contains(".spec.")
        || name.ends_with("Test.java")
        || name.ends_with("Test.kt")
        || name.ends_with("Tests.swift")
        || path.components().any(|c| {
            c.as_os_str()
                .to_str()
                .is_some_and(|s| s == "tests" || s == "test" || s == "__tests__")
        })
}

/// 0-100, newer = higher. Files modified within the last hour get max score.
fn recency(mtime: SystemTime) -> u32 {
    let age = SystemTime::now()
        .duration_since(mtime)
        .unwrap_or_default()
        .as_secs();

    match age {
        0..=3_600 => 100,          // last hour
        3_601..=86_400 => 80,      // last day
        86_401..=604_800 => 50,    // last week
        604_801..=2_592_000 => 20, // last month
        _ => 0,
    }
}

#[cfg(test)]
#[allow(clippy::doc_markdown)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn make_match(path: &str, is_definition: bool, exact: bool, file_lines: u32) -> Match {
        Match {
            path: PathBuf::from(path),
            line: 10,
            column: 0,
            text: "test".to_string(),
            is_definition,
            exact,
            file_lines,
            mtime: SystemTime::now(),
            def_range: None,
            def_name: None,
        }
    }

    /// The +1000 definition bonus is the single most important ranking signal.
    /// Every benchmark task that starts with a symbol search depends on the
    /// definition appearing first — otherwise the agent expands a usage site
    /// and has to do a follow-up search to find the actual implementation.
    #[test]
    fn definitions_rank_above_usages() {
        let mut matches = vec![
            make_match("src/a.rs", false, true, 100),
            make_match("src/b.rs", true, true, 100),
        ];
        let scope = Path::new("/tmp/project");
        sort(&mut matches, "test", scope, None);
        assert!(matches[0].is_definition, "definition should sort first");
    }

    /// Exact word match (+500) prevents substring false positives from
    /// outranking the real target. E.g., searching "Next" shouldn't rank
    /// a match on "NextHandler" above an exact "Next" match.
    #[test]
    fn exact_matches_rank_above_inexact() {
        let mut matches = vec![
            make_match("src/a.rs", false, false, 100),
            make_match("src/b.rs", false, true, 100),
        ];
        let scope = Path::new("/tmp/project");
        sort(&mut matches, "test", scope, None);
        assert!(matches[0].exact, "exact match should sort first");
    }

    /// Vendor penalty (-200) keeps node_modules/vendor results from drowning
    /// out source code. Without this, "Matcher" in a vendored copy could
    /// outrank the project's own trait definition.
    #[test]
    fn vendor_paths_penalized() {
        let mut matches = vec![
            make_match("node_modules/dep/index.js", false, true, 100),
            make_match("src/index.js", false, true, 100),
        ];
        let scope = Path::new("/tmp/project");
        sort(&mut matches, "test", scope, None);
        assert_eq!(
            matches[0].path,
            PathBuf::from("src/index.js"),
            "vendor path should sort last"
        );
    }

    /// Context boost (+100 same dir) is the key signal for multi-step navigation.
    /// When the agent has already read router.go and searches "handleRequest",
    /// results in the same directory should rank higher — the agent is likely
    /// exploring related code in the same package.
    #[test]
    fn context_boosts_same_directory() {
        let mut matches = vec![
            make_match("/tmp/project/other/far.rs", false, true, 100),
            make_match("/tmp/project/src/near.rs", false, true, 100),
        ];
        let scope = Path::new("/tmp/project");
        let context = Path::new("/tmp/project/src/main.rs");
        sort(&mut matches, "test", scope, Some(context));
        assert_eq!(
            matches[0].path,
            PathBuf::from("/tmp/project/src/near.rs"),
            "same-dir match should rank higher with context"
        );
    }

    /// Small file bonus (+50) slightly prefers focused files over large ones.
    /// A 50-line context.go is more likely to be the relevant result than a
    /// 2000-line generated file.
    #[test]
    fn small_files_get_bonus() {
        // Both usage, both exact, same scope distance — only differ on file_lines
        let mut matches = vec![
            make_match("src/big.rs", false, true, 500),
            make_match("src/small.rs", false, true, 50),
        ];
        let scope = Path::new("/tmp/project");
        sort(&mut matches, "test", scope, None);
        assert_eq!(
            matches[0].path,
            PathBuf::from("src/small.rs"),
            "small file should get +50 bonus"
        );
    }

    /// Test files (*_test.go, *_test.rs, *.test.ts, etc.) should rank below
    /// source files. When searching "ServeHTTP" in gin, auth_test.go usages
    /// are noise — the agent needs the real implementation, not test call sites.
    #[test]
    fn test_files_deprioritized() {
        // Use paths where the test file sorts alphabetically BEFORE the source file,
        // so alphabetical tiebreaker can't save us — only a real penalty works.
        let mut matches = vec![
            make_match("src/auth_test.go", false, true, 100),
            make_match("src/router.go", false, true, 100),
        ];
        let scope = Path::new("/tmp/project");
        sort(&mut matches, "test", scope, None);
        assert_eq!(
            matches[0].path,
            PathBuf::from("src/router.go"),
            "test file should rank below source file"
        );
    }

    /// Test file penalty should not override the definition bonus — a definition
    /// in a test file is still more valuable than a usage in source code.
    #[test]
    fn test_file_definition_still_ranks_above_source_usage() {
        let mut matches = vec![
            make_match("src/handler.go", false, true, 100), // usage in source
            make_match("src/handler_test.go", true, true, 100), // definition in test
        ];
        let scope = Path::new("/tmp/project");
        sort(&mut matches, "test", scope, None);
        assert!(
            matches[0].is_definition,
            "definition in test file should still outrank usage in source"
        );
    }

    /// Determinism ensures benchmark results are reproducible — same query
    /// against same codebase always produces the same ranking.
    #[test]
    fn deterministic_ordering() {
        let make_set = || {
            vec![
                make_match("src/c.rs", false, true, 100),
                make_match("src/a.rs", true, false, 200),
                make_match("src/b.rs", false, false, 50),
                make_match("node_modules/x.js", true, true, 10),
            ]
        };
        let scope = Path::new("/tmp/project");

        let mut a = make_set();
        let mut b = make_set();
        sort(&mut a, "test", scope, None);
        sort(&mut b, "test", scope, None);

        let paths_a: Vec<_> = a.iter().map(|m| &m.path).collect();
        let paths_b: Vec<_> = b.iter().map(|m| &m.path).collect();
        assert_eq!(paths_a, paths_b, "same inputs must produce same order");
    }
}
