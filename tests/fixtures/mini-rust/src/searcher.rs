use crate::Matcher;
use crate::lines::LineIter;

/// Searcher reads input and applies a Matcher to find results.
pub struct Searcher<M: Matcher> {
    matcher: M,
    max_count: Option<usize>,
}

impl<M: Matcher> Searcher<M> {
    pub fn new(matcher: M) -> Self {
        Searcher {
            matcher,
            max_count: None,
        }
    }

    pub fn set_max_count(&mut self, count: usize) {
        self.max_count = Some(count);
    }

    pub fn search(&self, input: &[u8]) -> Vec<usize> {
        let iter = LineIter::new(input);
        let mut results = Vec::new();
        for (offset, line) in iter.enumerate() {
            if self.matcher.is_match(line) {
                results.push(offset);
            }
            if let Some(max) = self.max_count {
                if results.len() >= max {
                    break;
                }
            }
        }
        results
    }
}
