use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tracks MCP activity across calls.
/// Stored alongside `OutlineCache` in server state.
pub struct Session {
    reads: AtomicUsize,
    searches: AtomicUsize,
    maps: AtomicUsize,
    symbols: Mutex<HashMap<String, usize>>, // query → search count
    dir_hits: Mutex<HashMap<String, usize>>, // dir → count
    expanded: Mutex<HashSet<String>>,       // "path:line" → expanded status
}

impl Session {
    pub fn new() -> Self {
        Session {
            reads: AtomicUsize::new(0),
            searches: AtomicUsize::new(0),
            maps: AtomicUsize::new(0),
            symbols: Mutex::new(HashMap::new()),
            dir_hits: Mutex::new(HashMap::new()),
            expanded: Mutex::new(HashSet::new()),
        }
    }

    pub fn record_read(&self, path: &Path) {
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.record_dir(path);
    }

    pub fn record_search(&self, query: &str) {
        self.searches.fetch_add(1, Ordering::Relaxed);
        let mut syms = self
            .symbols
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *syms.entry(query.to_string()).or_insert(0) += 1;
    }

    #[allow(dead_code)] // Map disabled in v0.3.2
    pub fn record_map(&self) {
        self.maps.fetch_add(1, Ordering::Relaxed);
    }

    fn record_dir(&self, path: &Path) {
        if let Some(dir) = path.parent() {
            let key = dir.to_string_lossy().to_string();
            let mut dirs = self
                .dir_hits
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *dirs.entry(key).or_insert(0) += 1;
        }
    }

    pub fn summary(&self) -> String {
        let reads = self.reads.load(Ordering::Relaxed);
        let searches = self.searches.load(Ordering::Relaxed);
        let maps = self.maps.load(Ordering::Relaxed);

        let mut out = format!("Files read: {reads} | Searches: {searches} | Maps: {maps}");

        // Top symbols
        let syms = self
            .symbols
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !syms.is_empty() {
            let mut sorted: Vec<_> = syms.iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(a.1));
            let top: Vec<String> = sorted
                .iter()
                .take(5)
                .map(|(name, count)| format!("{name} ({count})"))
                .collect();
            let _ = write!(out, "\nTop queries: {}", top.join(", "));
        }

        // Hot paths
        let dirs = self
            .dir_hits
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !dirs.is_empty() {
            let mut sorted: Vec<_> = dirs.iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(a.1));
            let top: Vec<String> = sorted
                .iter()
                .take(5)
                .map(|(dir, count)| format!("{dir} ({count})"))
                .collect();
            let _ = write!(out, "\nHot paths: {}", top.join(", "));
        }

        out
    }

    pub fn reset(&self) {
        self.reads.store(0, Ordering::Relaxed);
        self.searches.store(0, Ordering::Relaxed);
        self.maps.store(0, Ordering::Relaxed);
        self.symbols
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        self.dir_hits
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        self.expanded
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }

    pub fn is_expanded(&self, path: &Path, line: u32) -> bool {
        let key = format!("{}:{}", path.display(), line);
        self.expanded
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains(&key)
    }

    pub fn record_expand(&self, path: &Path, line: u32) {
        let key = format!("{}:{}", path.display(), line);
        self.expanded
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(key);
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn expand_dedup_tracking() {
        let session = Session::new();
        let path = Path::new("src/main.rs");

        assert!(!session.is_expanded(path, 42));
        session.record_expand(path, 42);
        assert!(session.is_expanded(path, 42));
        // Different line should not be expanded
        assert!(!session.is_expanded(path, 43));
        // Different path, same line
        assert!(!session.is_expanded(Path::new("src/other.rs"), 42));
    }

    #[test]
    fn session_summary_counts() {
        let session = Session::new();
        session.record_read(Path::new("/tmp/a.rs"));
        session.record_read(Path::new("/tmp/b.rs"));
        session.record_search("foo");
        session.record_search("bar");
        session.record_search("foo");

        let summary = session.summary();
        assert!(summary.contains("Files read: 2"), "reads: {summary}");
        assert!(summary.contains("Searches: 3"), "searches: {summary}");
        assert!(summary.contains("foo (2)"), "top query: {summary}");
    }

    #[test]
    fn session_reset_clears_all() {
        let session = Session::new();
        session.record_read(Path::new("/tmp/a.rs"));
        session.record_search("test");
        session.record_expand(Path::new("x.rs"), 1);

        session.reset();

        let summary = session.summary();
        assert!(summary.contains("Files read: 0"), "reads: {summary}");
        assert!(summary.contains("Searches: 0"), "searches: {summary}");
        assert!(!session.is_expanded(Path::new("x.rs"), 1));
    }
}
