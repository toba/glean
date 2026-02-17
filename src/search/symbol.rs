use std::fs;
use std::path::Path;
use std::time::SystemTime;

use super::file_metadata;
use super::treesitter::{
    DEFINITION_KINDS, extract_definition_name, extract_impl_trait, extract_impl_type,
    extract_implemented_interfaces,
};

use crate::error::GleanError;
use crate::read::detect_file_type;
use crate::read::outline::code::outline_language;
use crate::search::rank;
use crate::types::{FileType, Match, SearchResult};
use grep_regex::RegexMatcher;
use grep_searcher::BinaryDetection;
use grep_searcher::SearcherBuilder;
use grep_searcher::sinks::UTF8;

const MAX_MATCHES: usize = 10;
/// Stop walking once we have this many raw matches. Generous headroom for dedup + ranking.
const EARLY_QUIT_THRESHOLD: usize = MAX_MATCHES * 3;

/// Split a dotted query like `"Session.request"` into `("Session", "request")`.
/// Returns `None` for plain identifiers, empty parts, or multiple dots.
fn split_dotted_query(query: &str) -> Option<(&str, &str)> {
    let dot = query.find('.')?;
    let type_name = &query[..dot];
    let member_name = &query[dot + 1..];
    // Reject empty parts or multiple dots
    if type_name.is_empty() || member_name.is_empty() || member_name.contains('.') {
        return None;
    }
    Some((type_name, member_name))
}

/// Container node kinds that represent types a member can belong to.
const TYPE_CONTAINER_KINDS: &[&str] = &[
    // Classes
    "class_declaration",
    "class_definition",
    // Structs
    "struct_item",
    // Interfaces / protocols
    "interface_declaration",
    "protocol_declaration",
    // Enums
    "enum_item",
    "enum_declaration",
    // Rust impl blocks
    "impl_item",
    // Rust traits
    "trait_item",
    // Go type declarations
    "type_declaration",
];

/// Check if a node is inside a type container with the given name.
/// Walks the `node.parent()` chain looking for a container whose
/// `extract_definition_name() == type_name`.
fn is_inside_type(node: tree_sitter::Node, type_name: &str, lines: &[&str]) -> bool {
    let mut current = node.parent();
    while let Some(n) = current {
        if TYPE_CONTAINER_KINDS.contains(&n.kind())
            && extract_definition_name(n, lines).as_deref() == Some(type_name)
        {
            return true;
        }
        current = n.parent();
    }
    false
}

/// Symbol search: find definitions via tree-sitter, usages via ripgrep, concurrently.
/// Merge results, deduplicate, definitions first.
pub fn search(
    query: &str,
    scope: &Path,
    context: Option<&Path>,
) -> Result<SearchResult, GleanError> {
    // Dotted query: branch to specialized search
    if let Some((type_name, member_name)) = split_dotted_query(query) {
        return search_dotted(query, type_name, member_name, scope, context);
    }

    // Compile regex once, share across both arms
    let word_pattern = format!(r"\b{}\b", regex_syntax::escape(query));
    let matcher = RegexMatcher::new(&word_pattern).map_err(|e| GleanError::InvalidQuery {
        query: query.to_string(),
        reason: e.to_string(),
    })?;

    let (defs, usages) = rayon::join(
        || find_definitions(query, scope),
        || find_usages(query, &matcher, scope),
    );

    let defs = defs?;
    let usages = usages?;

    // Deduplicate: remove usage matches that overlap with definition matches.
    // Linear scan — max ~30 defs from EARLY_QUIT_THRESHOLD, no allocation needed.
    let mut merged: Vec<Match> = defs;
    let def_count = merged.len();

    for m in usages {
        let dominated = merged[..def_count]
            .iter()
            .any(|d| d.path == m.path && d.line == m.line);
        if !dominated {
            merged.push(m);
        }
    }

    let total = merged.len();
    let usage_count = total - def_count;

    rank::sort(&mut merged, query, scope, context);
    merged.truncate(MAX_MATCHES);

    Ok(SearchResult {
        query: query.to_string(),
        scope: scope.to_path_buf(),
        matches: merged,
        total_found: total,
        definitions: def_count,
        usages: usage_count,
    })
}

