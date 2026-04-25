use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use futures::channel::mpsc::UnboundedSender;
use gpui::{Element as _, ParentElement, PathBuilder, Pixels, Point, Styled as _, div, px, rgb};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use yrs::{
    Any, Array as _, ArrayRef, DeepObservable, Doc, Map, MapPrelim, MapRef, Observable as _,
    Origin, Out, ReadTxn, Transact, TransactionMut,
    encoding::serde::{from_any, to_any},
    sync::Awareness,
    types::{DefaultPrelim, EntryChange, PathSegment},
    undo::Options,
};

use ferrum_flow::{
    ChangeSource, Edge, EdgeId, FlowEvent, Graph, GraphChange, GraphChangeKind, GraphOp,
    InputEvent, Node, NodeBuilder, NodeId, Port, PortBuilder, PortId, PortKind, PortPosition,
    PortType, SyncPlugin, SyncPluginContext,
};

use crate::server::{WsSyncConfig, start_sync_thread};

pub struct YrsSyncPlugin {
    doc: yrs::Doc,
    awareness: Arc<Awareness>,
    init_graph: Graph,
    nodes: MapRef,        // ref HashMap<NodeId, Node>
    ports: MapRef,        // ref HashMap<PortId, Port>
    edges: MapRef,        // ref HashMap<EdgeId, Edge>
    node_order: ArrayRef, // ref Vec<NodeId>
    undo_manager: yrs::UndoManager,
    undo_origin: Origin,
    _subscription_nodes: Option<yrs::Subscription>,
    _subscription_ports: Option<yrs::Subscription>,
    _subscription_edges: Option<yrs::Subscription>,
    _subscription_order: Option<yrs::Subscription>,
    _subscription_doc_update: Option<yrs::Subscription>,
    last_awareness_push: Option<Instant>,
    ws_url: String,
    ws_sync_config: WsSyncConfig,
}

impl YrsSyncPlugin {
    pub fn new(graph: Graph, ws_url: &str) -> Self {
        Self::with_ws_config(graph, ws_url, WsSyncConfig::default())
    }

    /// Same as [`Self::new`], but uses a custom WebSocket reconnect / retry policy.
    pub fn with_ws_config(graph: Graph, ws_url: &str, ws_sync_config: WsSyncConfig) -> Self {
        let doc = Doc::new();
        let root = doc.get_or_insert_map("graph");
        let nodes = doc.get_or_insert_map("nodes");
        let ports = doc.get_or_insert_map("ports");
        let edges = doc.get_or_insert_map("edges");
        let node_order = doc.get_or_insert_array("node_order");

        let mut option = Options::default();
        option.tracked_origins.insert("local_intent".into());
        let mut undo_manager = yrs::UndoManager::with_scope_and_options(&doc, &root, option);
        undo_manager.expand_scope(&nodes);
        undo_manager.expand_scope(&ports);
        undo_manager.expand_scope(&edges);
        undo_manager.expand_scope(&node_order);
        let undo_origin = undo_manager.as_origin();

        let awareness = Arc::new(Awareness::new(doc.clone()));

        Self {
            awareness,
            last_awareness_push: None,
            init_graph: graph,
            undo_manager,
            undo_origin,
            doc,
            nodes,
            ports,
            edges,
            node_order,
            _subscription_nodes: None,
            _subscription_ports: None,
            _subscription_edges: None,
            _subscription_order: None,
            _subscription_doc_update: None,
            ws_url: ws_url.to_string(),
            ws_sync_config,
        }
    }

    pub fn from_graph(&self) {
        if self.init_graph.is_empty() {
            return;
        }
        let mut txn: TransactionMut<'_> = self.doc.transact_mut_with("local_init");
        for node in self.init_graph.nodes().values() {
            self.insert_node(&mut txn, node);
        }
        for port in self.init_graph.ports_values() {
            self.add_port(&mut txn, port);
        }
        for edge in self.init_graph.edges_values() {
            self.insert_edge(&mut txn, edge);
        }
        for order in self.init_graph.node_order() {
            self.add_node_order(&mut txn, order);
        }
    }

