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

## Crate Layout

- [`crates/core`](./crates/core/) - FerrumFlow editor core (graph model, plugins, interaction, rendering contracts)
- [`crates/sync_plugin`](./crates/sync_plugin/) - Yrs-based collaboration plugin (sync, awareness, multi-user workflows)

## Usage

```toml
[dependencies]
gpui = "0.2.2"
serde_json = "1.0"
ferrum-flow = { git = "https://github.com/tu6ge/ferrum-flow", branch = "master" }
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

Architecture details have moved to:

- [docs/architecture.md](./docs/architecture.md)

This includes core concepts, plugin/interaction/command model, node rendering contract, graph model, and performance/design principles.

### Feature parity & gap analysis (React Flow -> FerrumFlow)

We want an explicit, maintained view of **supported vs partial vs missing** relative to React Flow’s documented capabilities (nodes, edges, handles, selection, keyboard, minimap, controls, snapping, grouping/subflows, accessibility, etc.):

- [x] **Audit** — Walk the React Flow feature list and map each item to FerrumFlow (plugin, core graph, or N/A by design).
- [x] **Gap list** — For every row, mark *done*, *partial*, *missing*, or *different by design* (short rationale).
- [x] **Surface in this README** — Add a compact table or bullet matrix here (or link to `docs/react-flow-parity.md` if it grows large).
- [x] change `port_type` of Port struct to custom enum type.

Current mapping snapshot (as of this README update):

| React Flow capability | FerrumFlow mapping | Status | Notes |
| --- | --- | --- | --- |
| Canvas / viewport (zoom, pan, fit, controls) | Plugin: `ViewportPlugin`, `ZoomControlsPlugin`, `FitAllPlugin`; Core: `Viewport` | done/partial | Core zoom/pan done; behavior parity tuning ongoing. |
| Nodes (custom node UI, drag, multi-select drag) | Plugin: `NodePlugin`, `NodeInteractionPlugin`, `SelectionPlugin`; Core graph: `Node` | done |  |
| Handles/ports (typed endpoints, connectability rules) | Plugin: `PortInteractionPlugin`; Core graph: `Port`, `PortType` | done/partial | Typed ports and validator path exist; full RF handle-option parity not complete. |
| Edges (rendering, interaction, visibility) | Plugin: `EdgePlugin`; Core graph: `Edge` | done/partial |  |
| Selection (box select, additive selection, select-all viewport) | Plugin: `SelectionPlugin`, `SelectAllViewportPlugin`; Core selected sets | done |  |
| Delete / keyboard editing | Plugin: `DeletePlugin`, `HistoryPlugin`; Command system | done |  |
| Undo/redo command stack | Core: `canvas::undo`, `Command`, `LocalHistory`; Plugin: `HistoryPlugin` | done |  |
| Clipboard copy/paste | Plugin: `ClipboardPlugin` | done/partial | Core flow present; parity on all RF clipboard scenarios not fully audited. |
| Minimap | Plugin: `MinimapPlugin` | done |  |
| Context menu / node creation UX | Plugin: `ContextMenuPlugin` | done/partial |  |
| Alignment / layout helpers / focus selection | Plugin: `AlignPlugin`, `FocusSelectionPlugin` | done/partial |  |
| Background grid / viewport frame / chrome | Plugin: `BackgroundPlugin`, `ViewportFramePlugin` | done |  |
| Subflows / parent-child grouping | N/A by design (currently) | missing/different by design | No explicit parent node/group graph model yet. |
| Accessibility API parity (ARIA, keyboard nav parity) | N/A (currently) | partial/missing |  |
| Devtools-style state inspector parity | N/A (currently) | missing |  |
| Collaboration / multi-user awareness | Plugin: `sync_plugin` (Yrs + awareness) | different by design (done) | Implemented via plugin architecture, not RF-style built-in surface. |

### Beyond React Flow (FerrumFlow-specific strengths)

FerrumFlow is not only targeting parity. Some capabilities are intentionally stronger or more explicit than typical React Flow usage patterns:

- **Command interop guarantees (execute / undo / to_ops consistency)**  
  The `Command` pipeline is designed so local execution, undo/redo, and operation replay can be tested for equivalence (`command_interop`), reducing divergence bugs.

- **Plugin-first architecture across behavior and rendering**  
  Core editor capabilities (selection, viewport, minimap, clipboard, context menu, alignment, collaboration) are modeled as plugins instead of hardcoded monolith behavior.

- **CRDT collaboration as a first-class extension path**  
  `sync_plugin` integrates Yrs awareness + graph ops, giving a concrete multi-user architecture beyond single-user canvas editing.

- **Clear separation of graph model, interaction lifecycle, and rendering**  
  `Graph` / `Viewport` / `Interaction` / `PluginContext` boundaries are explicit, making it easier to evolve editor features without coupling everything into UI callbacks.

- **Rust-native integration potential**  
  FerrumFlow can align with native Rust systems (state machines, persistence, sync backends, tooling) without requiring a browser-first runtime assumption.

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
