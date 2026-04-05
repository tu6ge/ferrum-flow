# ferrum-flow-sync-plugin

A **ferrum-flow** collaboration plugin built on [Yjs / yrs](https://github.com/y-crdt/y-crdt): syncs the canvas graph over WebSocket peers and adds simple remote cursors via **awareness**.

## Features

- **Y.Doc** stores nodes, ports, edges, and `node_order`, mapped bidirectionally with ferrum-flow’s `Graph`.
- Uses the **y-sync** protocol (`DefaultProtocol`) to exchange updates with the server; local edits are distinguished from remote updates through `UndoManager` origins.
- **Awareness**: cursor positions in **canvas (world) coordinates**; leaving the canvas clears local presence so peers stop drawing your cursor.
- Bundled **`Assets`** (`rust-embed`): remote cursors use `assets/cursor.png`; examples load it with `Application::with_assets(Assets)`.

## Requirements

- Rust 2024
- Workspace crate **`ferrum-flow`** (GPUI canvas), plus **GPUI**, **yrs** (with the `sync` feature), **tokio-tungstenite**, etc.

## Quick try

1. **Run the sample server** (default `127.0.0.1:9001`):

   ```bash
   cargo run -p ferrum-flow-sync-plugin --example server
   ```

2. **Run the client** (same protocol as the server; register assets so the cursor image resolves):

   ```bash
   cargo run -p ferrum-flow-sync-plugin --example basic
   ```

   Open several `basic` windows to test together. To seed the Y.Doc from a **locally built graph on the first client**, set (matches the example):

   ```bash
   IS_INIT=1 cargo run -p ferrum-flow-sync-plugin --example basic
   ```

   Other clients can start with an empty graph; they will sync to the same document state from the server.

## Integrating in your app

1. Call **`Application::with_assets(Assets)`** (or supply your own `AssetSource` that serves `cursor.png`).
2. Register the plugin on `FlowCanvas::builder(...).sync_plugin(...)`:

   ```rust
   use ferrum_flow_sync_plugin::{Assets, YrsSyncPlugin};

   // ...
   .sync_plugin(YrsSyncPlugin::new(initial_graph, "ws://127.0.0.1:9001"))
   ```

- **`YrsSyncPlugin::new(graph, ws_url)`**
  - `graph`: if non-empty at `setup`, it is written into the Y.Doc as initial content (in multi-client setups, decide whether only one peer should supply that seed).
  - `ws_url`: WebSocket URL; must match server relay semantics for y-sync (**document updates** and **awareness**).

## Public API

| Item | Description |
| --- | --- |
| `YrsSyncPlugin` | Implements `ferrum_flow::SyncPlugin` |
| `Assets` | GPUI `AssetSource` embedding `cursor.png` |

## Server notes

The `server` example uses a shared **Y.Doc + Awareness** for the handshake and broadcasts document deltas and **awareness** frames to other clients. In production you can use any **y-sync**-compatible relay or backend, but it must forward **Sync updates** and **Awareness** messages; otherwise remote cursors will not work.

## License

Apache-2.0 (same as crate metadata).
