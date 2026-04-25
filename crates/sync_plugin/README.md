# ferrum-flow-sync-plugin

A **ferrum-flow** collaboration plugin built on [Yjs / yrs](https://github.com/y-crdt/y-crdt): syncs the canvas graph over WebSocket peers and adds simple remote cursors via **awareness**.

## Features

- **Y.Doc** stores nodes, ports, edges, and `node_order`, mapped bidirectionally with ferrum-flow’s `Graph`.
- Uses the **y-sync** protocol (`DefaultProtocol`) to exchange updates with the server; local edits are distinguished from remote updates through `UndoManager` origins.
- **Awareness**: cursor positions in **canvas (world) coordinates**; leaving the canvas clears local presence so peers stop drawing your cursor.
- Optional Cargo feature **`dev-ws-relay`**: compiles the sample WebSocket relay (`run_dev_ws_relay`, `tokio/net`). Off by default so the library surface stays client-focused.

## Requirements

- Rust 2024
- Workspace crate **`ferrum-flow`** (GPUI canvas), plus **GPUI**, **yrs** (with the `sync` feature), **tokio-tungstenite**, etc.

## Quick try

### One command — two GPUI clients (recommended)

Starts the sample WebSocket relay on `127.0.0.1:9001` in a background thread, then opens **two** windows that share the same Yjs document (graph + awareness cursors):

```bash
cargo run -p ferrum-flow-sync-plugin --features dev-ws-relay --example collab_two_windows
```

- **Left** window (`client A`): by default supplies the same **seed graph** as `IS_INIT=1` in `basic` (three nodes), so you immediately see content to edit and can watch the **right** window catch up.
- **Right** window (`client B`): starts from an **empty** `YrsSyncPlugin` seed and syncs from the relay.

To start **both** windows from an empty graph (same idea as running `basic` without `IS_INIT`):

```bash
IS_INIT=0 cargo run -p ferrum-flow-sync-plugin --features dev-ws-relay --example collab_two_windows
```

### Manual two-process workflow

1. **Relay only** (default `127.0.0.1:9001`):

   ```bash
   cargo run -p ferrum-flow-sync-plugin --features dev-ws-relay --example server
   ```

2. **One client per process** (assets are registered so remote cursor `cursor.png` resolves):

   ```bash
   cargo run -p ferrum-flow-sync-plugin --example basic
   ```

   Open several `basic` windows to test together. To seed the Y.Doc from a **locally built graph on that client**, set:

   ```bash
   IS_INIT=1 cargo run -p ferrum-flow-sync-plugin --example basic
   ```

   Other clients can start with an empty graph; they will sync to the same document state from the server.

## Integrating in your app

Register the plugin on `FlowCanvas::builder(...).sync_plugin(...)`:

   ```rust
   use ferrum_flow_sync_plugin::YrsSyncPlugin;

   // ...
   .sync_plugin(YrsSyncPlugin::new(initial_graph, "ws://127.0.0.1:9001"))
   ```

- **`YrsSyncPlugin::new(graph, ws_url)`**
  - `graph`: if non-empty at `setup`, it is written into the Y.Doc as initial content (in multi-client setups, decide whether only one peer should supply that seed).
  - `ws_url`: WebSocket URL; must match server relay semantics for y-sync (**document updates** and **awareness**).

WebSocket connect uses retries with exponential backoff (defaults: 10 attempts, 1–30s backoff). After the socket closes, the client waits `reconnect_delay` and starts a new connect phase with the same limits. Customize via **`YrsSyncPlugin::with_ws_config`** and **`WsSyncConfig`** (`max_connect_retries`: use `u32::MAX` for unlimited attempts per phase).

## Public API

| Item | Description |
| --- | --- |
| `YrsSyncPlugin` | Implements `ferrum_flow::SyncPlugin` |
| `WsSyncConfig` | Retry / backoff / reconnect delay for the sync WebSocket thread |
| `run_dev_ws_relay` | *(feature `dev-ws-relay`)* Async entry point for the **sample** y-sync WebSocket relay (`127.0.0.1:9001`); used by `examples/server` and `examples/collab_two_windows` |

## Server notes

The `server` example (and `run_dev_ws_relay` when feature **`dev-ws-relay`** is enabled) use a shared **Y.Doc + Awareness** for the handshake and broadcast document deltas and **awareness** frames to other clients. In production you can use any **y-sync**-compatible relay or backend, but it must forward **Sync updates** and **Awareness** messages; otherwise remote cursors will not work.

## License

Apache-2.0 (same as crate metadata).