    fn insert_node(&self, txn: &mut TransactionMut, node: &Node) {
        let node_map = MapPrelim::default();
        let node_ref = self.nodes.insert(txn, node.id().to_string(), node_map);

        node_ref.insert(txn, "type", node.renderer_key());
        node_ref.insert(txn, "execute_type", node.execute_type_ref());
        node_ref.insert(txn, "x", Into::<f32>::into(node.position().0));
        node_ref.insert(txn, "y", Into::<f32>::into(node.position().1));
        node_ref.insert(txn, "width", Into::<f32>::into(node.size_ref().width));
        node_ref.insert(txn, "height", Into::<f32>::into(node.size_ref().height));

        let inputs = node_ref.insert(txn, "inputs", ArrayRef::default_prelim());
        for port_id in node.inputs() {
            inputs.push_back(txn, port_id.to_string());
        }
        let outputs = node_ref.insert(txn, "outputs", ArrayRef::default_prelim());
        for port_id in node.outputs() {
            outputs.push_back(txn, port_id.to_string());
        }

        let data_json = to_any(&node.data_ref()).unwrap_or_else(|_| Any::Null);
        node_ref.insert(txn, "data", data_json);
    }

    fn update_node_position(&self, txn: &mut TransactionMut, id: &NodeId, x: f32, y: f32) {
        if let Some(yrs::Out::YMap(node_ref)) = self.nodes.get(txn, &id.to_string()) {
            node_ref.insert(txn, "x", x);
            node_ref.insert(txn, "y", y);
        }
    }

    fn add_node_order(&self, txn: &mut TransactionMut, id: &NodeId) {
        self.node_order.push_back(txn, id.to_string());
    }

    fn remove_noder_order(&self, txn: &mut TransactionMut, index: usize) {
        self.node_order.remove(txn, index as u32);
    }

    fn remove_node(&self, txn: &mut TransactionMut, id: &NodeId) {
        self.nodes.remove(txn, &id.to_string());
    }

    fn add_port(&self, txn: &mut TransactionMut, port: &Port) {
        let port_map = self
            .ports
            .insert(txn, port.id().to_string(), MapPrelim::default());
        write_port_to_map(txn, &port_map, port);
    }

    fn remove_port(&self, txn: &mut TransactionMut, id: &PortId) {
        self.ports.remove(txn, &id.to_string());
    }

    fn insert_edge(&self, txn: &mut TransactionMut, edge: &Edge) {
        let edge_map = MapPrelim::default();
        let edge_ref = self.edges.insert(txn, edge.id.to_string(), edge_map);

        edge_ref.insert(txn, "source_port", edge.source_port.to_string());
        edge_ref.insert(txn, "target_port", edge.target_port.to_string());
    }

    fn remove_edge(&self, txn: &mut TransactionMut, id: &EdgeId) {
        self.edges.remove(txn, &id.to_string());
    }

    fn inner_process_intent(&self, txn: &mut TransactionMut, intent: GraphOp) {
        match intent {
            GraphOp::MoveNode { id, x, y } => {
                self.update_node_position(txn, &id, x, y);
            }
            GraphOp::AddNode(node) => self.insert_node(txn, &node),
            GraphOp::RemoveNode { id } => self.remove_node(txn, &id),
            GraphOp::ResizeNode { .. } => todo!(),
            GraphOp::UpdateNodeData { .. } => todo!(),
            GraphOp::NodeOrderInsert { id } => self.add_node_order(txn, &id),
            GraphOp::NodeOrderRemove { index } => self.remove_noder_order(txn, index),
            GraphOp::AddPort(port) => self.add_port(txn, &port),
            GraphOp::RemovePort(port_id) => self.remove_port(txn, &port_id),
            GraphOp::AddEdge(edge) => self.insert_edge(txn, &edge),
            GraphOp::RemoveEdge(edge_id) => self.remove_edge(txn, &edge_id),
            GraphOp::Batch(graph_ops) => {
                for op in graph_ops {
                    self.inner_process_intent(txn, op);
                }
            }
            _ => unimplemented!("GraphOp: {:?}", intent),
        }
    }

