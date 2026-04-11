//! Standalone WebSocket relay (same as embedded in `collab_two_windows`).

#[tokio::main]
async fn main() {
    ferrum_flow_sync_plugin::run_dev_ws_relay().await;
}
