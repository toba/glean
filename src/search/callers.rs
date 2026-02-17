use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use streaming_iterator::StreamingIterator;

use super::treesitter::{DEFINITION_KINDS, extract_definition_name};

use crate::cache::OutlineCache;
use crate::error::GleanError;
use crate::read::detect_file_type;
use crate::read::outline::code::outline_language;
use crate::session::Session;
use crate::types::FileType;

const MAX_MATCHES: usize = 10;
/// Stop walking once we have this many raw matches. Generous headroom for dedup + ranking.
const EARLY_QUIT_THRESHOLD: usize = 30;

/// A single caller match — a call site of a target symbol.
#[derive(Debug)]
pub struct CallerMatch {
    pub path: PathBuf,
    pub line: u32,
    pub calling_function: String,
    pub call_text: String,
    /// Line range of the calling function (for expand).
    pub caller_range: Option<(u32, u32)>,
    /// File content, already read during `find_callers` — avoids re-reading during expand.
    pub content: String,
}

/// Find all call sites of a target symbol across the codebase using tree-sitter.
pub fn find_callers(target: &str, scope: &Path) -> Result<Vec<CallerMatch>, GleanError> {
    let needle = target.as_bytes();

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

            // Only process files with tree-sitter grammars
            let file_type = detect_file_type(path);
            let FileType::Code(lang) = file_type else {
                return Vec::new();
            };

            let Some(ts_lang) = outline_language(lang) else {
                return Vec::new();
            };

            find_callers_treesitter(path, target, &ts_lang, &content, lang)
        },
    ))
}

/// Tree-sitter call site detection.
fn find_callers_treesitter(
    path: &Path,
    target: &str,
    ts_lang: &tree_sitter::Language,
    content: &str,
    lang: crate::types::Lang,
) -> Vec<CallerMatch> {
    // Get the query string for this language
    let Some(query_str) = super::callees::callee_query_str(lang) else {
        return Vec::new();
    };

    // Compile the query
    let Ok(query) = tree_sitter::Query::new(ts_lang, query_str) else {
        return Vec::new();
    };

    let Some(callee_idx) = query.capture_index_for_name("callee") else {
        return Vec::new();
    };

    let Some(tree) = super::treesitter::parse_tree(content, ts_lang) else {
        return Vec::new();
    };

    let content_bytes = content.as_bytes();
    let lines: Vec<&str> = content.lines().collect();
    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), content_bytes);

    let mut callers = Vec::new();

    while let Some(m) = matches.next() {
        for cap in m.captures {
            if cap.index != callee_idx {
                continue;
            }

            // Check if the captured text matches our target symbol
            let Ok(text) = cap.node.utf8_text(content_bytes) else {
                continue;
            };

            if text != target {
                continue;
            }

            // Found a call site! Now walk up to find the calling function
            let line = cap.node.start_position().row as u32 + 1;

            // Get the call text (the whole call expression, not just the callee)
            let call_node = cap.node.parent().unwrap_or(cap.node);
            let same_line = call_node.start_position().row == call_node.end_position().row;
            let call_text: String = if same_line {
                let row = call_node.start_position().row;
                if row < lines.len() {
                    lines[row].trim().to_string()
                } else {
                    text.to_string()
                }
            } else {
                text.to_string()
            };

            // Walk up the tree to find the enclosing function
            let (calling_function, caller_range) = find_enclosing_function(cap.node, &lines);

            callers.push(CallerMatch {
                path: path.to_path_buf(),
                line,
                calling_function,
                call_text,
                caller_range,
                content: content.to_string(),
            });
        }
    }

    callers
}

/// Walk up the AST from a node to find the enclosing function definition.
/// Returns (`function_name`, `line_range`).
fn find_enclosing_function(
    node: tree_sitter::Node,
    lines: &[&str],
) -> (String, Option<(u32, u32)>) {
    // Walk up the tree until we find a definition node
    let mut current = Some(node);

    while let Some(n) = current {
        let kind = n.kind();

        if DEFINITION_KINDS.contains(&kind) {
            // Extract the function name
            let name =
                extract_definition_name(n, lines).unwrap_or_else(|| "<anonymous>".to_string());
            let range = Some((
                n.start_position().row as u32 + 1,
                n.end_position().row as u32 + 1,
            ));
            return (name, range);
        }

        current = n.parent();
    }

    // No enclosing function found — top-level call
    ("<top-level>".to_string(), None)
}