    fn on_mouse_move(&mut self, world: Point<Pixels>) {
        const MIN_INTERVAL: Duration = Duration::from_millis(33);
        let now = Instant::now();
        if let Some(prev) = self.last_awareness_push {
            if now.saturating_duration_since(prev) < MIN_INTERVAL {
                return;
            }
        }
        self.last_awareness_push = Some(now);
        let state = RemoteCursorState {
            x: world.x.into(),
            y: world.y.into(),
        };
        let _ = self.awareness.set_local_state(&state);
    }
}

impl SyncPlugin for YrsSyncPlugin {
    fn name(&self) -> &'static str {
        "YrsSyncPlugin"
    }

    fn setup(&mut self, change_sender: UnboundedSender<GraphChange>) {
        let change_sender_clone = change_sender.clone();
        let change_sender_clone2 = change_sender.clone();
        let change_sender_clone3 = change_sender.clone();
        let change_sender_clone4 = change_sender.clone();
        let undo_origin = self.undo_origin.clone();
        let nodes_ref = self.nodes.clone();
        let sub = self.nodes.observe_deep(move |txn, event| {
            let source = match txn.origin() {
                Some(orig) if *orig == Origin::from("local_intent") => ChangeSource::Local,
                Some(orig) if *orig == undo_origin => ChangeSource::Undo,
                _ => ChangeSource::Remote,
            };

            for ev in event.iter() {
                if let yrs::types::Event::Map(ev) = ev {
                    let kind = handler_node_change(txn, ev, &nodes_ref);
                    if !kind.is_empty() {
                        let _ = change_sender_clone.unbounded_send(GraphChange {
                            kind: GraphChangeKind::Batch(kind),
                            source,
                        });
                    }
                }
            }
        });

        self._subscription_nodes = Some(sub);

        let undo_origin = self.undo_origin.clone();
        let sub = self.ports.observe(move |txn, event| {
            let source = match txn.origin() {
                Some(orig) if *orig == Origin::from("local_intent") => ChangeSource::Local,
                Some(orig) if *orig == undo_origin => ChangeSource::Undo,
                _ => ChangeSource::Remote,
            };

            for (key, change) in event.keys(txn) {
                if let Some(kind) = parse_port_change(txn, key, change) {
                    let _ = change_sender_clone2.unbounded_send(GraphChange { kind, source });
                }
            }
        });
        self._subscription_ports = Some(sub);

        let undo_origin = self.undo_origin.clone();

        let sub = self.edges.observe(move |txn, event| {
            let source = match txn.origin() {
                Some(orig) if *orig == Origin::from("local_intent") => ChangeSource::Local,
                Some(orig) if *orig == undo_origin => ChangeSource::Undo,
                _ => ChangeSource::Remote,
            };

            for (key, change) in event.keys(txn) {
                if let Some(kind) = parse_edge_change(txn, key, change) {
                    let _ = change_sender_clone3.unbounded_send(GraphChange { kind, source });
                }
            }
        });
        self._subscription_edges = Some(sub);

        let undo_origin = self.undo_origin.clone();
        let sub = self.node_order.observe(move |txn, event| {
            let source = match txn.origin() {
                Some(orig) if *orig == Origin::from("local_intent") => ChangeSource::Local,
                Some(orig) if *orig == undo_origin => ChangeSource::Undo,
                _ => ChangeSource::Remote,
            };

            let array = event.target();

            let mut list = vec![];
            for item in array.iter(txn) {
                if let Out::Any(Any::String(str)) = item {
                    if let Ok(uuid) = str.to_string().parse() {
                        list.push(NodeId::from_uuid(uuid));
                    }
                }
            }

            let _ = change_sender_clone4.unbounded_send(GraphChange {
                kind: GraphChangeKind::NodeOrderUpdate(list),
                source,
            });
        });
        self._subscription_order = Some(sub);

        self.from_graph();

        start_sync_thread(
            Arc::clone(&self.awareness),
            self.undo_origin.clone(),
            change_sender,
            self.ws_url.clone(),
            self.ws_sync_config.clone(),
        );
    }

    fn process_intent(&self, intent: GraphOp) {
        // println!("current op: {:?}", intent.clone());
        let mut txn = self.doc.transact_mut_with(Origin::from("local_intent"));
        self.inner_process_intent(&mut txn, intent);
    }

    fn undo(&mut self) {
        self.undo_manager.undo_blocking();
    }

    fn redo(&mut self) {
        self.undo_manager.redo_blocking();
    }

    fn on_event(&mut self, event: &FlowEvent, ctx: &mut SyncPluginContext) {
        match event {
            FlowEvent::Input(InputEvent::MouseMove(event)) => {
                self.on_mouse_move(ctx.screen_to_world(event.position));
            }
            FlowEvent::Input(InputEvent::Hover(hovered)) => {
                if !*hovered {
                    self.last_awareness_push = None;
                    self.awareness.clean_local_state();
                }
            }
            _ => {}
        }
    }

    fn render(&mut self, ctx: &mut ferrum_flow::RenderContext) -> Vec<gpui::AnyElement> {
        let me = self.awareness.client_id();
        let mut out = Vec::new();
        for (client_id, state) in self.awareness.iter() {
            if client_id == me {
                continue;
            }
            let Some(data) = state.data.as_ref() else {
                continue;
            };
            let Ok(cursor) = serde_json::from_str::<RemoteCursorState>(data) else {
                continue;
            };
            let screen = ctx.world_to_screen(Point::new(px(cursor.x), px(cursor.y)));
            let color = color_for_client(client_id);
            out.push(
                div()
                    .absolute()
                    .left(screen.x)
                    .top(screen.y)
                    .w(px(18.0))
                    .h(px(24.0))
                    .child(
                        gpui::canvas(
                            move |_, _, _| color,
                            move |bounds, fill_rgb, win, _| {
                                let ox = bounds.origin.x;
                                let oy = bounds.origin.y;
                                let p0 = Point::new(ox + px(1.0), oy + px(1.0));
                                let p1 = Point::new(ox + px(1.0), oy + px(21.0));
                                let p2 = Point::new(ox + px(6.8), oy + px(15.8));
                                let p3 = Point::new(ox + px(10.4), oy + px(23.0));
                                let p4 = Point::new(ox + px(13.6), oy + px(21.4));
                                let p5 = Point::new(ox + px(10.0), oy + px(14.2));
                                let p6 = Point::new(ox + px(17.0), oy + px(14.0));

                                let mut fill = PathBuilder::fill();
                                fill.move_to(p0);
                                fill.line_to(p1);
                                fill.line_to(p2);
                                fill.line_to(p3);
                                fill.line_to(p4);
                                fill.line_to(p5);
                                fill.line_to(p6);
                                fill.line_to(p0);
                                if let Ok(path) = fill.build() {
                                    win.paint_path(path, rgb(fill_rgb));
                                }

                                let mut stroke = PathBuilder::stroke(px(1.2));
                                stroke.move_to(p0);
                                stroke.line_to(p1);
                                stroke.line_to(p2);
                                stroke.line_to(p3);
                                stroke.line_to(p4);
                                stroke.line_to(p5);
                                stroke.line_to(p6);
                                stroke.line_to(p0);
                                if let Ok(path) = stroke.build() {
                                    win.paint_path(path, rgb(0xFFFFFF));
                                }
                            },
                        )
                        .size_full(),
                    )
                    .into_any(),
            );
        }
        out
    }
}

