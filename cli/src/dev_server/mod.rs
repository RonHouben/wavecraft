//! Embedded development server for `wavecraft start`
//!
//! This module provides the infrastructure for running a WebSocket development
//! server that communicates with browser-based UIs. Unlike the standalone crate,
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
//! │  WsServer<H>    │  from standalone crate
//! └────────┬────────┘
//!          │
//!          ▼
//!     Browser UI
//! ```

mod host;

pub use host::DevServerHost;
pub use wavecraft_bridge::PluginParamLoader as PluginLoader;