/// Dotted symbol search: `Type.member` — find member definitions inside Type,
/// plus usages of the member name. Definitions are post-filtered by `is_inside_type`.
fn search_dotted(
    original_query: &str,
    type_name: &str,
    member_name: &str,
    scope: &Path,
    context: Option<&Path>,
) -> Result<SearchResult, GleanError> {
    let word_pattern = format!(r"\b{}\b", regex_syntax::escape(member_name));
    let matcher = RegexMatcher::new(&word_pattern).map_err(|e| GleanError::InvalidQuery {
        query: original_query.to_string(),
        reason: e.to_string(),
    })?;

    let (defs, usages) = rayon::join(
        || find_definitions_dotted(type_name, member_name, scope),
        || find_usages(member_name, &matcher, scope),
    );

    let defs = defs?;
    let usages = usages?;

    let mut merged: Vec<Match> = defs;
    let def_count = merged.len();

    for m in usages {
        let dominated = merged[..def_count]
            .iter()
            .any(|d| d.path == m.path && d.line == m.line);
        if !dominated {
            merged.push(m);
        }
    }

    let total = merged.len();
    let usage_count = total - def_count;

    rank::sort(&mut merged, original_query, scope, context);
    merged.truncate(MAX_MATCHES);

    Ok(SearchResult {
        query: original_query.to_string(),
        scope: scope.to_path_buf(),
        matches: merged,
        total_found: total,
        definitions: def_count,
        usages: usage_count,
    })
}

/// Find definitions using tree-sitter structural detection.
/// For each file containing the query string, parse with tree-sitter and walk
/// definition nodes to see if any declare the queried symbol.
/// Falls back to keyword heuristic for files without grammars.
///
/// Single-read design: reads each file once, checks for symbol via
/// `memchr::memmem` (SIMD), then reuses the buffer for tree-sitter parsing.
/// Early termination: quits the parallel walker once enough defs are found.
fn find_definitions(query: &str, scope: &Path) -> Result<Vec<Match>, GleanError> {
    let needle = query.as_bytes();

    Ok(super::walk_collect(
        scope,
        Some(EARLY_QUIT_THRESHOLD),
        Some(500_000),
        |entry| {
            let path = entry.path();

            // Single read: read file once, use buffer for both check and parse
            let Ok(content) = fs::read_to_string(path) else {
                return Vec::new();
            };

            // Fast byte check via memchr::memmem (SIMD) — skip files without the symbol
            if memchr::memmem::find(content.as_bytes(), needle).is_none() {
                return Vec::new();
            }

            // Get file metadata once per file
            let (file_lines, mtime) = file_metadata(path);

            // Try tree-sitter structural detection
            let file_type = detect_file_type(path);
            let is_code = matches!(file_type, FileType::Code(_));
            let ts_language = match file_type {
                FileType::Code(l) => outline_language(l),
                _ => None,
            };

            let mut file_defs = if let Some(ref ts_lang) = ts_language {
                find_defs_treesitter(path, query, ts_lang, &content, file_lines, mtime)
            } else {
                Vec::new()
            };

            // Fallback: keyword heuristic for code files without tree-sitter grammars.
            // Only for Code files — Markdown fenced code blocks, structured data, etc.
            // must not produce definitions (they're examples, not declarations).
            if file_defs.is_empty() && ts_language.is_none() && is_code {
                file_defs = find_defs_heuristic_buf(path, query, &content, file_lines, mtime);
            }

            file_defs
        },
    ))
}

