use std::path::{Path, PathBuf};
use std::process::Command;

/// Expected elements for correctness validation.
#[derive(Clone)]
pub struct GroundTruth {
    pub required_strings: Vec<&'static str>,
    pub forbidden_strings: Vec<&'static str>,
    /// File to check for diffs (when non-empty, git diff is validated).
    pub file_path: &'static str,
    pub expected_diff_contains: Vec<&'static str>,
}

impl Default for GroundTruth {
    fn default() -> Self {
        Self {
            required_strings: Vec::new(),
            forbidden_strings: vec!["I cannot", "I don't have access", "no such file"],
            file_path: "",
            expected_diff_contains: Vec::new(),
        }
    }
}

impl GroundTruth {
    pub fn new(required: Vec<&'static str>) -> Self {
        Self {
            required_strings: required,
            ..Self::default()
        }
    }

    pub fn with_edit(
        required: Vec<&'static str>,
        file_path: &'static str,
        diff_contains: Vec<&'static str>,
    ) -> Self {
        Self {
            required_strings: required,
            file_path,
            expected_diff_contains: diff_contains,
            ..Self::default()
        }
    }
}

pub trait Task: Sync {
    #[expect(dead_code)]
    fn name(&self) -> &'static str;
    fn prompt(&self) -> &'static str;
    fn ground_truth(&self) -> GroundTruth;

    #[expect(dead_code)]
    fn task_type(&self) -> &'static str {
        "read"
    }

    fn repo(&self) -> &'static str {
        "synthetic"
    }

    /// Override to provide an explicit working directory (e.g. for fixture-based eval tasks).
    /// When `Some`, `run_single` uses this path directly instead of looking up the repo.
    fn work_dir(&self) -> Option<PathBuf> {
        None
    }

    /// Validate result against ground truth.
    ///
    /// For navigate tasks: checks that all `required_strings` appear in the
    /// concatenated assistant text across all turns.
    ///
    /// For edit tasks (non-empty `file_path`): checks git diff for expected
    /// patterns. `required_strings` are checked against *both* the assistant
    /// text and the diff output — a match in either counts.
    fn check_correctness(&self, result_text: &str, repo_path: &Path) -> (bool, String) {
        let gt = self.ground_truth();
        let text_lower = result_text.to_lowercase();

        for forbidden in &gt.forbidden_strings {
            if text_lower.contains(&forbidden.to_lowercase()) {
                return (false, format!("Contains forbidden: {forbidden}"));
            }
        }

        // For edit tasks, get the diff first — required_strings can match
        // against either assistant text or the diff.
        let diff_text = if !gt.file_path.is_empty() {
            let output = Command::new("git")
                .args(["diff", gt.file_path])
                .current_dir(repo_path)
                .output();

            match output {
                Ok(o) => {
                    let diff = String::from_utf8_lossy(&o.stdout).to_string();
                    if diff.is_empty() {
                        return (false, "No changes in target file".into());
                    }
                    for pattern in &gt.expected_diff_contains {
                        if !diff.contains(pattern) {
                            return (false, format!("Diff missing: {pattern}"));
                        }
                    }
                    Some(diff)
                }
                Err(e) => return (false, format!("git diff failed: {e}")),
            }
        } else {
            None
        };

        // Check required strings against assistant text + diff (if available).
        let search_text = if let Some(ref diff) = diff_text {
            format!("{text_lower}\n{}", diff.to_lowercase())
        } else {
            text_lower
        };

        for required in &gt.required_strings {
            if !search_text.contains(&required.to_lowercase()) {
                return (false, format!("Missing: {required}"));
            }
        }

        (true, "All checks passed".into())
    }
}
