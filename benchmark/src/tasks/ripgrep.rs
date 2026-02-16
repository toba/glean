use crate::task::{GroundTruth, Task};

pub struct TraitImplementors;
impl Task for TraitImplementors {
    fn name(&self) -> &'static str {
        "rg_trait_implementors"
    }
    fn repo(&self) -> &'static str {
        "ripgrep"
    }
    fn prompt(&self) -> &'static str {
        "Find the `Matcher` trait definition in the matcher crate. \
         Then find all types that implement this trait. For each implementor, \
         show where it is defined and what crate it lives in."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec!["Matcher", "find_at", "RegexMatcher"])
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
}

pub struct FlagDefinition;
impl Task for FlagDefinition {
    fn name(&self) -> &'static str {
        "rg_flag_definition"
    }
    fn repo(&self) -> &'static str {
        "ripgrep"
    }
    fn prompt(&self) -> &'static str {
        "In crates/core/flags/defs.rs, find the implementation of the --type-list flag. \
         Show the complete Flag trait implementation for this flag, including all methods."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec!["TypeList", "impl Flag for", "name_long", "type-list"])
    }
}

pub struct SearchDispatch;
impl Task for SearchDispatch {
    fn name(&self) -> &'static str {
        "rg_search_dispatch"
    }
    fn repo(&self) -> &'static str {
        "ripgrep"
    }
    fn prompt(&self) -> &'static str {
        "Explain how ripgrep dispatches between single-line and multi-line search. \
         Trace the code path from the Searcher to the actual matching logic. \
         What structs are involved and how do the generic type parameters flow?"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec!["ReadByLine", "MultiLine", "Sink", "glue.rs"])
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
}

pub struct WalkerParallel;
impl Task for WalkerParallel {
    fn name(&self) -> &'static str {
        "rg_walker_parallel"
    }
    fn repo(&self) -> &'static str {
        "ripgrep"
    }
    fn prompt(&self) -> &'static str {
        "In the ignore crate, find the parallel directory walker. Show the WalkParallel \
         struct, the ParallelVisitor trait, and explain how work is distributed across threads."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "WalkParallel",
            "ParallelVisitor",
            "ParallelVisitorBuilder",
        ])
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
}

pub struct LineIterDefinition;
impl Task for LineIterDefinition {
    fn name(&self) -> &'static str {
        "rg_lineiter_definition"
    }
    fn repo(&self) -> &'static str {
        "ripgrep"
    }
    fn prompt(&self) -> &'static str {
        "Find the LineIter struct definition in the ripgrep codebase. \
         Show the struct and its fields, and explain what it's used for."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec!["LineIter", "stepper", "bytes"])
    }
}

pub struct LineIterUsage;
impl Task for LineIterUsage {
    fn name(&self) -> &'static str {
        "rg_lineiter_usage"
    }
    fn repo(&self) -> &'static str {
        "ripgrep"
    }
    fn prompt(&self) -> &'static str {
        "Find the LineIter struct in ripgrep's searcher crate. Show its \
         definition, then find where LineIter is constructed (look for \
         LineIter::new or LineIter { calls). Show 2-3 key call sites."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec!["LineIter", "LineStep", "new"])
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
}

pub struct BinaryDetectionDefault;
impl Task for BinaryDetectionDefault {
    fn name(&self) -> &'static str {
        "rg_binary_detection_default"
    }
    fn repo(&self) -> &'static str {
        "ripgrep"
    }
    fn prompt(&self) -> &'static str {
        "When ripgrep encounters a binary file during implicit search (e.g., recursive \
         directory search), it quits searching that file by default. Trace how this \
         default binary detection behavior is configured — from the CLI flag processing \
         through to where the searcher is built — and change the default implicit binary \
         detection from `quit` to `convert` (so binary files are searched but with null \
         bytes replaced instead of being skipped)."
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::with_edit(
            vec!["convert", "from_low_args"],
            "crates/core/flags/hiargs.rs",
            vec!["convert"],
        )
    }
}
