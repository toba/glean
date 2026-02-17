use std::path::Path;

use super::file_metadata;

use crate::error::GleanError;
use crate::search::rank;
use crate::types::{Match, SearchResult};
use grep_regex::RegexMatcher;
use grep_searcher::BinaryDetection;
use grep_searcher::SearcherBuilder;
use grep_searcher::sinks::UTF8;

const MAX_MATCHES: usize = 10;
const EARLY_QUIT_THRESHOLD: usize = MAX_MATCHES * 3;
const MAX_SEARCH_FILE_SIZE: u64 = 500_000;

/// Content search using ripgrep crates. Literal by default, regex if `is_regex`.
pub fn search(
    pattern: &str,
    scope: &Path,
    is_regex: bool,
    context: Option<&Path>,
) -> Result<SearchResult, GleanError> {
    let matcher = if is_regex {
        RegexMatcher::new(pattern)
    } else {
        RegexMatcher::new(&regex_syntax::escape(pattern))
    }
    .map_err(|e| GleanError::InvalidQuery {
        query: pattern.to_string(),
        reason: e.to_string(),
    })?;

    let mut all_matches = super::walk_collect(
        scope,
        Some(EARLY_QUIT_THRESHOLD),
        Some(MAX_SEARCH_FILE_SIZE),
        |entry| {
            let path = entry.path();
            let (file_lines, mtime) = file_metadata(path);

            let mut file_matches = Vec::new();
            let mut searcher = SearcherBuilder::new()
                .binary_detection(BinaryDetection::convert(b'\x00'))
                .build();

            let _ = searcher.search_path(
                &matcher,
                path,
                UTF8(|line_num, line| {
                    file_matches.push(Match {
                        path: path.to_path_buf(),
                        line: line_num as u32,
                        column: 0,
                        text: line.trim_end().to_string(),
                        is_definition: false,
                        exact: false,
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
    );

    let total = all_matches.len();

    rank::sort(&mut all_matches, pattern, scope, context);
    all_matches.truncate(MAX_MATCHES);

    Ok(SearchResult {
        query: pattern.to_string(),
        scope: scope.to_path_buf(),
        matches: all_matches,
        total_found: total,
        definitions: 0,
        usages: total,
    })
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

    /// Benchmark analog: gin_client_ip — agent searches "X-Forwarded-For" to find
    /// the header parsing logic. Quality signal: context.go must be the TOP result
    /// (ranked first), not just present. An agent seeing the right file first
    /// avoids a follow-up search.
    #[test]
    fn top_result_is_most_relevant_file() {
        let result = search("X-Forwarded-For", &fixture("mini-go"), false, None).unwrap();
        assert!(result.total_found > 0, "should find X-Forwarded-For");
        let first = &result.matches[0];
        assert!(
            first.path.to_string_lossy().contains("context.go"),
            "context.go (where header is parsed) should rank first, got: {}",
            first.path.display()
        );
    }

    /// Regex content search should find the method definition line, not just
    /// any line mentioning "Continue". The matched text should be the func signature.
    #[test]
    fn regex_search_finds_method_signature() {
        let result = search(r"func \(.*\) Continue", &fixture("mini-go"), true, None).unwrap();
        assert!(
            result.total_found > 0,
            "should find Continue method via regex"
        );
        let first = &result.matches[0];
        assert!(
            first.text.contains("func") && first.text.contains("Continue"),
            "matched text should be the func signature, got: {:?}",
            first.text
        );
    }

    /// Result set should be tight — a focused query in a small codebase shouldn't
    /// return inflated counts. An agent seeing "10 matches" for a unique string
    /// wastes time scanning irrelevant results.
    #[test]
    fn unique_string_returns_tight_count() {
        // "X-Forwarded-For" appears in exactly one file
        let result = search("X-Forwarded-For", &fixture("mini-go"), false, None).unwrap();
        assert!(
            result.total_found <= 3,
            "unique string should have tight result count, got {}",
            result.total_found
        );
    }

    #[test]
    fn no_results_returns_empty() {
        let result = search(
            "xyzzy_nonexistent_string_42",
            &fixture("mini-go"),
            false,
            None,
        )
        .unwrap();
        assert_eq!(result.total_found, 0);
    }
}
