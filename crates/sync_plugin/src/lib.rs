#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

#[cfg(feature = "dev-ws-relay")]
mod dev_ws_relay;
mod plugin;
mod server;

#[cfg(feature = "dev-ws-relay")]
pub use dev_ws_relay::run_dev_ws_relay;
pub use plugin::{PresenceConfig, YrsSyncPlugin};
pub use server::WsSyncConfig;
