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

pub struct AcceptableStatus;
impl Task for AcceptableStatus {
    fn name(&self) -> &'static str {
        "af_acceptable_status"
    }
    fn repo(&self) -> &'static str {
        "alamofire"
    }
    fn prompt(&self) -> &'static str {
        "When Alamofire validates a response, it checks the HTTP status code against \
         an acceptable range. Trace how validate() on a DataRequest determines which \
         status codes are acceptable, and change the default acceptable range to start \
         from 100 instead of 200 (i.e., accept 100..<300 instead of 200..<300)."
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::with_edit(
            vec!["100", "acceptableStatusCodes", "Validation"],
            "Source/Features/Validation.swift",
            vec!["100"],
        )
    }
}
