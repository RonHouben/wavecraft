//! Plugin loader using libloading for FFI
//!
//! This module provides safe wrappers around the FFI interface exposed by
//! wavecraft plugins. It loads the compiled plugin dylib and extracts
//! parameter metadata via the `wavecraft_get_params_json` FFI function.

use libloading::{Library, Symbol};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::Path;
use wavecraft_protocol::ParameterInfo;

/// Errors that can occur during plugin loading
#[derive(Debug)]
pub enum PluginLoaderError {
    /// Failed to load the dynamic library
    LibraryLoad(libloading::Error),
    /// Failed to find a required FFI symbol
    SymbolNotFound(String),
    /// FFI function returned a null pointer
    NullPointer(&'static str),
    /// Failed to parse the parameter JSON
    JsonParse(serde_json::Error),
    /// The returned string was not valid UTF-8
    InvalidUtf8(std::str::Utf8Error),
}

impl std::fmt::Display for PluginLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LibraryLoad(e) => write!(f, "Failed to load plugin library: {}", e),
            Self::SymbolNotFound(name) => write!(f, "Symbol not found: {}", name),
            Self::NullPointer(func) => write!(f, "FFI function {} returned null", func),
            Self::JsonParse(e) => write!(f, "Failed to parse parameter JSON: {}", e),
            Self::InvalidUtf8(e) => write!(f, "Invalid UTF-8 in FFI response: {}", e),
        }
    }
}

impl std::error::Error for PluginLoaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::LibraryLoad(e) => Some(e),
            Self::JsonParse(e) => Some(e),
            Self::InvalidUtf8(e) => Some(e),
            _ => None,
        }
    }
}

/// FFI function type signatures
type GetParamsJsonFn = unsafe extern "C" fn() -> *const c_char;
type FreeStringFn = unsafe extern "C" fn(*const c_char);

/// Plugin loader that extracts parameter metadata via FFI
///
/// # Safety
///
/// This struct manages the lifecycle of a dynamically loaded library.
/// The library must remain loaded while any data from it is in use.
/// The `PluginLoader` ensures proper cleanup via Drop.
pub struct PluginLoader {
    /// The loaded library (kept alive to prevent unloading)
    _library: Library,
    /// Cached parameter information
    parameters: Vec<ParameterInfo>,
}

impl PluginLoader {
    /// Load a plugin from the given path and extract its parameter metadata.
    ///
    /// # Arguments
    ///
    /// * `dylib_path` - Path to the compiled plugin dylib
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The library fails to load
    /// - Required FFI symbols are not found
    /// - The JSON response is invalid
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let loader = PluginLoader::load("target/release/libmy_plugin.dylib")?;
    /// for param in loader.parameters() {
    ///     println!("{}: {}", param.id, param.name);
    /// }
    /// ```
    pub fn load<P: AsRef<Path>>(dylib_path: P) -> Result<Self, PluginLoaderError> {
        // Load the dynamic library
        // SAFETY: We trust the plugin author to provide a valid dylib.
        // The library is kept alive in the struct to prevent symbol invalidation.
        let library =
            unsafe { Library::new(dylib_path.as_ref()) }.map_err(PluginLoaderError::LibraryLoad)?;

        // Get the FFI symbols
        let get_params_json: Symbol<GetParamsJsonFn> = unsafe {
            library
                .get(b"wavecraft_get_params_json\0")
                .map_err(|e| PluginLoaderError::SymbolNotFound(format!("wavecraft_get_params_json: {}", e)))?
        };

        let free_string: Symbol<FreeStringFn> = unsafe {
            library
                .get(b"wavecraft_free_string\0")
                .map_err(|e| PluginLoaderError::SymbolNotFound(format!("wavecraft_free_string: {}", e)))?
        };

        // Call the FFI function to get parameter JSON
        let params = unsafe {
            let json_ptr = get_params_json();
            if json_ptr.is_null() {
                return Err(PluginLoaderError::NullPointer("wavecraft_get_params_json"));
            }

            // Convert to Rust string (borrow the C string)
            let c_str = CStr::from_ptr(json_ptr);
            let json_str = c_str.to_str().map_err(PluginLoaderError::InvalidUtf8)?;

            // Parse the JSON
            let params: Vec<ParameterInfo> =
                serde_json::from_str(json_str).map_err(PluginLoaderError::JsonParse)?;

            // Free the C string
            free_string(json_ptr);

            params
        };

        Ok(Self {
            _library: library,
            parameters: params,
        })
    }

    /// Get the loaded parameter information
    pub fn parameters(&self) -> &[ParameterInfo] {
        &self.parameters
    }

    /// Get a parameter by ID
    #[allow(dead_code)] // Reserved for future use
    pub fn get_parameter(&self, id: &str) -> Option<&ParameterInfo> {
        self.parameters.iter().find(|p| p.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = PluginLoaderError::SymbolNotFound("test_symbol".to_string());
        assert!(err.to_string().contains("test_symbol"));
    }

    #[test]
    fn test_null_pointer_error() {
        let err = PluginLoaderError::NullPointer("wavecraft_get_params_json");
        assert!(err.to_string().contains("null"));
    }
}