/// Find definitions for dotted queries: search for `member_name` in files
/// containing `member_name`, then post-filter by `is_inside_type(type_name)`.
fn find_definitions_dotted(
    type_name: &str,
    member_name: &str,
    scope: &Path,
) -> Result<Vec<Match>, GleanError> {
    let needle = member_name.as_bytes();

    Ok(super::walk_collect(
        scope,
        Some(EARLY_QUIT_THRESHOLD),
        Some(500_000),
        |entry| {
            let path = entry.path();

            let Ok(content) = fs::read_to_string(path) else {
                return Vec::new();
            };

            if memchr::memmem::find(content.as_bytes(), needle).is_none() {
                return Vec::new();
            }

            let (file_lines, mtime) = file_metadata(path);

            let file_type = detect_file_type(path);
            let ts_language = match file_type {
                FileType::Code(l) => outline_language(l),
                _ => None,
            };

            if let Some(ref ts_lang) = ts_language {
                find_defs_treesitter_dotted(
                    path,
                    type_name,
                    member_name,
                    ts_lang,
                    &content,
                    file_lines,
                    mtime,
                )
            } else {
                Vec::new()
            }
        },
    ))
}

/// Tree-sitter dotted definition detection: find `member_name` definitions
/// that are inside a container named `type_name`.
fn find_defs_treesitter_dotted(
    path: &Path,
    type_name: &str,
    member_name: &str,
    ts_lang: &tree_sitter::Language,
    content: &str,
    file_lines: u32,
    mtime: SystemTime,
) -> Vec<Match> {
    let Some(tree) = super::treesitter::parse_tree(content, ts_lang) else {
        return Vec::new();
    };

    let lines: Vec<&str> = content.lines().collect();
    let root = tree.root_node();
    let mut defs = Vec::new();

    walk_for_definitions_dotted(
        root,
        type_name,
        member_name,
        path,
        &lines,
        file_lines,
        mtime,
        &mut defs,
        0,
    );

    defs
}

/// Recursively walk AST looking for definitions of `member_name` inside `type_name`.
/// Depth limit 4 (vs 3 for plain search) to handle deeper nesting.
fn walk_for_definitions_dotted(
    node: tree_sitter::Node,
    type_name: &str,
    member_name: &str,
    path: &Path,
    lines: &[&str],
    file_lines: u32,
    mtime: SystemTime,
    defs: &mut Vec<Match>,
    depth: usize,
) {
    if depth > 4 {
        return;
    }

    let kind = node.kind();

    if DEFINITION_KINDS.contains(&kind)
        && let Some(name) = extract_definition_name(node, lines)
        && name == member_name
        && is_inside_type(node, type_name, lines)
    {
        let line_num = node.start_position().row as u32 + 1;
        let line_text = lines
            .get(node.start_position().row)
            .unwrap_or(&"")
            .trim_end();
        defs.push(Match {
            path: path.to_path_buf(),
            line: line_num,
            column: node.start_position().column as u32,
            text: line_text.to_string(),
            is_definition: true,
            exact: true,
            file_lines,
            mtime,
            def_range: Some((
                node.start_position().row as u32 + 1,
                node.end_position().row as u32 + 1,
            )),
            def_name: Some(format!("{type_name}.{member_name}")),
        });
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_for_definitions_dotted(
            child,
            type_name,
            member_name,
            path,
            lines,
            file_lines,
            mtime,
            defs,
            depth + 1,
        );
    }
}

/// Tree-sitter structural definition detection.
/// Accepts pre-read content — no redundant file read.
fn find_defs_treesitter(
    path: &Path,
    query: &str,
    ts_lang: &tree_sitter::Language,
    content: &str,
    file_lines: u32,
    mtime: SystemTime,
) -> Vec<Match> {
    let Some(tree) = super::treesitter::parse_tree(content, ts_lang) else {
        return Vec::new();
    };

    let lines: Vec<&str> = content.lines().collect();
    let root = tree.root_node();
    let mut defs = Vec::new();

    walk_for_definitions(root, query, path, &lines, file_lines, mtime, &mut defs, 0);

    defs
}

