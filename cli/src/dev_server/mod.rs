//! Embedded development server for `wavecraft start`
//!
//! This module provides the infrastructure for running a WebSocket development
//! server that communicates with browser-based UIs. Unlike the wavecraft-dev-server crate,
//! this implementation dynamically loads user plugins to discover their parameters.
//!
//! # Architecture
//!
//! ```text
//! ┌────────────────────┐     FFI      ┌──────────────┐
//! │ PluginParamLoader  │─────────────►│ User Plugin  │
//! │ (wavecraft-bridge) │◄─────────────│  (dylib)     │
//! └─────────┬──────────┘  JSON params └──────────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │  DevServerHost  │  implements ParameterHost
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │  IpcHandler<H>  │  from wavecraft-bridge
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │  WsServer<H>    │  from wavecraft-dev-server crate
//! └────────┬────────┘
//!          │
//!          ▼
//!     Browser UI
//! ```

mod host;
mod rebuild;
mod session;
mod watcher;

pub use host::DevServerHost;
pub use session::DevSession;
pub use wavecraft_bridge::PluginParamLoader as PluginLoader;
