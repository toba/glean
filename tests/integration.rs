//! Integration tests exercising the full `run()` flow.
//!
//! These test what an agent would experience: the formatted output string
//! from a single tool call. Quality is measured by whether the output
//! contains enough information to COMPLETE a benchmark task in one step,
//! rather than just "does it find anything."

use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn run(query: &str, scope: &Path) -> String {
    let cache = glean::cache::OutlineCache::new();
    glean::run(query, scope, None, None, &cache).unwrap()
}

// ---------------------------------------------------------------------------
// Symbol search: definition-first ranking in formatted output
// ---------------------------------------------------------------------------

/// Benchmark analog: gin_servehttp_flow
/// One `glean_search("ServeHTTP")` should give the agent:
///   1. The definition file (router.go) — FIRST in output
///   2. The "definition" tag — so the agent knows to expand it
///   3. Usage sites — so the agent can see where it's called
///
/// If the definition isn't first, the agent wastes a turn scrolling past usages.
#[test]
fn symbol_search_definition_appears_first_in_output() {
    let output = run("ServeHTTP", &fixture("mini-go"));

    // The FIRST ## section should be the definition
    let first_section = output.split("\n\n##").nth(1).expect("should have sections");
    assert!(
        first_section.contains("router.go"),
        "first section should reference router.go:\n{first_section}"
    );
    assert!(
        first_section.contains("[definition]"),
        "first section should be tagged [definition]:\n{first_section}"
    );
}

/// Benchmark analog: rg_trait_implementors
/// Searching "Matcher" should show the trait definition AND usages in other files.
/// The output must contain enough for the agent to know: "trait is in lib.rs,
/// used in searcher.rs" — completing the navigation in one tool call.
#[test]
fn symbol_search_shows_definition_and_cross_file_usages() {
    let output = run("Matcher", &fixture("mini-rust"));

    // Must contain definition
    assert!(
        output.contains("[definition]"),
        "output must contain a definition tag"
    );
    assert!(
        output.contains("lib.rs"),
        "output must show lib.rs (where trait is defined)"
    );

    // Must contain usage in ANOTHER file (the cross-file navigation breadcrumb)
    assert!(
        output.contains("searcher.rs"),
        "output must show searcher.rs (where Matcher is used) — \
         this is the navigation breadcrumb:\n{output}"
    );
}

/// Benchmark analog: zod_string_schema
/// TypeScript class search should find the definition with line range.
/// The line range in the header enables expand in MCP mode.
#[test]
fn ts_class_definition_has_line_range() {
    let output = run("ZodString", &fixture("mini-ts"));

    // Should have a definition section with line range (e.g., "schemas.ts:1-12 [definition]")
    let has_range = output.lines().any(|line| {
        line.contains("schemas.ts") && line.contains("[definition]") && line.contains('-') // line range like "1-12"
    });
    assert!(
        has_range,
        "definition should show line range for expand:\n{output}"
    );
}

/// Benchmark analog: af_session_config
/// Swift class search in a multi-file project.
#[test]
fn swift_class_definition_first() {
    let output = run("Session", &fixture("mini-swift"));

    let first_section = output.split("\n\n##").nth(1).expect("should have sections");
    assert!(
        first_section.contains("Session.swift"),
        "first section should be in Session.swift:\n{first_section}"
    );
    assert!(
        first_section.contains("[definition]"),
        "first section should be [definition]:\n{first_section}"
    );
}

// ---------------------------------------------------------------------------
// Content search: result precision
// ---------------------------------------------------------------------------

/// Benchmark analog: gin_client_ip
/// Searching a string literal should rank the file where it's semantically
/// used (parsed/processed) above files that merely mention it.
#[test]
fn content_search_ranks_relevant_file_first() {
    let output = run("X-Forwarded-For", &fixture("mini-go"));

    // context.go is where the header is actually parsed — should appear first
    let context_pos = output.find("context.go");
    assert!(
        context_pos.is_some(),
        "output must contain context.go:\n{output}"
    );

    // If other files are mentioned, context.go should come first
    if let Some(other_pos) = output.find("middleware.go") {
        assert!(
            context_pos.unwrap() < other_pos,
            "context.go should appear before middleware.go in output"
        );
    }
}

// ---------------------------------------------------------------------------
// Glob: file listing completeness
// ---------------------------------------------------------------------------

/// Glob should list ALL matching files — an incomplete list means the agent
/// doesn't know what files exist and needs follow-up queries.
#[test]
fn glob_lists_all_matching_files() {
    let output = run("*.go", &fixture("mini-go"));
    assert!(output.contains("router.go"), "should list router.go");
    assert!(output.contains("context.go"), "should list context.go");
    assert!(
        output.contains("middleware.go"),
        "should list middleware.go"
    );
    // go.mod should NOT appear (not *.go)
    assert!(!output.contains("go.mod"), "should not list go.mod");
}

// ---------------------------------------------------------------------------
// File read: content completeness
// ---------------------------------------------------------------------------

/// Reading a small file should show FULL content (not outline), including
/// the trait definition the agent is looking for. If this gets truncated
/// to an outline, the agent needs a follow-up section read.
#[test]
fn file_read_shows_full_content_for_small_files() {
    let output = run("src/lib.rs", &fixture("mini-rust"));

    // Should show full mode, not outline
    assert!(
        output.contains("[full]"),
        "small file should show [full] mode:\n{output}"
    );
    // Should contain the actual trait definition
    assert!(
        output.contains("trait Matcher"),
        "should show trait definition"
    );
    // Should contain the impl block
    assert!(
        output.contains("impl Matcher for RegexMatcher"),
        "should show impl block"
    );
}

// ---------------------------------------------------------------------------
// Fallthrough: ambiguous query resolution
// ---------------------------------------------------------------------------

/// Benchmark analog: rg_lineiter_definition
/// "RegexMatcher" looks path-like (no glob chars, valid identifier) but
/// isn't a file. Fallthrough should try symbol search and find the struct
/// definition on the first attempt — no extra tool calls needed.
#[test]
fn fallthrough_resolves_to_symbol_on_first_try() {
    let output = run("RegexMatcher", &fixture("mini-rust"));
    assert!(
        output.contains("[definition]"),
        "fallthrough should resolve to symbol search with definition:\n{output}"
    );
    assert!(
        output.contains("lib.rs"),
        "should find RegexMatcher in lib.rs:\n{output}"
    );
}

// ---------------------------------------------------------------------------
// Error case: clear error for nonexistent paths
// ---------------------------------------------------------------------------

#[test]
fn nonexistent_path_returns_clear_error() {
    let cache = glean::cache::OutlineCache::new();
    let result = glean::run(
        "nonexistent/path.rs",
        &fixture("mini-rust"),
        None,
        None,
        &cache,
    );
    assert!(result.is_err(), "nonexistent path should return Err");
}

// ---------------------------------------------------------------------------
// Budget: output stays within token limits
// ---------------------------------------------------------------------------

/// When a budget is set, the output must not exceed it. This prevents
/// context window blowup in long agent sessions.
#[test]
fn budget_constrains_output_size() {
    let cache = glean::cache::OutlineCache::new();
    let result = glean::run("*.go", &fixture("mini-go"), None, Some(50), &cache).unwrap();
    let tokens = glean::error::GleanError::exit_code; // just need estimate_tokens
    let _ = tokens; // unused, using direct calc
    let est_tokens = (result.len() as u64).div_ceil(4);
    // Allow some overhead for the truncation message itself
    assert!(
        est_tokens <= 80,
        "output should respect budget (~50 tokens), got ~{est_tokens} tokens"
    );
}
