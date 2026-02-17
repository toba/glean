use crate::task::{GroundTruth, Task};
use std::path::PathBuf;

/// Resolve the project root (parent of `benchmark/`).
fn project_root() -> PathBuf {
    let benchmark_dir = crate::config::benchmark_dir();
    // benchmark_dir is either `<root>/benchmark` or `<root>` if cwd is already in benchmark
    if benchmark_dir.join("tests/fixtures").exists() {
        benchmark_dir.clone()
    } else {
        benchmark_dir
            .parent()
            .expect("benchmark dir has no parent")
            .to_path_buf()
    }
}

fn fixture_dir(name: &str) -> PathBuf {
    project_root().join("tests/fixtures").join(name)
}

// ---------------------------------------------------------------------------
// Rust: find Matcher trait + all implementors
// ---------------------------------------------------------------------------

pub struct RustTraitImpls;
impl Task for RustTraitImpls {
    fn name(&self) -> &'static str {
        "eval_rust_trait_impls"
    }
    fn repo(&self) -> &'static str {
        "mini-rust"
    }
    fn prompt(&self) -> &'static str {
        "This is a small Rust crate in the current directory. Only look at files in this directory. \
         Find the `Matcher` trait definition in src/lib.rs. Then find ALL types that implement \
         this trait — there are multiple implementors. For each implementor, show where it is \
         defined and list its methods."
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "Matcher",
            "RegexMatcher",
            "LiteralMatcher",
            "find",
            "is_match",
        ])
    }
    fn work_dir(&self) -> Option<PathBuf> {
        Some(fixture_dir("mini-rust"))
    }
}

// ---------------------------------------------------------------------------
// Rust: rename Matcher → PatternMatcher everywhere
// ---------------------------------------------------------------------------

pub struct RustRenameTrait;
impl Task for RustRenameTrait {
    fn name(&self) -> &'static str {
        "eval_rust_rename_trait"
    }
    fn repo(&self) -> &'static str {
        "mini-rust"
    }
    fn prompt(&self) -> &'static str {
        "This is a small Rust crate in the current directory. Only edit files in this directory. \
         Rename the `Matcher` trait to `PatternMatcher` everywhere it appears — \
         the trait definition in src/lib.rs, all impl blocks, and all usage sites in other files \
         like src/searcher.rs. Do not search outside this directory."
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::with_edit(
            vec!["PatternMatcher"],
            "src/lib.rs",
            vec!["PatternMatcher"],
        )
    }
    fn work_dir(&self) -> Option<PathBuf> {
        Some(fixture_dir("mini-rust"))
    }
}

// ---------------------------------------------------------------------------
// Go: trace ServeHTTP → handleRequest → Next → middleware
// ---------------------------------------------------------------------------

pub struct GoRequestFlow;
impl Task for GoRequestFlow {
    fn name(&self) -> &'static str {
        "eval_go_request_flow"
    }
    fn repo(&self) -> &'static str {
        "mini-go"
    }
    fn prompt(&self) -> &'static str {
        "This is a small Go package in the current directory. Only look at files in this directory. \
         Trace the full request handling flow starting from `ServeHTTP` in router.go. Show how \
         a request flows through `handleRequest`, `Next()`, and into the middleware chain. \
         Explain the role of the `index` field on Context."
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "ServeHTTP",
            "handleRequest",
            "Next",
            "index",
            "handlers",
        ])
    }
    fn work_dir(&self) -> Option<PathBuf> {
        Some(fixture_dir("mini-go"))
    }
}

// ---------------------------------------------------------------------------
// Go: rename Next() → Continue() in all call sites
// ---------------------------------------------------------------------------

pub struct GoRenameMethod;
impl Task for GoRenameMethod {
    fn name(&self) -> &'static str {
        "eval_go_rename_method"
    }
    fn repo(&self) -> &'static str {
        "mini-go"
    }
    fn prompt(&self) -> &'static str {
        "This is a small Go package in the current directory. Only edit files in this directory. \
         Rename the `Next()` method on `Context` to `Continue()`. Update the method definition \
         in context.go and all call sites in other .go files. Do not search outside this directory."
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::with_edit(vec!["Continue"], "context.go", vec!["Continue"])
    }
    fn work_dir(&self) -> Option<PathBuf> {
        Some(fixture_dir("mini-go"))
    }
}

// ---------------------------------------------------------------------------
// TypeScript: find ZodError and all files that use it
// ---------------------------------------------------------------------------

pub struct TsClassUsage;
impl Task for TsClassUsage {
    fn name(&self) -> &'static str {
        "eval_ts_class_usage"
    }
    fn repo(&self) -> &'static str {
        "mini-ts"
    }
    fn prompt(&self) -> &'static str {
        "This is a small TypeScript project in the current directory. Only look at files in this \
         directory. Find the `ZodError` class definition in src/errors.ts. Then find every file \
         in src/ that imports or uses `ZodError`. For each usage site, show the relevant code."
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec!["ZodError", "errors.ts", "parse.ts", "ZodIssue"])
    }
    fn work_dir(&self) -> Option<PathBuf> {
        Some(fixture_dir("mini-ts"))
    }
}

// ---------------------------------------------------------------------------
// Swift: trace Session → DataRequest → Validation
// ---------------------------------------------------------------------------

pub struct SwiftChain;
impl Task for SwiftChain {
    fn name(&self) -> &'static str {
        "eval_swift_chain"
    }
    fn repo(&self) -> &'static str {
        "mini-swift"
    }
    fn prompt(&self) -> &'static str {
        "This is a small Swift project in the current directory. Only look at .swift files here. \
         Trace the request lifecycle: starting from `Session.request()` in Session.swift, show \
         how a `DataRequest` is created, how `validate()` is called on it, and how the \
         `Validation` struct in Validation.swift performs status code checking using \
         `acceptableStatusCodes`."
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "Session",
            "DataRequest",
            "Validation",
            "validate",
            "acceptableStatusCodes",
        ])
    }
    fn work_dir(&self) -> Option<PathBuf> {
        Some(fixture_dir("mini-swift"))
    }
}
