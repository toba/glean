use crate::task::{GroundTruth, Task};

pub struct StringSchema;
impl Task for StringSchema {
    fn name(&self) -> &'static str {
        "zod_string_schema"
    }
    fn repo(&self) -> &'static str {
        "zod"
    }
    fn prompt(&self) -> &'static str {
        "Find the ZodString implementation in Zod v4. Show how it's defined, what \
         validation methods it has (like min, max, regex), and how the constructor \
         initializes it. Look in the packages/zod/src/v4/ directory."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "ZodString",
            "minLength",
            "maxLength",
            "regex",
            "schemas.ts",
        ])
    }
}

pub struct ParseFlow;
impl Task for ParseFlow {
    fn name(&self) -> &'static str {
        "zod_parse_flow"
    }
    fn repo(&self) -> &'static str {
        "zod"
    }
    fn prompt(&self) -> &'static str {
        "Trace how parsing works in Zod v4. Start from the parse function in \
         packages/zod/src/v4/core/parse.ts. Show how it calls schema._zod.run(), \
         handles issues, and throws ZodError. Also show safeParse."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "parse.ts",
            "_zod.run",
            "issues",
            "safeParse",
        ])
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
}

pub struct ErrorHandling;
impl Task for ErrorHandling {
    fn name(&self) -> &'static str {
        "zod_error_handling"
    }
    fn repo(&self) -> &'static str {
        "zod"
    }
    fn prompt(&self) -> &'static str {
        "Show the error types in Zod v4. Find the ZodIssue types and ZodError class \
         in packages/zod/src/v4/core/errors.ts. What are the different issue types \
         (invalid type, too big, too small, etc.)?"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "errors.ts",
            "$ZodIssueInvalidType",
            "$ZodIssueTooBig",
            "$ZodIssueTooSmall",
        ])
    }
}

pub struct DiscriminatedUnion;
impl Task for DiscriminatedUnion {
    fn name(&self) -> &'static str {
        "zod_discriminated_union"
    }
    fn repo(&self) -> &'static str {
        "zod"
    }
    fn prompt(&self) -> &'static str {
        "Find the discriminatedUnion implementation in Zod v4. Show the function \
         signature and how it differs from a regular union. Look in \
         packages/zod/src/v4/classic/schemas.ts."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "discriminatedUnion",
            "discriminator",
            "schemas.ts",
        ])
    }
}

pub struct TransformPipe;
impl Task for TransformPipe {
    fn name(&self) -> &'static str {
        "zod_transform_pipe"
    }
    fn repo(&self) -> &'static str {
        "zod"
    }
    fn prompt(&self) -> &'static str {
        "Show how transform() and pipe() work in Zod v4. Find their implementations \
         in packages/zod/src/v4/classic/schemas.ts. How does transform create a pipe \
         internally? What is ZodPipe?"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "transform",
            "ZodPipe",
            "pipe",
            "schemas.ts",
        ])
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
}

pub struct ErrorFallback;
impl Task for ErrorFallback {
    fn name(&self) -> &'static str {
        "zod_error_fallback"
    }
    fn repo(&self) -> &'static str {
        "zod"
    }
    fn prompt(&self) -> &'static str {
        "When a Zod schema validation fails and no custom error message is provided, \
         Zod falls back to a default message. Trace the error message resolution chain \
         — from where validation issues are created in a schema's parse function, through \
         to where the final fallback message is determined — and change the fallback \
         message from \"Invalid input\" to \"Validation failed\"."
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::with_edit(
            vec!["Validation failed", "finalizeIssue"],
            "packages/zod/src/v4/core/util.ts",
            vec!["Validation failed"],
        )
    }
}

pub struct OptionalNullable;
impl Task for OptionalNullable {
    fn name(&self) -> &'static str {
        "zod_optional_nullable"
    }
    fn repo(&self) -> &'static str {
        "zod"
    }
    fn prompt(&self) -> &'static str {
        "Show how .optional() and .nullable() work in Zod v4. Find the ZodOptional \
         and ZodNullable types in packages/zod/src/v4/classic/schemas.ts. How does \
         nullish() combine them?"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "ZodOptional",
            "ZodNullable",
            "nullish",
            "schemas.ts",
        ])
    }
}
