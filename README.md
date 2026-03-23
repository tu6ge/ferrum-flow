# FerrumFlow

A high-performance, extensible node-based editor built with Rust and gpui.
Designed for building visual programming tools, workflow editors, and graph-based UIs.

**This project is in early stage (alpha), API may change**

## Features

- 🧩 Plugin-based architecture
- 🧠 Interaction system (drag, pan, select, etc.)
- 🔄 Undo / Redo (Command pattern)
- 🔍 Viewport control (zoom & pan)
- 🖱️ Box selection & multi-select
- 🔗 Node / Port / Edge model
- 🎨 Custom node rendering system
- ⚡ Built with performance in mind (virtualization-ready)

[![Watch the video](https://img.youtube.com/vi/mimeKsIldog/0.jpg)](https://www.youtube.com/watch?v=mimeKsIldog)

[GitHub](https://github.com/tu6ge/ferrum-flow)

## Usage

```bash
cargo add ferrum-flow
```

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

    // computing the position of port relative to node
    fn port_offset(&self, node: &Node, port: &Port, graph: &Graph) -> Point<Pixels> {
        // ... default implement
    }
}
```

Render example:

```rust
div()
    .absolute()
    .left(x)
    .top(y)
    .w(width)
    .h(height)
    .bg(white())
```

### Graph Model

```rust
pub struct Node {
    pub id: NodeId,
    pub node_type: String,
    pub x: Pixels,
    pub y: Pixels,
    pub size: Size<Pixels>,
    pub inputs: Vec<PortId>,
    pub outputs: Vec<PortId>,
    pub data: serde_json::Value,
}
```

🏗️ Creating Nodes (Builder API)

```rust
graph.create_node("math.add")
    .position(100.0, 100.0)
    .input()
    .output()
    .build(&mut graph);
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

### Roadmap

- Edge rendering improvements
- Connection (drag-to-connect)
- Spatial indexing (Quadtree/Grid)
- Interaction priority & conflict resolution
- Collaboration support

## Contributing

Contributions are welcome!

Feel free to open issues or PRs for:

- New plugins
- Performance improvements
- API design suggestions

## License

Apache2.0