/// Recursively walk AST nodes looking for definitions of the queried symbol.
fn walk_for_definitions(
    node: tree_sitter::Node,
    query: &str,
    path: &Path,
    lines: &[&str],
    file_lines: u32,
    mtime: SystemTime,
    defs: &mut Vec<Match>,
    depth: usize,
) {
    if depth > 3 {
        return;
    }

    let kind = node.kind();

    if DEFINITION_KINDS.contains(&kind) {
        // Standard definition check: name matches query directly
        if let Some(name) = extract_definition_name(node, lines)
            && name == query
        {
            let line_num = node.start_position().row as u32 + 1;
            let line_text = lines
                .get(node.start_position().row)
                .unwrap_or(&"")
                .trim_end();
            defs.push(Match {
                path: path.to_path_buf(),
                line: line_num,
                column: node.start_position().column as u32,
                text: line_text.to_string(),
                is_definition: true,
                exact: true,
                file_lines,
                mtime,
                def_range: Some((
                    node.start_position().row as u32 + 1,
                    node.end_position().row as u32 + 1,
                )),
                def_name: Some(query.to_string()),
            });
        }

        // Impl/trait detection: `impl Trait for Type` — surface when searching for the trait
        if kind == "impl_item"
            && let Some(trait_name) = extract_impl_trait(node, lines)
            && trait_name == query
            && let Some(impl_type) = extract_impl_type(node, lines)
        {
            let line_num = node.start_position().row as u32 + 1;
            let line_text = lines
                .get(node.start_position().row)
                .unwrap_or(&"")
                .trim_end();
            defs.push(Match {
                path: path.to_path_buf(),
                line: line_num,
                column: node.start_position().column as u32,
                text: line_text.to_string(),
                is_definition: true,
                exact: true,
                file_lines,
                mtime,
                def_range: Some((
                    node.start_position().row as u32 + 1,
                    node.end_position().row as u32 + 1,
                )),
                def_name: Some(format!("impl {query} for {impl_type}")),
            });
        }

        // Class implements interface: `class Foo implements Bar`
        if kind == "class_declaration" || kind == "class_definition" {
            let interfaces = extract_implemented_interfaces(node, lines);
            if interfaces.iter().any(|i| i == query) {
                let class_name =
                    extract_definition_name(node, lines).unwrap_or_else(|| "<class>".into());
                let line_num = node.start_position().row as u32 + 1;
                let line_text = lines
                    .get(node.start_position().row)
                    .unwrap_or(&"")
                    .trim_end();
                defs.push(Match {
                    path: path.to_path_buf(),
                    line: line_num,
                    column: node.start_position().column as u32,
                    text: line_text.to_string(),
                    is_definition: true,
                    exact: true,
                    file_lines,
                    mtime,
                    def_range: Some((
                        node.start_position().row as u32 + 1,
                        node.end_position().row as u32 + 1,
                    )),
                    def_name: Some(format!("{class_name} implements {query}")),
                });
            }
        }
    }

    // Recurse into children (for nested definitions, class bodies, impl blocks, etc.)
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_for_definitions(
            child,
            query,
            path,
            lines,
            file_lines,
            mtime,
            defs,
            depth + 1,
        );
    }
}

/// Keyword heuristic fallback for files without tree-sitter grammars.
/// Operates on pre-read buffer — no redundant file read.
fn find_defs_heuristic_buf(
    path: &Path,
    query: &str,
    content: &str,
    file_lines: u32,
    mtime: SystemTime,
) -> Vec<Match> {
    let mut defs = Vec::new();

    for (i, line) in content.lines().enumerate() {
        if line.contains(query) && is_definition_line(line) {
            defs.push(Match {
                path: path.to_path_buf(),
                line: (i + 1) as u32,
                column: 0,
                text: line.trim_end().to_string(),
                is_definition: true,
                exact: true,
                file_lines,
                mtime,
                def_range: None,
                def_name: Some(query.to_string()),
            });
        }
    }

    defs
}