/// Format and rank caller search results with optional expand.
pub fn search_callers_expanded(
    target: &str,
    scope: &Path,
    _cache: &OutlineCache,
    _session: &Session,
    expand: usize,
    context: Option<&Path>,
) -> Result<String, GleanError> {
    let callers = find_callers(target, scope)?;

    if callers.is_empty() {
        return Ok(format!(
            "# Callers of \"{}\" in {} — no call sites found",
            target,
            scope.display()
        ));
    }

    // Sort by relevance (context file first, then by proximity)
    let mut sorted_callers = callers;
    rank_callers(&mut sorted_callers, scope, context);

    let total = sorted_callers.len();
    sorted_callers.truncate(MAX_MATCHES);

    // Format the output
    let mut output = format!(
        "# Callers of \"{}\" in {} — {} call site{}\n",
        target,
        scope.display(),
        total,
        if total == 1 { "" } else { "s" }
    );

    for (i, caller) in sorted_callers.iter().enumerate() {
        // Header: file:line [caller: calling_function]
        let _ = write!(
            output,
            "\n## {}:{} [caller: {}]\n",
            caller
                .path
                .strip_prefix(scope)
                .unwrap_or(&caller.path)
                .display(),
            caller.line,
            caller.calling_function
        );

        // Show the call text
        let _ = writeln!(output, "→ {}", caller.call_text);

        // Expand if requested and we have the range
        if i < expand
            && let Some((start, end)) = caller.caller_range
        {
            // Use cached content — no re-read needed
            let lines: Vec<&str> = caller.content.lines().collect();
            let start_idx = (start as usize).saturating_sub(1);
            let end_idx = (end as usize).min(lines.len());

            output.push('\n');
            output.push_str("```\n");

            for (idx, line) in lines[start_idx..end_idx].iter().enumerate() {
                let line_num = start_idx + idx + 1;
                let prefix = if line_num == caller.line as usize {
                    "► "
                } else {
                    "  "
                };
                let _ = writeln!(output, "{prefix}{line_num:4} │ {line}");
            }

            output.push_str("```\n");
        }
    }

    // Show token estimate
    let token_est = crate::types::estimate_tokens(output.len() as u64);
    let _ = writeln!(output, "\n[~{token_est} tokens]");

    Ok(output)
}

/// Simple ranking: context file first, then by path length (proximity heuristic).
fn rank_callers(callers: &mut [CallerMatch], scope: &Path, context: Option<&Path>) {
    callers.sort_by(|a, b| {
        // Context file wins
        if let Some(ctx) = context {
            match (a.path == ctx, b.path == ctx) {
                (true, false) => return std::cmp::Ordering::Less,
                (false, true) => return std::cmp::Ordering::Greater,
                _ => {}
            }
        }

        // Shorter paths (more similar to scope) rank higher
        let a_rel = a.path.strip_prefix(scope).unwrap_or(&a.path);
        let b_rel = b.path.strip_prefix(scope).unwrap_or(&b.path);
        a_rel
            .components()
            .count()
            .cmp(&b_rel.components().count())
            .then_with(|| a.path.cmp(&b.path))
            .then_with(|| a.line.cmp(&b.line))
    });
}

#[cfg(test)]
#[allow(clippy::doc_markdown)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    /// Benchmark analog: gin_middleware_chain — after finding Continue's definition,
    /// the agent needs to find who CALLS Continue. Quality signals:
    /// 1. calling_function is populated (tells agent which function to read next)
    /// 2. caller_range is populated (enables expand without a follow-up read)
    /// 3. The middleware file (where Logger calls c.Continue()) is found
    ///
    /// Without these, the agent needs extra tool calls to understand call chains.
    #[test]
    fn callers_provide_full_navigation_context() {
        let callers = find_callers("Continue", &fixture("mini-go")).unwrap();
        assert!(!callers.is_empty(), "should find call sites for Next");

        // Must find the middleware call site
        let middleware_caller = callers
            .iter()
            .find(|c| c.path.to_string_lossy().contains("middleware.go"));
        assert!(
            middleware_caller.is_some(),
            "should find caller in middleware.go"
        );

        let mc = middleware_caller.unwrap();

        // calling_function populated = agent knows WHICH function calls Next
        assert!(
            !mc.calling_function.is_empty(),
            "calling_function must be populated for navigation"
        );

        // caller_range populated = agent can expand the caller without a separate read
        assert!(
            mc.caller_range.is_some(),
            "caller_range must be populated to enable expand"
        );

        // content cached = no redundant file read during expand
        assert!(
            !mc.content.is_empty(),
            "content should be cached from the initial read"
        );
    }

    /// Also find Continue callers in router.go — handleRequest calls c.Continue().
    /// This tests that multiple call sites across files are all found.
    #[test]
    fn finds_callers_across_multiple_files() {
        let callers = find_callers("Continue", &fixture("mini-go")).unwrap();
        let files: std::collections::HashSet<_> = callers
            .iter()
            .map(|c| c.path.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        // Next is called in both middleware.go (Logger) and router.go (handleRequest)
        assert!(
            files.len() >= 2,
            "should find callers in multiple files, got: {files:?}"
        );
    }

    #[test]
    fn no_callers_returns_empty() {
        let callers = find_callers("nonexistent_function_xyz", &fixture("mini-go")).unwrap();
        assert!(callers.is_empty());
    }
}
