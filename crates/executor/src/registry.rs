use std::collections::HashMap;

use crate::context::NodeProcessor;

pub struct NodeRegistry {
    handlers: HashMap<String, Box<dyn NodeProcessor>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register(&mut self, handler: impl NodeProcessor + 'static) {
        self.handlers
            .insert(handler.name().to_string(), Box::new(handler));
    }

    pub fn get(&self, node_type: &str) -> Option<&dyn NodeProcessor> {
        self.handlers.get(node_type).map(|h| h.as_ref())
    }
}