/// Find all usages via ripgrep (word-boundary matching).
/// Collects per-file, locks once per file (not per line).
/// Early termination once enough usages found.
fn find_usages(
    query: &str,
    matcher: &RegexMatcher,
    scope: &Path,
) -> Result<Vec<Match>, GleanError> {
    Ok(super::walk_collect(
        scope,
        Some(EARLY_QUIT_THRESHOLD),
        Some(500_000),
        |entry| {
            let path = entry.path();
            let (file_lines, mtime) = file_metadata(path);

            let mut file_matches = Vec::new();
            let mut searcher = SearcherBuilder::new()
                .binary_detection(BinaryDetection::convert(b'\x00'))
                .build();

            let _ = searcher.search_path(
                matcher,
                path,
                UTF8(|line_num, line| {
                    file_matches.push(Match {
                        path: path.to_path_buf(),
                        line: line_num as u32,
                        column: 0,
                        text: line.trim_end().to_string(),
                        is_definition: false,
                        exact: line.contains(query),
                        file_lines,
                        mtime,
                        def_range: None,
                        def_name: None,
                    });
                    Ok(true)
                }),
            );

            file_matches
        },
    ))
}

/// Keyword heuristic fallback — only used when tree-sitter grammar unavailable.
fn is_definition_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("fn ")
        || trimmed.starts_with("pub fn ")
        || trimmed.starts_with("pub(crate) fn ")
        || trimmed.starts_with("async fn ")
        || trimmed.starts_with("pub async fn ")
        || trimmed.starts_with("function ")
        || trimmed.starts_with("export function ")
        || trimmed.starts_with("export default function ")
        || trimmed.starts_with("export async function ")
        || trimmed.starts_with("async function ")
        || trimmed.starts_with("const ")
        || trimmed.starts_with("export const ")
        || trimmed.starts_with("let ")
        || trimmed.starts_with("export let ")
        || trimmed.starts_with("var ")
        || trimmed.starts_with("export var ")
        || trimmed.starts_with("class ")
        || trimmed.starts_with("export class ")
        || trimmed.starts_with("interface ")
        || trimmed.starts_with("export interface ")
        || trimmed.starts_with("type ")
        || trimmed.starts_with("export type ")
        || trimmed.starts_with("struct ")
        || trimmed.starts_with("pub struct ")
        || trimmed.starts_with("enum ")
        || trimmed.starts_with("pub enum ")
        || trimmed.starts_with("trait ")
        || trimmed.starts_with("pub trait ")
        || trimmed.starts_with("impl ")
        || trimmed.starts_with("def ")
        || trimmed.starts_with("async def ")
        || trimmed.starts_with("func ")
}

#[cfg(test)]
#[allow(clippy::doc_markdown)]
pub(crate) mod tests {
    use super::*;
    use std::time::SystemTime;

    /// Helper for cross-module tests.
    pub fn find_defs_for_test(
        path: &Path,
        query: &str,
        ts_lang: &tree_sitter::Language,
        content: &str,
    ) -> Vec<Match> {
        find_defs_treesitter(path, query, ts_lang, content, 100, SystemTime::now())
    }

    #[test]
    fn rust_definitions_detected() {
        let code = r#"pub fn hello(name: &str) -> String {
    format!("Hello, {}", name)
}

pub struct Foo {
    bar: i32,
}

pub(crate) fn dispatch_tool(tool: &str) -> Result<String, String> {
    match tool {
        "read" => Ok("read".to_string()),
        _ => Err("unknown".to_string()),
    }
}
"#;
        let ts_lang =
            crate::read::outline::code::outline_language(crate::types::Lang::Rust).unwrap();

        let defs = find_defs_treesitter(
            std::path::Path::new("test.rs"),
            "hello",
            &ts_lang,
            code,
            15,
            SystemTime::now(),
        );
        assert!(!defs.is_empty(), "should find 'hello' definition");
        assert!(defs[0].is_definition);
        assert!(defs[0].def_range.is_some());

