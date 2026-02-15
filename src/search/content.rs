use std::path::Path;

use super::file_metadata;

use crate::error::GleanError;
use crate::search::rank;
use crate::types::{Match, SearchResult};
use grep_regex::RegexMatcher;
use grep_searcher::Searcher;
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
            let mut searcher = Searcher::new();

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
