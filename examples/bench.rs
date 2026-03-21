use ferrum_flow::*;
use gpui::{AppContext as _, Application, WindowOptions};

fn main() {
    Application::new().run(|cx| {
        let mut graph = Graph::new();

        for j in 0..100 {
            for i in 0..100 {
                graph
                    .create_node("")
                    .position(200.0 * i as f32, 200.0 * j as f32)
                    .input()
                    .output()
                    .build(&mut graph);
            }
        }

        let node_ids = graph.nodes().iter().map(|(id, _)| *id).collect::<Vec<_>>();

        generate_chain_edges(&mut graph, node_ids);

        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|fc| {
                let mut flow = FlowCanvas::new(graph, fc)
                    .plugin(SelectionPlugin::new())
                    .plugin(NodeInteractionPlugin::new())
                    .plugin(ViewportPlugin::new())
                    .plugin(BackgroundPlugin::new())
                    .plugin(NodePlugin::new())
                    .plugin(PortInteractionPlugin::new())
                    .plugin(EdgePlugin::new())
                    .plugin(DeletePlugin::new())
                    .plugin(HistoryPlugin::new());
                flow.init_plugins();
                flow
            })
        })
        .unwrap();
    });
}

pub fn generate_chain_edges(graph: &mut Graph, node_ids: Vec<NodeId>) {
    for window in node_ids.windows(2) {
        let from = window[0];
        let to = window[1];

        let from_node = graph.get_node(&from).unwrap();
        let to_node = graph.get_node(&to).unwrap();

        let source_port = from_node.outputs[0];
        let target_port = to_node.inputs[0];

        EdgeBuilder::new()
            .source(source_port)
            .target(target_port)
            .build(graph);
    }
}