        let defs = find_defs_treesitter(
            std::path::Path::new("test.rs"),
            "Foo",
            &ts_lang,
            code,
            15,
            SystemTime::now(),
        );
        assert!(!defs.is_empty(), "should find 'Foo' definition");

        let defs = find_defs_treesitter(
            std::path::Path::new("test.rs"),
            "dispatch_tool",
            &ts_lang,
            code,
            15,
            SystemTime::now(),
        );
        assert!(!defs.is_empty(), "should find 'dispatch_tool' definition");
    }

    fn fixture(name: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    /// Benchmark analog: gin_servehttp_flow — agent searches "ServeHTTP".
    /// Quality signal: the definition in router.go must be matches[0].
    /// An agent that sees the definition first can expand it immediately
    /// instead of wading through usages.
    #[test]
    fn definition_ranks_first_go() {
        let result = search("ServeHTTP", &fixture("mini-go"), None).unwrap();
        assert!(result.definitions > 0, "should find ServeHTTP definition");
        let first = &result.matches[0];
        assert!(first.is_definition, "matches[0] must be a definition");
        assert!(
            first.path.to_string_lossy().contains("router.go"),
            "definition should be in router.go, got: {}",
            first.path.display()
        );
        // def_range must be populated — this is what enables expand
        assert!(
            first.def_range.is_some(),
            "definition must have def_range for expand to work"
        );
    }

    /// Benchmark analog: rg_trait_implementors — agent searches "PatternMatcher".
    /// Quality signals:
    /// 1. Definition (trait) ranks first
    /// 2. Usages in other files appear too (these are the navigation breadcrumbs)
    /// 3. def_range is populated so expand can show the full trait body
    #[test]
    fn definition_first_with_cross_file_usages() {
        let result = search("PatternMatcher", &fixture("mini-rust"), None).unwrap();
        let first = &result.matches[0];
        assert!(first.is_definition, "matches[0] must be the definition");
        assert!(
            first.path.to_string_lossy().contains("lib.rs"),
            "trait definition should be in lib.rs, got: {}",
            first.path.display()
        );
        assert!(first.def_range.is_some(), "def needs range for expand");

        // Usages must exist in OTHER files — this is the cross-file navigation signal
        let cross_file_usages: Vec<_> = result
            .matches
            .iter()
            .filter(|m| !m.is_definition && !m.path.to_string_lossy().contains("lib.rs"))
            .collect();
        assert!(
            !cross_file_usages.is_empty(),
            "should find usages in searcher.rs (the next file an agent would read)"
        );
    }

    /// Benchmark analog: gin_middleware_chain — agent searches "Continue" which has
    /// a definition AND call sites. Quality signals:
    /// 1. No duplicate (path, line) pairs — agent shouldn't see the same match twice
    /// 2. Both definition and usages present
    /// 3. Result count is not inflated (small codebase = small result set)
    #[test]
    fn results_deduped_and_balanced() {
        let result = search("Continue", &fixture("mini-go"), None).unwrap();

        // No duplicates
        let mut seen = std::collections::HashSet::new();
        for m in &result.matches {
            let key = (m.path.clone(), m.line);
            assert!(seen.insert(key), "duplicate (path, line) in results");
        }

        // Should have both definitions and usages
        assert!(result.definitions > 0, "should find Continue definition");
        assert!(
            result.usages > 0,
            "should find Continue usages (call sites)"
        );

        // Result count should be tight for a 4-file codebase
        assert!(
            result.total_found <= 10,
            "small codebase shouldn't produce inflated results, got {}",
            result.total_found
        );
    }

    /// Benchmark analog: af_session_config — searching "Session" in Alamofire
    /// returned Documentation/AdvancedUsage.md code examples as top "definitions"
    /// above the actual Session class. Markdown fenced code blocks must never
    /// produce definitions — they're examples, not declarations.
    #[test]
    fn markdown_code_examples_not_classified_as_definitions() {
        // mini-rust has a README.md with ```rust code blocks mentioning PatternMatcher and RegexMatcher
        let result = search("PatternMatcher", &fixture("mini-rust"), None).unwrap();

        for m in &result.matches {
            if m.is_definition {
                assert!(
                    !m.path.to_string_lossy().ends_with(".md"),
                    "Markdown file should not produce definitions, got: {}:{}",
                    m.path.display(),
                    m.line
                );
            }
        }

        // The actual definition should still be in lib.rs
        assert!(
            result.matches[0].is_definition,
            "first result should be a definition"
        );
        assert!(
            result.matches[0].path.to_string_lossy().contains("lib.rs"),
            "definition should be in lib.rs, not {}",
            result.matches[0].path.display()
        );
    }

    /// Context-aware ranking: when a context file is provided, the definition
    /// should STILL be first (definition bonus > context bonus). This ensures
    /// that context doesn't accidentally demote definitions below usages.
    #[test]
    fn context_does_not_demote_definitions() {
        let scope = fixture("mini-rust");
        let context = scope.join("src/searcher.rs");
        let result = search("PatternMatcher", &scope, Some(&context)).unwrap();

        // Even with context pointing at searcher.rs, definitions must still be first
        // (definition +1000 > context +100)
        assert!(
            result.matches[0].is_definition,
            "definition must still rank first even with context set"
        );
    }

    #[test]
    fn swift_definitions_detected() {
        let code = r"protocol Drawable {
    func draw()
}

class Shape {
    func render() {}
}

struct Point {
    var x: Double
}

func globalHelper() -> Bool {
    return true
}
";
        let ts_lang =
            crate::read::outline::code::outline_language(crate::types::Lang::Swift).unwrap();

        let defs = find_defs_treesitter(
            std::path::Path::new("test.swift"),
            "Shape",
            &ts_lang,
            code,
            15,
            SystemTime::now(),
        );
        assert!(!defs.is_empty(), "should find 'Shape' definition");
        assert!(defs[0].is_definition);
        assert!(defs[0].def_range.is_some());

        let defs = find_defs_treesitter(
            std::path::Path::new("test.swift"),
            "Drawable",
            &ts_lang,
            code,
            15,
            SystemTime::now(),
        );
        assert!(!defs.is_empty(), "should find 'Drawable' definition");

        let defs = find_defs_treesitter(
            std::path::Path::new("test.swift"),
            "globalHelper",
            &ts_lang,
            code,
            15,
            SystemTime::now(),
        );
        assert!(!defs.is_empty(), "should find 'globalHelper' definition");
    }

    /// Searching for a trait name should surface `impl Trait for Type` blocks
    /// as definitions, so agents can discover all implementors.
    #[test]
    fn rust_impl_trait_detected_by_trait_name() {
        let code = r#"pub trait PatternMatcher {
    fn find(&self) -> bool;
}