fn color_for_client(client_id: u64) -> u32 {
    // Keep colors bright and deterministic per awareness client id.
    let hue = client_id.wrapping_mul(2654435761) % 360;
    hsl_to_rgb_u32(hue as f32, 72.0, 55.0)
}

fn hsl_to_rgb_u32(h: f32, s: f32, l: f32) -> u32 {
    let s = (s / 100.0).clamp(0.0, 1.0);
    let l = (l / 100.0).clamp(0.0, 1.0);
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = (h / 60.0).rem_euclid(6.0);
    let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
    let (r1, g1, b1) = if hp < 1.0 {
        (c, x, 0.0)
    } else if hp < 2.0 {
        (x, c, 0.0)
    } else if hp < 3.0 {
        (0.0, c, x)
    } else if hp < 4.0 {
        (0.0, x, c)
    } else if hp < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let m = l - c / 2.0;
    let r = ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u32;
    let g = ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u32;
    let b = ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u32;
    (r << 16) | (g << 8) | b
}

/// Cursor position in graph (world) coordinates; stored in Yjs awareness as JSON.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct RemoteCursorState {
    pub x: f32,
    pub y: f32,
}

fn write_port_to_map(txn: &mut TransactionMut, port_map: &MapRef, port: &Port) {
    port_map.insert(txn, "kind", port.kind().to_string());
    port_map.insert(txn, "node_id", port.node_id().to_string());
    port_map.insert(txn, "index", port.index() as u32);
    port_map.insert(txn, "position", port.position().to_string());
    port_map.insert(txn, "width", Into::<f32>::into(port.size_ref().width));
    port_map.insert(txn, "height", Into::<f32>::into(port.size_ref().height));
    port_map.insert(
        txn,
        "port_type",
        to_any(&port.port_type_ref()).unwrap_or_else(|_| Any::Null),
    );
}

