use crate::task::{GroundTruth, Task};

pub struct FindDefinition;
impl Task for FindDefinition {
    fn name(&self) -> &'static str {
        "find_definition"
    }
    fn prompt(&self) -> &'static str {
        "Find where `validate_jwt_token` is defined. Show the full implementation."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec!["tokens.py", "def validate_jwt_token", "jwt.decode"])
    }
}

pub struct ReadLargeFile;
impl Task for ReadLargeFile {
    fn name(&self) -> &'static str {
        "read_large_file"
    }
    fn prompt(&self) -> &'static str {
        "Show me the rate limiting logic in src/api/routes.py"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec!["def rate_limit", "requests_per_minute", "@wraps"])
    }
}

pub struct EditTask;
impl Task for EditTask {
    fn name(&self) -> &'static str {
        "edit_task"
    }
    fn prompt(&self) -> &'static str {
        "In src/database/connection.py, change the return type annotation of the \
         `get_pool` function to `Optional[ConnectionPool]`. Add the necessary import for Optional."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::with_edit(
            vec!["Optional"],
            "src/database/connection.py",
            vec!["Optional[ConnectionPool]", "Optional"],
        )
    }
    fn task_type(&self) -> &'static str {
        "edit"
    }
}

pub struct CodebaseNavigation;
impl Task for CodebaseNavigation {
    fn name(&self) -> &'static str {
        "codebase_navigation"
    }
    fn prompt(&self) -> &'static str {
        "What files in this codebase handle database operations? \
         List each file with a one-line description of what it does."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec!["connection.py", "queries.py", "migrations.py"])
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
}

pub struct MarkdownSection;
impl Task for MarkdownSection {
    fn name(&self) -> &'static str {
        "markdown_section"
    }
    fn prompt(&self) -> &'static str {
        "Read the Deployment section from README.md. What environment variables are required?"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec!["DATABASE_URL", "SECRET_KEY", "REDIS_URL"])
    }
}
