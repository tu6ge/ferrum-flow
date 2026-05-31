//! Hierarchy scope helpers for [`super::GraphPlugin`].

use crate::{Graph, NodeId};

/// Paint-scope queries for nested graphs ([`super::GraphPlugin`] only; not used with
/// [`crate::plugins::NodePlugin`] / [`crate::plugins::EdgePlugin`] on the same canvas).
pub(super) trait GraphHierarchy {
    /// `true` when any node has a parent or children.
    fn has_node_hierarchy(&self) -> bool;

    /// Top-level group anchor in [`Graph::paint_order`] (has children, not under another group).
    fn top_level_group_anchors_in_paint_order(&self) -> Vec<NodeId>;
}

impl GraphHierarchy for Graph {
    fn has_node_hierarchy(&self) -> bool {
        self.nodes()
            .values()
            .any(|n| n.parent().is_some() || !n.children().is_empty())
    }

    fn top_level_group_anchors_in_paint_order(&self) -> Vec<NodeId> {
        self.paint_order()
            .into_iter()
            .filter(|id| is_top_level_group_anchor(self, *id))
            .collect()
    }
}

fn is_top_level_group_anchor(graph: &Graph, id: NodeId) -> bool {
    if graph.children_of(id).is_empty() {
        return false;
    }
    graph.ancestors(id).all(|a| graph.children_of(a).is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_graph_skips_graph_scope_nodes() {
        let mut g = Graph::new();
        let _a = g.create_node("default").build();
        let _b = g.create_node("default").build();
        assert!(!g.has_node_hierarchy());
    }

    #[test]
    fn nested_graph_classifies_scope() {
        let mut g = Graph::new();
        let p = g.create_node("default").build();
        let c = g.create_node("default").build();
        g.add_child(p, c).unwrap();
        assert!(g.has_node_hierarchy());
        assert_eq!(g.top_level_group_anchors_in_paint_order(), vec![p]);
    }
}