/// Resolve node id for a [MapEvent]. For nested maps (per-node YMap), `MapEvent::path()` is often
/// **empty** because yrs sets `current_target == target` (see `MapEvent::new`), so we fall back to
/// finding which key under `nodes` holds the same `YMap` as `ev.target()`.
fn node_id_for_map_event(
    txn: &yrs::TransactionMut,
    nodes: &MapRef,
    ev: &yrs::types::map::MapEvent,
) -> Option<NodeId> {
    for segment in ev.path().iter().rev() {
        if let PathSegment::Key(key) = segment {
            if let Ok(id) = key.to_string().parse::<Uuid>() {
                return Some(NodeId::from_uuid(id));
            }
        }
    }

    let target = ev.target();
    for (key, out) in nodes.iter(txn) {
        if let Out::YMap(m) = out {
            if m == *target {
                return key.to_string().parse::<Uuid>().ok().map(NodeId::from_uuid);
            }
        }
    }

    None
}

fn handler_node_change(
    txn: &yrs::TransactionMut,
    ev: &yrs::types::map::MapEvent,
    nodes: &MapRef,
) -> Vec<GraphChangeKind> {
    let node_id = node_id_for_map_event(txn, nodes, ev);

    let changed_keys: Vec<&str> = ev.keys(txn).keys().map(|k| k.as_ref()).collect();
    let is_nodes_map_child_change = changed_keys.iter().any(|k| k.parse::<Uuid>().is_ok());

    let mut kind: Vec<GraphChangeKind> = vec![];

    if is_nodes_map_child_change {
        for (key, change) in ev.keys(txn) {
            if let Some(k) = parse_node_change(txn, key, change) {
                kind.push(k);
            }
        }
    }

    let pos_dirty = changed_keys.iter().any(|k| *k == "x" || *k == "y");
    let width_dirty = changed_keys.iter().any(|k| *k == "width");
    let height_dirty = changed_keys.iter().any(|k| *k == "height");
    let data_dirty = changed_keys.iter().any(|k| *k == "data");

    if pos_dirty || width_dirty || height_dirty || data_dirty {
        if let Some(id) = node_id {
            let node_map = ev.target();
            if pos_dirty {
                if let (Some(x), Some(y)) = (
                    read_map_f32(txn, node_map, "x"),
                    read_map_f32(txn, node_map, "y"),
                ) {
                    kind.push(GraphChangeKind::NodeMoved { id, x, y });
                }
            }
            if width_dirty {
                if let Some(width) = read_map_f32(txn, node_map, "width") {
                    kind.push(GraphChangeKind::NodeSetWidthed { id, width });
                }
            }
            if height_dirty {
                if let Some(height) = read_map_f32(txn, node_map, "height") {
                    kind.push(GraphChangeKind::NodeSetHeighted { id, height });
                }
            }
            if data_dirty {
                if let Ok(data) =
                    from_any(&node_map.get_as(txn, "data").unwrap_or_else(|_| Any::Null))
                {
                    kind.push(GraphChangeKind::NodeDataUpdated { id, data });
                }
            }
        }
    }

    kind
}

