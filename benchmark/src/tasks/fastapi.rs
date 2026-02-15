use crate::task::{GroundTruth, Task};

pub struct DependencyResolution;
impl Task for DependencyResolution {
    fn name(&self) -> &'static str {
        "fastapi_dependency_resolution"
    }
    fn repo(&self) -> &'static str {
        "fastapi"
    }
    fn prompt(&self) -> &'static str {
        "In FastAPI, trace how dependency injection works. Start from how Depends() is \
         defined, then find how dependencies are resolved at request time. Show the key \
         functions involved in the dependency resolution chain."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "get_dependant",
            "solve_dependencies",
            "Dependant",
            "analyze_param",
        ])
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
}

pub struct RequestValidation;
impl Task for RequestValidation {
    fn name(&self) -> &'static str {
        "fastapi_request_validation"
    }
    fn repo(&self) -> &'static str {
        "fastapi"
    }
    fn prompt(&self) -> &'static str {
        "How does FastAPI validate request parameters? Trace the validation chain from \
         when a request comes in to where Pydantic validation happens. Show the key \
         functions and explain how path parameters, query parameters, and body are validated."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "get_request_handler",
            "solve_dependencies",
            "RequestValidationError",
            "async def app",
        ])
    }
}

pub struct DependsInternals;
impl Task for DependsInternals {
    fn name(&self) -> &'static str {
        "fastapi_depends_internals"
    }
    fn repo(&self) -> &'static str {
        "fastapi"
    }
    fn prompt(&self) -> &'static str {
        "In fastapi/param_functions.py, find the Depends function. Show its complete \
         signature including all parameters. Then explain: is Depends a class or a function? \
         What does it actually return, and where is the underlying implementation?"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec!["def Depends", "use_cache", "params.Depends"])
    }
}

pub struct DependsFunction;
impl Task for DependsFunction {
    fn name(&self) -> &'static str {
        "fastapi_depends_function"
    }
    fn repo(&self) -> &'static str {
        "fastapi"
    }
    fn prompt(&self) -> &'static str {
        "Find the Depends function in FastAPI. Show its complete signature and docstring."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec!["def Depends", "param_functions.py", "use_cache"])
    }
}

pub struct DependsProcessing;
impl Task for DependsProcessing {
    fn name(&self) -> &'static str {
        "fastapi_depends_processing"
    }
    fn repo(&self) -> &'static str {
        "fastapi"
    }
    fn prompt(&self) -> &'static str {
        "Find the Depends function in fastapi/param_functions.py and show \
         its implementation. Then find how FastAPI processes dependencies \
         — look for the function that walks the dependency tree and resolves \
         each Depends() call at request time."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "def Depends",
            "solve_dependencies",
            "Dependant",
            "get_dependant",
        ])
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
}
