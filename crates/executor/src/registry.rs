use std::collections::HashMap;

use crate::context::NodeHandler;

pub struct NodeRegistry {
    handlers: HashMap<String, Box<dyn NodeHandler>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register(&mut self, handler: impl NodeHandler + 'static) {
        self.handlers
            .insert(handler.name().to_string(), Box::new(handler));
    }

    pub fn get(&self, node_type: &str) -> Option<&dyn NodeHandler> {
        self.handlers.get(node_type).map(|h| h.as_ref())
    }
}