pub struct Regex {
    pattern: String,
}

impl PatternMatcher for Regex {
    fn find(&self) -> bool {
        true
    }
}

impl Regex {
    pub fn new(p: &str) -> Self {
        Regex { pattern: p.to_string() }
    }
}
"#;
        let ts_lang =
            crate::read::outline::code::outline_language(crate::types::Lang::Rust).unwrap();

        // Searching "PatternMatcher" should find both the trait AND the impl
        let defs = find_defs_treesitter(
            std::path::Path::new("test.rs"),
            "PatternMatcher",
            &ts_lang,
            code,
            20,
            SystemTime::now(),
        );
        assert!(
            defs.len() >= 2,
            "should find trait def + impl block, got {}",
            defs.len()
        );

        let trait_def = defs
            .iter()
            .find(|d| d.def_name.as_deref() == Some("PatternMatcher"));
        assert!(trait_def.is_some(), "should find trait definition");

        let impl_def = defs
            .iter()
            .find(|d| d.def_name.as_deref() == Some("impl PatternMatcher for Regex"));
        assert!(
            impl_def.is_some(),
            "should find impl PatternMatcher for Regex"
        );
        assert!(impl_def.unwrap().def_range.is_some());
    }

    /// Searching for a type name should find bare `impl Type` blocks.
    #[test]
    fn rust_bare_impl_detected_by_type_name() {
        let code = r#"pub struct Foo {
    x: i32,
}

