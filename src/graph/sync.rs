use yrs::{
    Array, ArrayPrelim, ArrayRef, Doc, GetString, Map, MapPrelim, MapRef, Transact, TransactionMut,
};

use crate::{Edge, EdgeId, Graph, Node, NodeId, Port, PortId};

pub struct YGraph {
    pub doc: Doc,
    pub nodes: MapRef,        // ref HashMap<NodeId, Node>
    pub ports: MapRef,        // ref HashMap<PortId, Port>
    pub edges: MapRef,        // ref HashMap<EdgeId, Edge>
    pub node_order: ArrayRef, // ref Vec<NodeId>
}

impl YGraph {
    pub fn new() -> Self {
        let doc = Doc::new();
        let root = doc.get_or_insert_map("graph");

        let mut txn = doc.transact_mut();
        let nodes = root.insert(&mut txn, "nodes", MapPrelim::default());
        let ports = root.insert(&mut txn, "ports", MapPrelim::default());
        let edges = root.insert(&mut txn, "edges", MapPrelim::default());
        let node_order = root.insert(&mut txn, "node_order", ArrayPrelim::default());

        drop(txn);

        Self {
            doc,
            nodes,
            ports,
            edges,
            node_order,
        }
    }

    pub fn insert_node(&self, node: &Node, ports: &[&Port]) {
        let mut txn = self.doc.transact_mut();

        let node_map = MapPrelim::default();
        let node_ref = self.nodes.insert(&mut txn, node.id.0.to_string(), node_map);

        node_ref.insert(&mut txn, "type", node.node_type.clone());
        node_ref.insert(&mut txn, "x", Into::<f32>::into(node.x));
        node_ref.insert(&mut txn, "y", Into::<f32>::into(node.y));
        node_ref.insert(&mut txn, "width", Into::<f32>::into(node.size.width));
        node_ref.insert(&mut txn, "height", Into::<f32>::into(node.size.height));

        let data_json = serde_json::to_string(&node.data).unwrap_or_default();
        node_ref.insert(&mut txn, "data", data_json);

        let ports_container = node_ref.insert(&mut txn, "ports", MapPrelim::default());

        for port in ports {
            let port_map =
                ports_container.insert(&mut txn, port.id.0.to_string(), MapPrelim::default());
            write_port_to_map(&mut txn, &port_map, port);
        }

        self.node_order.push_back(&mut txn, node.id.0.to_string());
    }

    pub fn insert_edge(&self, edge: &Edge) {
        let mut txn = self.doc.transact_mut();

        let edge_map = MapPrelim::default();
        let edge_ref = self.edges.insert(&mut txn, edge.id.0.to_string(), edge_map);

        edge_ref.insert(&mut txn, "source_port", edge.source_port.0.to_string());
        edge_ref.insert(&mut txn, "target_port", edge.target_port.0.to_string());
    }

    pub fn remove_edge(&self, id: &EdgeId) {
        let mut txn = self.doc.transact_mut();

        self.edges.remove(&mut txn, &id.0.to_string());
    }
}

impl From<Graph> for YGraph {
    fn from(value: Graph) -> Self {
        let ygraph = Self::new();
        for node in value.nodes().values() {
            let ports: Vec<_> = value
                .ports
                .values()
                .filter(|port| port.node_id == node.id)
                .collect();
            ygraph.insert_node(node, &ports);
        }

        for edge in value.edges.values() {
            ygraph.insert_edge(edge);
        }

        ygraph
    }
}

fn write_port_to_map(txn: &mut TransactionMut, port_map: &MapRef, port: &Port) {
    port_map.insert(txn, "kind", port.kind.to_string());
    port_map.insert(txn, "index", port.index as u32);
    port_map.insert(txn, "position", port.position.to_string());
    port_map.insert(txn, "width", Into::<f32>::into(port.size.width));
    port_map.insert(txn, "height", Into::<f32>::into(port.size.height));
}
