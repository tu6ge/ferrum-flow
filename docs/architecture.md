# Architecture Overview

The system is designed with clear separation of concerns.

## Core Concepts

- **Graph**  
  Stores persistent data (nodes, edges, ports).

- **Viewport**  
  Handles zooming and panning.

- **Plugin System**  
  Extends behavior (rendering, input handling, etc.).

- **Interaction System**  
  Manages ongoing user interactions (dragging, selecting, etc.).

- **Command System**  
  Enables undo/redo support.

## Plugin System

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

### Responsibilities

A plugin can:

- Handle input events
- Start interactions
- Render UI layers
- Modify graph state

## Interaction System

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

Interaction lifecycle:

```text
Start -> Update -> End / Replace
```

```rust
pub enum InteractionResult {
    Continue,
    End,
    Replace(Box<dyn Interaction>),
}
```

## Command System (Undo / Redo)

Implements the Command Pattern:

```rust
pub trait Command {
    fn execute(&mut self, ctx: &mut CommandContext);
    fn undo(&mut self, ctx: &mut CommandContext);
}
```

Built-in features:

- Undo / Redo stacks
- Composite commands
- Easy integration via `PluginContext`

```rust
ctx.execute_command(MyCommand { ... });
```

## Node Rendering

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

## Graph Model

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

Creating nodes (builder API):

```rust
graph.create_node("math.add")
    .position(100.0, 100.0)
    .input()
    .output()
    .build();
```

## Performance

Designed to scale to large graphs:

- Viewport-based rendering (virtualization)
- Layered rendering system
- Interaction-aware rendering (degraded mode during drag)
- Ready for spatial indexing

## Design Principles

- Separation of data and interaction
- Plugins over hardcoded behavior
- Explicit state transitions
- Performance-first rendering
- Composable architecture