fn parse_node_change(
    txn: &yrs::TransactionMut,
    key: &Arc<str>,
    change: &EntryChange,
) -> Option<GraphChangeKind> {
    let id = NodeId::from_uuid(key.to_string().parse().ok()?);

    match change {
        EntryChange::Inserted(value) => {
            if let yrs::Out::YMap(node_map) = value {
                Some(GraphChangeKind::NodeAdded(read_node_from_map(
                    txn, node_map, id,
                )))
            } else {
                None
            }
        }
        EntryChange::Removed(_) => Some(GraphChangeKind::NodeRemoved { id }),
        EntryChange::Updated(_, _) => None,
    }
}

fn read_node_from_map(txn: &yrs::TransactionMut, node_map: &MapRef, id: NodeId) -> Node {
    let node_type: String = node_map.get_as(txn, "type").unwrap_or_default();
    let execute_type: String = node_map.get_as(txn, "execute_type").unwrap_or_default();
    let x = read_map_f32(txn, node_map, "x").unwrap_or_default();
    let y = read_map_f32(txn, node_map, "y").unwrap_or_default();
    let width = read_map_f32(txn, node_map, "width").unwrap_or_default();
    let height = read_map_f32(txn, node_map, "height").unwrap_or_default();
    let data = from_any(&node_map.get_as(txn, "data").unwrap_or_else(|_| Any::Null))
        .unwrap_or_else(|_| Value::Null);

    let out_inputs = node_map.get(txn, "inputs");
    let mut inputs = vec![];
    if let Some(Out::YArray(arr)) = out_inputs {
        for item in arr.iter(txn) {
            if let Out::Any(Any::String(str)) = item {
                if let Ok(uuid) = str.to_string().parse() {
                    inputs.push(PortId::from_uuid(uuid));
                }
            }
        }
    }

    let out_outputs = node_map.get(txn, "outputs");
    let mut outputs = vec![];
    if let Some(Out::YArray(arr)) = out_outputs {
        for item in arr.iter(txn) {
            if let Out::Any(Any::String(str)) = item {
                if let Ok(uuid) = str.to_string().parse() {
                    outputs.push(PortId::from_uuid(uuid));
                }
            }
        }
    }

    NodeBuilder::new(node_type)
        .execute_type(execute_type)
        .position(x, y)
        .size(width, height)
        .data(data)
        .build_raw_with_port_ids(id, inputs, outputs)
}

fn parse_port_change(
    txn: &yrs::TransactionMut,
    key: &Arc<str>,
    change: &EntryChange,
) -> Option<GraphChangeKind> {
    let id = PortId::from_uuid(key.to_string().parse().ok()?);

    match change {
        EntryChange::Inserted(value) => {
            if let yrs::Out::YMap(port_map) = value {
                Some(GraphChangeKind::PortAdded(read_port_from_map(
                    txn, port_map, id,
                )))
            } else {
                None
            }
        }

        EntryChange::Removed(_) => Some(GraphChangeKind::PortRemoved { id }),

        _ => None,
    }
}

