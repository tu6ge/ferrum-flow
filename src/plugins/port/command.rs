use crate::{Edge, GraphOp, canvas::Command};

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
    fn execute(&mut self, ctx: &mut crate::canvas::CommandContext) {
        ctx.add_edge(self.edge.clone());
    }
    fn undo(&mut self, ctx: &mut crate::canvas::CommandContext) {
        ctx.remove_edge(self.edge.id);
    }

    fn to_ops(&self, _ctx: &mut crate::CommandContext) -> Vec<crate::GraphOp> {
        vec![GraphOp::AddEdge(self.edge.clone())]
    }
}
