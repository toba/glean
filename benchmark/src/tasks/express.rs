use crate::task::{GroundTruth, Task};

pub struct JsonSend;
impl Task for JsonSend {
    fn name(&self) -> &'static str {
        "express_json_send"
    }
    fn repo(&self) -> &'static str {
        "express"
    }
    fn prompt(&self) -> &'static str {
        "In Express, trace how res.json() works internally. Show the implementation \
         of both res.json and res.send. How does res.send handle different content types \
         (strings, buffers, objects)? What application settings does res.json read?"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "res.json",
            "json replacer",
            "json spaces",
            "res.send",
            "Content-Type",
        ])
    }
}

pub struct RenderChain;
impl Task for RenderChain {
    fn name(&self) -> &'static str {
        "express_render_chain"
    }
    fn repo(&self) -> &'static str {
        "express"
    }
    fn prompt(&self) -> &'static str {
        "Trace the full template rendering chain in Express. When res.render() is called, \
         how does it reach the template engine? Show the code path through res.render, \
         app.render, and the View class. How does View resolve the template file path?"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "res.render",
            "app.render",
            "View",
            "lookup",
            "view.js",
        ])
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
}

pub struct AppInit;
impl Task for AppInit {
    fn name(&self) -> &'static str {
        "express_app_init"
    }
    fn repo(&self) -> &'static str {
        "express"
    }
    fn prompt(&self) -> &'static str {
        "How does Express create and initialize an application? Show what happens when \
         you call express(). Find the createApplication function and trace through \
         app.init. What default settings are configured? Where is the router created?"
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec![
            "createApplication",
            "express.js",
            "application.js",
            "trust proxy",
            "etag",
        ])
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
}

pub struct ResSend;
impl Task for ResSend {
    fn name(&self) -> &'static str {
        "express_res_send"
    }
    fn repo(&self) -> &'static str {
        "express"
    }
    fn prompt(&self) -> &'static str {
        "Find the res.send function implementation in Express. Show its complete code."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec!["res.send", "response", "Content-Length"])
    }
}

pub struct AppRender;
impl Task for AppRender {
    fn name(&self) -> &'static str {
        "express_app_render"
    }
    fn repo(&self) -> &'static str {
        "express"
    }
    fn prompt(&self) -> &'static str {
        "Find app.render in Express's application.js. Show its \
         implementation, then find the View class it uses and explain \
         how View resolves template file paths."
    }
    fn ground_truth(&self) -> GroundTruth {
        GroundTruth::new(vec!["app.render", "View", "application.js", "view.js"])
    }
    fn task_type(&self) -> &'static str {
        "navigate"
    }
}