fn read_port_from_map(txn: &yrs::TransactionMut, node_map: &MapRef, id: PortId) -> Port {
    let kind: String = node_map.get_as(txn, "kind").unwrap_or_default();
    let node_id: String = node_map.get_as(txn, "node_id").unwrap_or_default();
    let index: u32 = node_map.get_as(txn, "index").unwrap_or_default();
    let position: String = node_map.get_as(txn, "position").unwrap_or_default();
    let width: f32 = node_map.get_as(txn, "width").unwrap_or_default();
    let height: f32 = node_map.get_as(txn, "height").unwrap_or_default();
    let port_type = match node_map.get_as::<_, Any>(txn, "port_type") {
        Ok(any) => from_any::<PortType>(&any).unwrap_or(PortType::Any),
        Err(_) => PortType::Any,
    };

    let kind = if kind == "input" {
        PortKind::Input
    } else {
        PortKind::Output
    };
    let node_id = NodeId::from_uuid(node_id.parse().unwrap_or_default());
    let position = PortPosition::from_str(&position).unwrap_or(PortPosition::Left);

    PortBuilder::new(id)
        .kind(kind)
        .node_id(node_id)
        .index(index as usize)
        .position(position)
        .size(width, height)
        .port_type(port_type)
        .build()
}

fn parse_edge_change(
    txn: &yrs::TransactionMut,
    key: &Arc<str>,
    change: &EntryChange,
) -> Option<GraphChangeKind> {
    let id = EdgeId::from_uuid(key.to_string().parse().ok()?);

    match change {
        EntryChange::Inserted(value) => {
            if let yrs::Out::YMap(edge_map) = value {
                Some(GraphChangeKind::EdgeAdded(read_edge_from_map(
                    txn, edge_map, id,
                )?))
            } else {
                None
            }
        }

        EntryChange::Removed(_) => Some(GraphChangeKind::EdgeRemoved { id }),

        _ => None,
    }
}

fn read_edge_from_map(txn: &yrs::TransactionMut, node_map: &MapRef, id: EdgeId) -> Option<Edge> {
    let source_port: String = node_map.get_as(txn, "source_port").unwrap_or_default();
    let target_port: String = node_map.get_as(txn, "target_port").unwrap_or_default();

    Some(Edge {
        id,
        source_port: PortId::from_uuid(source_port.parse().ok()?),
        target_port: PortId::from_uuid(target_port.parse().ok()?),
    })
}

