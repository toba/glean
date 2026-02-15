use std::path::Path;
use std::process::Command;

/// Expected elements for correctness validation.
#[derive(Clone)]
pub struct GroundTruth {
    pub required_strings: Vec<&'static str>,
    pub forbidden_strings: Vec<&'static str>,
    /// For edit tasks only.
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

    fn task_type(&self) -> &'static str {
        "read"
    }

    fn repo(&self) -> &'static str {
        "synthetic"
    }

    /// Validate result against ground truth.
    fn check_correctness(&self, result_text: &str, repo_path: &Path) -> (bool, String) {
        let gt = self.ground_truth();
        let text_lower = result_text.to_lowercase();

        for required in &gt.required_strings {
            if !text_lower.contains(&required.to_lowercase()) {
                return (false, format!("Missing: {required}"));
            }
        }

        for forbidden in &gt.forbidden_strings {
            if text_lower.contains(&forbidden.to_lowercase()) {
                return (false, format!("Contains forbidden: {forbidden}"));
            }
        }

        if self.task_type() == "edit" && !gt.file_path.is_empty() {
            let output = Command::new("git")
                .args(["diff", gt.file_path])
                .current_dir(repo_path)
                .output();

            match output {
                Ok(o) => {
                    let diff = String::from_utf8_lossy(&o.stdout);
                    if diff.is_empty() {
                        return (false, "No changes in target file".into());
                    }
                    for pattern in &gt.expected_diff_contains {
                        if !diff.contains(pattern) {
                            return (false, format!("Diff missing: {pattern}"));
                        }
                    }
                }
                Err(e) => return (false, format!("git diff failed: {e}")),
            }
        }

        (true, "All checks passed".into())
    }
}
