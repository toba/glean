use crate::task::{GroundTruth, Task};

pub struct SessionConfig;
impl Task for SessionConfig {
    fn name(&self) -> &'static str {
        "af_session_config"
    }
    fn repo(&self) -> &'static str {
        "alamofire"
    }
    fn prompt(&self) -> &'static str {
        "Find the Session class in Alamofire. Show its class definition, key properties \
         (like the underlying URLSession, delegate, queues), and the default initializer. \
         What is the default URLSessionConfiguration used?"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "class Session",
            "Session.swift",
            "URLSession",
            "rootQueue",
            "SessionDelegate",
        ])
    }
}

pub struct RequestChain;
impl Task for RequestChain {
    fn name(&self) -> &'static str {
        "af_request_chain"
    }
    fn repo(&self) -> &'static str {
        "alamofire"
    }
    fn prompt(&self) -> &'static str {
        "Trace how a request is created and dispatched in Alamofire. Start from \
         Session.request(), show how it creates a DataRequest, how the URLRequest \
         is built, how adapters process it, and how the URLSessionTask is finally created."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "DataRequest",
            "Session.swift",
            "didCreateURLRequest",
            "adapter",
        ])
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
}

pub struct ResponseValidation;
impl Task for ResponseValidation {
    fn name(&self) -> &'static str {
        "af_response_validation"
    }
    fn repo(&self) -> &'static str {
        "alamofire"
    }
    fn prompt(&self) -> &'static str {
        "Show the response validation implementation in Alamofire. Find the Validation \
         typealias, the acceptable status code range, and how validate() works on \
         DataRequest. What file contains the validation logic?"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "Validation.swift",
            "acceptableStatusCodes",
            "200",
        ])
    }
}

pub struct InterceptorProtocol;
impl Task for InterceptorProtocol {
    fn name(&self) -> &'static str {
        "af_interceptor_protocol"
    }
    fn repo(&self) -> &'static str {
        "alamofire"
    }
    fn prompt(&self) -> &'static str {
        "Find the RequestInterceptor protocol in Alamofire. Show the RequestAdapter \
         and RequestRetrier protocols, the RetryResult enum, and how the Interceptor \
         class combines adapters and retriers."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "RequestInterceptor",
            "RequestAdapter",
            "RequestRetrier",
            "RetryResult",
            "RequestInterceptor.swift",
        ])
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
}

pub struct UploadMultipart;
impl Task for UploadMultipart {
    fn name(&self) -> &'static str {
        "af_upload_multipart"
    }
    fn repo(&self) -> &'static str {
        "alamofire"
    }
    fn prompt(&self) -> &'static str {
        "Show the MultipartFormData class in Alamofire. What is the encoding memory \
         threshold? How are body parts structured? Show the append methods for adding \
         data, files, and streams."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "MultipartFormData",
            "encodingMemoryThreshold",
            "BodyPart",
            "append",
        ])
    }
}

pub struct EditEncodingThreshold;
impl Task for EditEncodingThreshold {
    fn name(&self) -> &'static str {
        "af_edit_encoding_threshold"
    }
    fn repo(&self) -> &'static str {
        "alamofire"
    }
    fn prompt(&self) -> &'static str {
        "In Alamofire's MultipartFormData.swift, change the encodingMemoryThreshold \
         from 10_000_000 (10 MB) to 20_000_000 (20 MB)."
    }
    fn task_type(&self) -> &'static str {
        "edit"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::with_edit(
            vec!["20_000_000"],
            "Source/Features/MultipartFormData.swift",
            vec!["20_000_000"],
        )
    }
}