impl Foo {
    pub fn new() -> Self {
        Foo { x: 0 }
    }
}
"#;
        let ts_lang =
            crate::read::outline::code::outline_language(crate::types::Lang::Rust).unwrap();

        let defs = find_defs_treesitter(
            std::path::Path::new("test.rs"),
            "Foo",
            &ts_lang,
            code,
            20,
            SystemTime::now(),
        );
        // Should find both the struct and the bare impl
        assert!(
            defs.len() >= 2,
            "should find struct + impl Foo, got {}",
            defs.len()
        );
    }

    /// TypeScript class implements interface detection.
    #[test]
    fn typescript_class_implements_interface() {
        let code = r#"interface Serializable {
    serialize(): string;
}

interface Loggable {
    log(): void;
}

class User implements Serializable, Loggable {
    serialize(): string { return ""; }
    log(): void {}
}
"#;
        let ts_lang =
            crate::read::outline::code::outline_language(crate::types::Lang::TypeScript).unwrap();

        // Searching "Serializable" should find interface def + implementing class
        let defs = find_defs_treesitter(
            std::path::Path::new("test.ts"),
            "Serializable",
            &ts_lang,
            code,
            20,
            SystemTime::now(),
        );
        assert!(
            defs.len() >= 2,
            "should find interface + implementing class, got {}",
            defs.len()
        );

        let class_impl = defs
            .iter()
            .find(|d| d.def_name.as_deref() == Some("User implements Serializable"));
        assert!(
            class_impl.is_some(),
            "should find User implements Serializable"
        );
    }

    /// Integration test: searching "PatternMatcher" in mini-rust should now find
    /// both the trait definition AND the impl block as definitions.
    #[test]
    fn impl_trait_surfaces_in_symbol_search() {
        let result = search("PatternMatcher", &fixture("mini-rust"), None).unwrap();
        assert!(
            result.definitions >= 2,
            "should find trait + impl as definitions, got {}",
            result.definitions
        );

        let impl_match = result.matches.iter().find(|m| {
            m.def_name
                .as_ref()
                .is_some_and(|n| n.starts_with("impl PatternMatcher"))
        });
        assert!(
            impl_match.is_some(),
            "should find impl PatternMatcher for RegexMatcher as a definition"
        );
    }

    // ── Dotted symbol search tests ──

    #[test]
    fn split_dotted_valid() {
        assert_eq!(
            split_dotted_query("Session.request"),
            Some(("Session", "request"))
        );
        assert_eq!(split_dotted_query("Foo.bar"), Some(("Foo", "bar")));
    }

    #[test]
    fn split_dotted_rejects_plain() {
        assert_eq!(split_dotted_query("request"), None);
    }

    #[test]
    fn split_dotted_rejects_empty_parts() {
        assert_eq!(split_dotted_query(".request"), None);
        assert_eq!(split_dotted_query("Session."), None);
    }

    #[test]
    fn split_dotted_rejects_multi_dot() {
        assert_eq!(split_dotted_query("a.b.c"), None);
    }

    /// Integration test: `Session.request` should find the `request` method
    /// inside the `Session` class in mini-swift.
    #[test]
    fn dotted_symbol_search_swift() {
        let result = search("Session.request", &fixture("mini-swift"), None).unwrap();
        assert!(
            result.definitions > 0,
            "should find Session.request definition, got 0 defs out of {} matches",
            result.matches.len()
        );

        let def = result.matches.iter().find(|m| m.is_definition).unwrap();
        assert!(
            def.path.to_string_lossy().contains("Session.swift"),
            "definition should be in Session.swift, got: {}",
            def.path.display()
        );
        assert_eq!(def.def_name.as_deref(), Some("Session.request"));
        assert!(def.def_range.is_some());
    }
}
