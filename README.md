# FerrumFlow

A high-performance, extensible node-based editor built with Rust and gpui.
Designed for building visual programming tools, workflow editors, and graph-based UIs.

**This project is in early stage (alpha), API may change**

## Features

- Plugin-based architecture
- Interaction system (drag, pan, select, etc.)
- Undo / Redo (Command pattern)
- Viewport control (zoom & pan)
- Box selection & multi-select
- Node / Port / Edge model
- Custom node rendering system
- Built with performance in mind
- Multi-user collaboration support (by [plugin](https://github.com/tu6ge/ferrum-flow/tree/master/crates/sync_plugin))

https://github.com/user-attachments/assets/1d275e79-fcbf-4a4e-aba5-a1e3f3ff6029

[GitHub](https://github.com/tu6ge/ferrum-flow)

## Usage

```bash
cargo add ferrum-flow
```

This is a hello world example:

```rust
use ferrum_flow::{FlowCanvas, Graph};
use gpui::{AppContext as _, Application, WindowOptions};
use serde_json::json;

fn main() {
    Application::new().run(|cx| {
        let mut graph = Graph::new();

        graph
            .create_node("default")
            .position(100.0, 100.0)
            .data(json!({ "label": "Hello World" }))
            .build();

        cx.open_window(WindowOptions::default(), |window, cx| {
            cx.new(|ctx| {
                FlowCanvas::builder(graph, ctx, window)
                    .default_plugins() // Includes built-in rendering for nodes, edges, selection, and more. Replace with custom plugins as needed.
                    .build()
            })
        })
        .unwrap();
    });
}
```

For more examples, see the [examples directory](./crates/core/examples/).

## Architecture Overview

The system is designed with clear separation of concerns:

### Core Concepts

- Graph
  Stores persistent data (nodes, edges, ports)

- Viewport
  Handles zooming and panning

- Plugin System
  Extends behavior (rendering, input handling, etc.)
- Interaction System
  Manages ongoing user interactions (dragging, selecting, etc.)

- Command System
  Enables undo/redo support

### Plugin System

Plugins are the primary extension mechanism:

```rust
pub trait Plugin {
    fn name(&self) -> &'static str;

    fn setup(&mut self, ctx: &mut InitPluginContext);

    fn on_event(&mut self, event: &FlowEvent, ctx: &mut PluginContext) -> EventResult;

    fn render(&mut self, ctx: &mut RenderContext) -> Option<AnyElement>;

    fn priority(&self) -> i32 {
        0
    }

    fn render_layer(&self) -> RenderLayer {
        RenderLayer::Overlay
    }
}
```

**Responsibilities**

A plugin can:

- Handle input events
- Start interactions
- Render UI layers
- Modify graph state

### Interaction System

Interactions represent ongoing user actions, such as:

- Node dragging
- Box selection
- Viewport panning

```rust
pub trait Interaction {
    fn on_mouse_move(&mut self, event: &MouseMoveEvent, ctx: &mut PluginContext) -> InteractionResult;

    fn on_mouse_up(&mut self, event: &MouseUpEvent, ctx: &mut PluginContext) -> InteractionResult;

    fn render(&self, ctx: &mut RenderContext) -> Option<AnyElement>;
}
```

Interaction Lifecycle

```
Start → Update → End / Replace
```

```rust
pub enum InteractionResult {
    Continue,
    End,
    Replace(Box<dyn Interaction>),
}
```

### Command System (Undo / Redo)

Implements the Command Pattern:

```rust
pub trait Command {
    fn execute(&mut self, ctx: &mut CommandContext);
    fn undo(&mut self, ctx: &mut CommandContext);
}
```

Built-in Features

- Undo / Redo stacks
- Composite commands
- Easy integration via PluginContext

```rust
ctx.execute_command(MyCommand { ... });
```

### Node Rendering

Rendering is fully customizable via a registry:

```rust
pub trait NodeRenderer {
    fn render(&self, node: &Node, ctx: &mut RenderContext) -> AnyElement;

    // custom render port UI
    fn port_render(&self, node: &Node, port: &Port, ctx: &mut RenderContext) -> Option<AnyElement> {
        // ... default implement
    }

    // computing the position of port relative to node
    fn port_offset(&self, node: &Node, port: &Port, graph: &Graph) -> Point<Pixels> {
        // ... default implement
    }
}
```

Render example:

```rust
// Absolute-positioned node card shell: screen origin, zoom-scaled size.
ctx.node_card_shell(node, false, NodeCardVariant::Custom)
    .rounded(px(6.0))
    .border(px(1.5))
```

### Graph Model

```rust
pub struct Node {
    id: NodeId,
    node_type: String,
    x: Pixels,
    y: Pixels,
    size: Size<Pixels>,
    inputs: Vec<PortId>,
    outputs: Vec<PortId>,
    data: serde_json::Value,
}
```

Creating Nodes (Builder API)

```rust
graph.create_node("math.add")
    .position(100.0, 100.0)
    .input()
    .output()
    .build();
```

### Performance

Designed to scale to large graphs:

- Viewport-based rendering (virtualization)
- Layered rendering system
- Interaction-aware rendering (degraded mode during drag)
- Ready for spatial indexing

### Design Principles

- Separation of data and interaction
- Plugins over hardcoded behavior
- Explicit state transitions
- Performance-first rendering
- Composable architecture

### Feature parity & gap analysis (TODO)

We want an explicit, maintained view of **supported vs partial vs missing** relative to React Flow’s documented capabilities (nodes, edges, handles, selection, keyboard, minimap, controls, snapping, grouping/subflows, accessibility, etc.):

- [ ] **Audit** — Walk the React Flow feature list and map each item to FerrumFlow (plugin, core graph, or N/A by design).
- [ ] **Gap list** — For every row, mark *done*, *partial*, *missing*, or *different by design* (short rationale).
- [ ] **Surface in this README** — Add a compact table or bullet matrix here (or link to `docs/react-flow-parity.md` if it grows large).
- [ ] change `port_type` of Port struct to custom enum type.

Contributions welcome: propose a matrix in an issue or open a PR that extends this section.

### Mid-term design goals

Foundation work that unlocks most other extensions:

- [ ] **Separate data, logic, and rendering** — Clear boundaries between graph/state, interaction and commands, and GPUI (or future) paint so features can grow without entangling layers.
- [ ] **Large-graph performance** — Investigate **arena-style** allocation for nodes/edges and **ID-based references** instead of pointer-heavy graphs to improve locality and scale.
- [ ] **Encapsulate graph model fields** — Make transitional public fields on `Node` / `Port` private in the next release after migration APIs settle.

### Long-term / directional

No active schedule; keep these in mind when designing APIs above.

- **Abstract / pluggable render backend** — Allow FerrumFlow to sit inside **existing render systems** (similar in spirit to graph editors embedded in tools like Blender or Unreal: the host owns the surface; the library supplies model + interaction contracts).
- **WASM target** — When the stack allows, support **Web** deployment paths.
- **Predictive rendering for collaboration** — Reduce **felt latency** in multi-user editing (optimistic / speculative UI reconciled with synced state, e.g. CRDT-backed updates).

## Contributing

Contributions are welcome!

Feel free to open issues or PRs for:

- New plugins
- Performance improvements
- API design suggestions

## License

Apache2.0
