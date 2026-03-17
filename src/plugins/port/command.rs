use crate::{Edge, canvas::Command};

pub(super) struct CreateEdge {
    edge: Edge,
}

impl CreateEdge {
    pub(super) fn new(edge: Edge) -> Self {
        Self { edge }
    }
}

impl Command for CreateEdge {
    fn name(&self) -> &'static str {
        "create_edge"
    }
    fn execute(&mut self, ctx: &mut crate::canvas::CanvasState) {
        ctx.add_edge(self.edge.clone());
    }
    fn undo(&mut self, ctx: &mut crate::canvas::CanvasState) {
        ctx.remove_edge(self.edge.id);
    }
}