fn read_map_f32<T: ReadTxn>(txn: &T, map: &MapRef, key: &str) -> Option<f32> {
    map.get_as::<_, f64>(txn, key).ok().map(|v| v as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Number, Value, json};
    use std::collections::BTreeMap;

    fn canonicalize_json(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let sorted = map
                    .iter()
                    .map(|(k, v)| (k.clone(), canonicalize_json(v)))
                    .collect::<BTreeMap<_, _>>();
                Value::Object(sorted.into_iter().collect())
            }
            Value::Array(arr) => Value::Array(arr.iter().map(canonicalize_json).collect()),
            _ => value.clone(),
        }
    }

    fn json_numbers_equal(a: &Number, b: &Number) -> bool {
        match (a.as_i64(), b.as_i64()) {
            (Some(x), Some(y)) => return x == y,
            _ => {}
        }
        match (a.as_u64(), b.as_u64()) {
            (Some(x), Some(y)) => return x == y,
            _ => {}
        }
        match (a.as_f64(), b.as_f64()) {
            (Some(x), Some(y)) => (x - y).abs() < f64::EPSILON,
            _ => false,
        }
    }

    fn json_semantically_equal(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Number(x), Value::Number(y)) => json_numbers_equal(x, y),
            (Value::Array(xs), Value::Array(ys)) => {
                xs.len() == ys.len()
                    && xs
                        .iter()
                        .zip(ys.iter())
                        .all(|(x, y)| json_semantically_equal(x, y))
            }
            (Value::Object(xm), Value::Object(ym)) => {
                xm.len() == ym.len()
                    && xm.iter().all(|(k, xv)| {
                        ym.get(k)
                            .map(|yv| json_semantically_equal(xv, yv))
                            .unwrap_or(false)
                    })
            }
            _ => false,
        }
    }

    #[test]
    fn node_roundtrip_through_yrs_map_preserves_all_fields() {
        let plugin = YrsSyncPlugin::new(Graph::new(), "ws://localhost:0");

        let node_id = NodeId::new();
        let input_ports = vec![PortId::new(), PortId::new()];
        let output_ports = vec![PortId::new(), PortId::new()];
        let original = NodeBuilder::new("math/add")
            .execute_type("sync")
            .position(123.5, -45.25)
            .size(222.0, 88.0)
            .data(json!({
                "label": "Adder",
                "params": { "a": 1, "b": 2, "ratio": 1.25, "eps": 0.0001 },
                "flags": [true, false, true]
            }))
            .build_raw_with_port_ids(node_id, input_ports.clone(), output_ports.clone());

        {
            let mut txn = plugin.doc.transact_mut();
            plugin.insert_node(&mut txn, &original);
        }

        let txn = plugin.doc.transact_mut();
        let Some(Out::YMap(node_map)) = plugin.nodes.get(&txn, &original.id().to_string()) else {
            panic!("expected node map to be present in yrs document");
        };

        let restored = read_node_from_map(&txn, &node_map, original.id());

        assert_eq!(restored.id(), original.id());
        assert_eq!(restored.renderer_key(), original.renderer_key());
        assert_eq!(restored.execute_type_ref(), original.execute_type_ref());
        assert_eq!(restored.position().0, original.position().0);
        assert_eq!(restored.position().1, original.position().1);
        assert_eq!(restored.size_ref().width, original.size_ref().width);
        assert_eq!(restored.size_ref().height, original.size_ref().height);
        assert_eq!(restored.inputs(), original.inputs());
        assert_eq!(restored.outputs(), original.outputs());
        let restored_data = canonicalize_json(restored.data_ref());
        let original_data = canonicalize_json(original.data_ref());
        assert!(
            json_semantically_equal(&restored_data, &original_data),
            "semantic json mismatch:\nleft: {restored_data:?}\nright: {original_data:?}"
        );
    }

    #[test]
    fn port_roundtrip_through_yrs_map_preserves_all_fields() {
        let plugin = YrsSyncPlugin::new(Graph::new(), "ws://localhost:0");

        let port_id = PortId::new();
        let node_id = NodeId::new();
        let original = PortBuilder::new(port_id)
            .kind(PortKind::Input)
            .node_id(node_id)
            .index(3)
            .position(PortPosition::Bottom)
            .size(17.5, 9.25)
            .port_type(PortType::String)
            .build();

        {
            let mut txn = plugin.doc.transact_mut();
            plugin.add_port(&mut txn, &original);
        }

        let txn = plugin.doc.transact_mut();
        let Some(Out::YMap(port_map)) = plugin.ports.get(&txn, &original.id().to_string()) else {
            panic!("expected port map to be present in yrs document");
        };

        let restored = read_port_from_map(&txn, &port_map, original.id());

        assert_eq!(restored.id(), original.id());
        assert_eq!(restored.kind(), original.kind());
        assert_eq!(restored.node_id(), original.node_id());
        assert_eq!(restored.index(), original.index());
        assert_eq!(restored.position(), original.position());
        assert_eq!(restored.size_ref().width, original.size_ref().width);
        assert_eq!(restored.size_ref().height, original.size_ref().height);
        assert_eq!(restored.port_type_ref(), original.port_type_ref());
    }
}
