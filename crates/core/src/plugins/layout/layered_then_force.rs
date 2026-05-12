//! Two-stage layout: [`LayeredDagLayout`] (initializer) then [`ForceDirectedLayout`] (optimizer),
//! threading [`PositionHint`] between them.

use crate::Graph;

use super::force_directed::ForceDirectedLayout;
use super::layered_dag::LayeredDagLayout;
use super::strategy::{
    LayoutError, LayoutOptions, LayoutOutput, LayoutPhase, LayoutStrategy, PositionHint,
};

/// Runs layered placement, then force-directed refinement using the layered result as warm start.
#[derive(Debug, Clone)]
pub struct LayeredThenForceLayout {
    pub layered: LayeredDagLayout,
    pub force: ForceDirectedLayout,
}

impl Default for LayeredThenForceLayout {
    fn default() -> Self {
        Self {
            layered: LayeredDagLayout,
            force: ForceDirectedLayout::default(),
        }
    }
}

impl LayoutStrategy for LayeredThenForceLayout {
    fn id(&self) -> &'static str {
        "layered_then_force"
    }

    fn label(&self) -> &'static str {
        "Layered → force"
    }

    fn phase(&self) -> LayoutPhase {
        LayoutPhase::Optimizer
    }

    fn compute(
        &self,
        graph: &Graph,
        options: &LayoutOptions,
        hint: Option<&PositionHint>,
    ) -> Result<LayoutOutput, LayoutError> {
        let first = self.layered.compute(graph, options, hint)?;
        let warm = match &first {
            LayoutOutput::Unchanged => PositionHint::from_graph(graph),
            LayoutOutput::Delta(d) => PositionHint::from_delta_to(d),
        };
        self.force.compute(graph, options, Some(&warm))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Graph;
    use serde_json::json;

    #[test]
    fn pipeline_runs_without_error_on_small_dag() {
        let graph = Graph::build(|g| {
            let (_, _, o_a) = g
                .create_node("")
                .position(0.0, 0.0)
                .output()
                .data(json!({}))
                .build_with_ports();
            let (_, i_b, _) = g
                .create_node("")
                .position(200.0, 200.0)
                .input()
                .data(json!({}))
                .build_with_ports();
            g.create_edge().source(o_a[0]).target(i_b[0]).build();
        });

        let s = LayeredThenForceLayout::default();
        let out = s
            .compute(&graph, &LayoutOptions::default(), None)
            .expect("compute");
        let LayoutOutput::Delta(d) = out else {
            panic!("expected delta");
        };
        assert!(d.has_changes());
    }
}
