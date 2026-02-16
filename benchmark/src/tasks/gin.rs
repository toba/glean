use crate::task::{GroundTruth, Task};

pub struct RadixTree;
impl Task for RadixTree {
    fn name(&self) -> &'static str {
        "gin_radix_tree"
    }
    fn repo(&self) -> &'static str {
        "gin"
    }
    fn prompt(&self) -> &'static str {
        "Find the radix tree implementation that gin uses for routing. Show the node \
         struct definition, the nodeType constants, and explain how wildcard parameters \
         (like :id and *filepath) are handled in the tree. What method looks up a route?"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "type node struct",
            "catchAll",
            "getValue",
            "wildChild",
        ])
    }
}

pub struct ClientIP;
impl Task for ClientIP {
    fn name(&self) -> &'static str {
        "gin_client_ip"
    }
    fn repo(&self) -> &'static str {
        "gin"
    }
    fn prompt(&self) -> &'static str {
        "In gin's Context, show the complete implementation of the ClientIP() method. \
         What headers does it check, in what order? How does it handle trusted proxies \
         and the X-Forwarded-For header? Trace into the Engine to show how trustedCIDRs \
         is used."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "ClientIP",
            "RemoteIPHeaders",
            "X-Forwarded-For",
            "trustedCIDRs",
        ])
    }
}

pub struct MiddlewareChain;
impl Task for MiddlewareChain {
    fn name(&self) -> &'static str {
        "gin_middleware_chain"
    }
    fn repo(&self) -> &'static str {
        "gin"
    }
    fn prompt(&self) -> &'static str {
        "Trace how a request flows through gin's middleware chain. Start from \
         Engine.ServeHTTP, show how it gets a Context from the pool, finds the route \
         handlers, and executes the handler chain. What is HandlersChain and how does \
         Context.Next() advance through it?"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec!["ServeHTTP", "HandlersChain", "Next", "pool", "index"])
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
}

pub struct ContextNext;
impl Task for ContextNext {
    fn name(&self) -> &'static str {
        "gin_context_next"
    }
    fn repo(&self) -> &'static str {
        "gin"
    }
    fn prompt(&self) -> &'static str {
        "Find the Context.Next() method in gin. Show its complete implementation."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec!["Next", "context.go", "index"])
    }
}

pub struct ServeHTTPFlow;
impl Task for ServeHTTPFlow {
    fn name(&self) -> &'static str {
        "gin_servehttp_flow"
    }
    fn repo(&self) -> &'static str {
        "gin"
    }
    fn prompt(&self) -> &'static str {
        "Find Engine.ServeHTTP in gin and show its implementation. Then \
         trace what functions it calls — how does it get a Context, find \
         the matching route handlers, and start executing them?"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec!["ServeHTTP", "gin.go", "handleHTTPRequest", "pool"])
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
}

pub struct BindingTag;
impl Task for BindingTag {
    fn name(&self) -> &'static str {
        "gin_binding_tag"
    }
    fn repo(&self) -> &'static str {
        "gin"
    }
    fn prompt(&self) -> &'static str {
        "Gin uses a struct tag name for binding validation. Find where this tag name \
         is configured in the validator and change the tag name from \"binding\" to \
         \"validate\"."
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::with_edit(
            vec!["validate", "SetTagName"],
            "binding/default_validator.go",
            vec!["validate"],
        )
    }
}
