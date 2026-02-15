mod express;
mod fastapi;
mod gin;
mod ripgrep;
mod synthetic;

use crate::task::Task;
use std::collections::HashMap;

/// Build the full task registry.
pub fn all_tasks() -> HashMap<&'static str, Box<dyn Task>> {
    let mut m: HashMap<&'static str, Box<dyn Task>> = HashMap::new();

    // Synthetic repo tasks
    m.insert("find_definition", Box::new(synthetic::FindDefinition));
    m.insert("read_large_file", Box::new(synthetic::ReadLargeFile));
    m.insert("edit_task", Box::new(synthetic::EditTask));
    m.insert(
        "codebase_navigation",
        Box::new(synthetic::CodebaseNavigation),
    );
    m.insert("markdown_section", Box::new(synthetic::MarkdownSection));

    // ripgrep (Rust)
    m.insert(
        "rg_trait_implementors",
        Box::new(ripgrep::TraitImplementors),
    );
    m.insert("rg_flag_definition", Box::new(ripgrep::FlagDefinition));
    m.insert("rg_search_dispatch", Box::new(ripgrep::SearchDispatch));
    m.insert("rg_walker_parallel", Box::new(ripgrep::WalkerParallel));
    m.insert(
        "rg_lineiter_definition",
        Box::new(ripgrep::LineIterDefinition),
    );
    m.insert("rg_lineiter_usage", Box::new(ripgrep::LineIterUsage));

    // fastapi (Python)
    m.insert(
        "fastapi_dependency_resolution",
        Box::new(fastapi::DependencyResolution),
    );
    m.insert(
        "fastapi_request_validation",
        Box::new(fastapi::RequestValidation),
    );
    m.insert(
        "fastapi_depends_internals",
        Box::new(fastapi::DependsInternals),
    );
    m.insert(
        "fastapi_depends_function",
        Box::new(fastapi::DependsFunction),
    );
    m.insert(
        "fastapi_depends_processing",
        Box::new(fastapi::DependsProcessing),
    );

    // gin (Go)
    m.insert("gin_radix_tree", Box::new(gin::RadixTree));
    m.insert("gin_client_ip", Box::new(gin::ClientIP));
    m.insert("gin_middleware_chain", Box::new(gin::MiddlewareChain));
    m.insert("gin_context_next", Box::new(gin::ContextNext));
    m.insert("gin_servehttp_flow", Box::new(gin::ServeHTTPFlow));

    // express (JavaScript)
    m.insert("express_json_send", Box::new(express::JsonSend));
    m.insert("express_render_chain", Box::new(express::RenderChain));
    m.insert("express_app_init", Box::new(express::AppInit));
    m.insert("express_res_send", Box::new(express::ResSend));
    m.insert("express_app_render", Box::new(express::AppRender));

    m
}
