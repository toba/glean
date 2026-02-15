mod alamofire;
mod gin;
mod ripgrep;
mod zod;

use crate::task::Task;
use std::collections::HashMap;

/// Build the full task registry.
pub fn all_tasks() -> HashMap<&'static str, Box<dyn Task>> {
    let mut m: HashMap<&'static str, Box<dyn Task>> = HashMap::new();

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
    m.insert("rg_edit_buffer_capacity", Box::new(ripgrep::EditBufferCapacity));

    // gin (Go)
    m.insert("gin_radix_tree", Box::new(gin::RadixTree));
    m.insert("gin_client_ip", Box::new(gin::ClientIP));
    m.insert("gin_middleware_chain", Box::new(gin::MiddlewareChain));
    m.insert("gin_context_next", Box::new(gin::ContextNext));
    m.insert("gin_servehttp_flow", Box::new(gin::ServeHTTPFlow));
    m.insert(
        "gin_edit_multipart_memory",
        Box::new(gin::EditMultipartMemory),
    );

    // alamofire (Swift)
    m.insert("af_session_config", Box::new(alamofire::SessionConfig));
    m.insert("af_request_chain", Box::new(alamofire::RequestChain));
    m.insert(
        "af_response_validation",
        Box::new(alamofire::ResponseValidation),
    );
    m.insert(
        "af_interceptor_protocol",
        Box::new(alamofire::InterceptorProtocol),
    );
    m.insert(
        "af_upload_multipart",
        Box::new(alamofire::UploadMultipart),
    );
    m.insert(
        "af_edit_encoding_threshold",
        Box::new(alamofire::EditEncodingThreshold),
    );

    // zod (TypeScript)
    m.insert("zod_string_schema", Box::new(zod::StringSchema));
    m.insert("zod_parse_flow", Box::new(zod::ParseFlow));
    m.insert("zod_error_handling", Box::new(zod::ErrorHandling));
    m.insert(
        "zod_discriminated_union",
        Box::new(zod::DiscriminatedUnion),
    );
    m.insert("zod_transform_pipe", Box::new(zod::TransformPipe));
    m.insert("zod_optional_nullable", Box::new(zod::OptionalNullable));

    m
}
