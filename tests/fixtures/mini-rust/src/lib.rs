pub mod lines;
pub mod searcher;

/// Matcher trait for pattern matching against byte slices.
pub trait Matcher {
    fn find(&self, haystack: &[u8]) -> Option<usize>;
    fn is_match(&self, haystack: &[u8]) -> bool;
}

/// A regex-based matcher implementation.
pub struct RegexMatcher {
    pattern: String,
}

impl RegexMatcher {
    pub fn new(pattern: &str) -> Self {
        RegexMatcher {
            pattern: pattern.to_string(),
        }
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }
}

impl Matcher for RegexMatcher {
    fn find(&self, _haystack: &[u8]) -> Option<usize> {
        Some(0)
    }

    fn is_match(&self, _haystack: &[u8]) -> bool {
        true
    }
}
