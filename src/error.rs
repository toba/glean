use std::path::{Path, PathBuf};

/// Every error glean can produce. Displayed as user-facing messages with suggestions.
#[derive(Debug)]
pub enum GleanError {
    NotFound {
        path: PathBuf,
        suggestion: Option<String>,
    },
    PermissionDenied {
        path: PathBuf,
    },
    InvalidQuery {
        query: String,
        reason: String,
    },
    IoError {
        path: PathBuf,
        source: std::io::Error,
    },
    ParseError {
        path: PathBuf,
        reason: String,
    },
}

impl std::fmt::Display for GleanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { path, suggestion } => {
                write!(f, "not found: {}", path.display())?;
                if let Some(s) = suggestion {
                    write!(f, " — did you mean: {s}")?;
                }
                Ok(())
            }
            Self::PermissionDenied { path } => {
                write!(f, "{} [permission denied]", path.display())
            }
            Self::InvalidQuery { query, reason } => {
                write!(f, "invalid query \"{query}\": {reason}")
            }
            Self::IoError { path, source } => {
                write!(f, "{}: {source}", path.display())
            }
            Self::ParseError { path, reason } => {
                write!(f, "parse error in {}: {reason}", path.display())
            }
        }
    }
}

impl std::error::Error for GleanError {}

/// Returns a closure suitable for `.map_err(io_err(path))`.
pub fn io_err(path: &Path) -> impl FnOnce(std::io::Error) -> GleanError {
    let path = path.to_path_buf();
    move |source| GleanError::IoError { path, source }
}

impl GleanError {
    /// Exit code matching the spec.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::NotFound { .. } | Self::IoError { .. } => 2,
            Self::InvalidQuery { .. } | Self::ParseError { .. } => 3,
            Self::PermissionDenied { .. } => 4,
        }
    }
}
