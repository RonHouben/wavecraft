//! IPC bridge for WebView ↔ Rust communication
//!
//! This crate provides the message handling layer between the React UI
//! (running in a WebView) and Rust application logic. It implements
//! JSON-RPC 2.0 style request/response handling with typed message contracts.
//!
//! # Architecture
//!
//! - **ParameterHost** trait: Abstracts parameter storage (desktop POC, plugin, etc.)
//! - **IpcHandler**: Dispatches JSON-RPC requests to appropriate handlers
//! - **BridgeError**: Typed error handling with conversion to IPC error codes
//!
//! # Example
//!
//! ```rust,no_run
//! use wavecraft_bridge::{IpcHandler, ParameterHost, BridgeError};
//! use wavecraft_protocol::{MeterFrame, ParameterInfo};
//!
//! // Implement ParameterHost for your application state
//! struct MyApp;
//!
//! impl ParameterHost for MyApp {
//!     fn get_parameter(&self, id: &str) -> Option<ParameterInfo> {
//!         // Implementation
//!         None
//!     }
//!     
//!     fn set_parameter(&self, id: &str, value: f32) -> Result<(), BridgeError> {
//!         // Implementation
//!         Ok(())
//!     }
//!     
//!     fn get_all_parameters(&self) -> Vec<ParameterInfo> {
//!         // Implementation
//!         vec![]
//!     }
//!
//!     fn get_meter_frame(&self) -> Option<MeterFrame> {
//!         // Implementation
//!         None
//!     }
//!
//!     fn request_resize(&self, _width: u32, _height: u32) -> bool {
//!         // Implementation
//!         false
//!     }
//! }
//!
//! // Create handler
//! let handler = IpcHandler::new(MyApp);
//!
//! // Handle incoming JSON from WebView
//! let request_json = r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#;
//! let response_json = handler.handle_json(request_json);
//! ```

pub mod error;
pub mod handler;
pub mod host;
pub mod in_memory_host;
pub mod plugin_loader;

// Re-export key types for convenience
pub use error::BridgeError;
pub use handler::IpcHandler;
pub use host::ParameterHost;
pub use in_memory_host::{InMemoryParameterHost, MeterProvider};
pub use plugin_loader::{PluginLoaderError, PluginParamLoader};

// Re-export protocol types used in bridge API
pub use wavecraft_protocol::{
    GetAllParametersResult, GetParameterParams, GetParameterResult, IpcError, IpcNotification,
    IpcRequest, IpcResponse, ParameterChangedNotification, ParameterInfo, ParameterType, RequestId,
    SetParameterParams, SetParameterResult,
};
