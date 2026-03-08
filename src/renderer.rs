use gpui::*;
use std::collections::HashMap;
use std::sync::Arc;

use crate::node::Node;

pub struct NodeRenderContext {
    pub zoom: f32,
}

pub trait NodeRenderer: Send + Sync {
    /// node's world size
    fn size(&self, node: &Node) -> Size<Pixels>;

    /// render node inner UI
    fn render(&self, node: &Node, cx: &mut NodeRenderContext) -> AnyElement;
}

#[derive(Clone)]
pub struct RendererRegistry {
    map: HashMap<String, Arc<dyn NodeRenderer>>,
}

impl RendererRegistry {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn register<R>(&mut self, name: impl Into<String>, renderer: R)
    where
        R: NodeRenderer + 'static,
    {
        self.map.insert(name.into(), Arc::new(renderer));
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn NodeRenderer>> {
        self.map.get(name).cloned()
    }
}
