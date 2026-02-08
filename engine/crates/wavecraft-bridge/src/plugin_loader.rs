//! Plugin parameter loader using libloading for FFI.

use libloading::{Library, Symbol};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::Path;
use wavecraft_protocol::{DEV_PROCESSOR_VTABLE_VERSION, DevProcessorVTable, ParameterInfo};

/// Errors that can occur during plugin loading.
#[derive(Debug)]
pub enum PluginLoaderError {
    /// Failed to load the dynamic library.
    LibraryLoad(libloading::Error),
    /// Failed to find a required FFI symbol.
    SymbolNotFound(String),
    /// FFI function returned a null pointer.
    NullPointer(&'static str),
    /// Failed to parse the parameter JSON.
    JsonParse(serde_json::Error),
    /// The returned string was not valid UTF-8.
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

type GetParamsJsonFn = unsafe extern "C" fn() -> *mut c_char;
type FreeStringFn = unsafe extern "C" fn(*mut c_char);
type DevProcessorVTableFn = unsafe extern "C" fn() -> DevProcessorVTable;

/// Plugin loader that extracts parameter metadata and optionally the
/// dev audio processor vtable via FFI.
///
/// # Safety
///
/// This struct manages the lifecycle of a dynamically loaded library.
/// The library must remain loaded while any data from it is in use.
/// The `PluginParamLoader` ensures proper cleanup via Drop.
///
/// # Drop Ordering
///
/// `_library` is listed first and is therefore dropped last (Rust drops
/// fields in declaration order). Any `FfiProcessor` created from the
/// vtable must be dropped *before* this loader to keep function pointers
/// valid.
pub struct PluginParamLoader {
    _library: Library,
    parameters: Vec<ParameterInfo>,
    /// Audio processor vtable (None if symbol not found — older plugins).
    dev_processor_vtable: Option<DevProcessorVTable>,
}

impl PluginParamLoader {
    /// Load a plugin from the given path and extract its parameter metadata.
    pub fn load<P: AsRef<Path>>(dylib_path: P) -> Result<Self, PluginLoaderError> {
        let library =
            unsafe { Library::new(dylib_path.as_ref()) }.map_err(PluginLoaderError::LibraryLoad)?;

        let get_params_json: Symbol<GetParamsJsonFn> = unsafe {
            library.get(b"wavecraft_get_params_json\0").map_err(|e| {
                PluginLoaderError::SymbolNotFound(format!("wavecraft_get_params_json: {}", e))
            })?
        };

        let free_string: Symbol<FreeStringFn> = unsafe {
            library.get(b"wavecraft_free_string\0").map_err(|e| {
                PluginLoaderError::SymbolNotFound(format!("wavecraft_free_string: {}", e))
            })?
        };

        let params = unsafe {
            let json_ptr = get_params_json();
            if json_ptr.is_null() {
                return Err(PluginLoaderError::NullPointer("wavecraft_get_params_json"));
            }

            let c_str = CStr::from_ptr(json_ptr);
            let json_str = c_str.to_str().map_err(PluginLoaderError::InvalidUtf8)?;

            let params: Vec<ParameterInfo> =
                serde_json::from_str(json_str).map_err(PluginLoaderError::JsonParse)?;

            free_string(json_ptr);

            params
        };

        // Try to load audio processor vtable (optional — graceful fallback)
        let dev_processor_vtable = Self::try_load_processor_vtable(&library);

        Ok(Self {
            _library: library,
            parameters: params,
            dev_processor_vtable,
        })
    }

    /// Get the loaded parameter information.
    pub fn parameters(&self) -> &[ParameterInfo] {
        &self.parameters
    }

    /// Get a parameter by ID.
    #[allow(dead_code)]
    pub fn get_parameter(&self, id: &str) -> Option<&ParameterInfo> {
        self.parameters.iter().find(|p| p.id == id)
    }

    /// Returns the dev processor vtable if the plugin exports it.
    ///
    /// Returns `None` if:
    /// - The plugin was built with an older SDK that doesn't export the symbol
    /// - The vtable version doesn't match (ABI incompatibility)
    pub fn dev_processor_vtable(&self) -> Option<&DevProcessorVTable> {
        self.dev_processor_vtable.as_ref()
    }

    /// Attempt to load the dev audio processor vtable from the library.
    ///
    /// This is a best-effort operation: if the symbol isn't found or the
    /// version doesn't match, we return `None` and continue without audio.
    fn try_load_processor_vtable(library: &Library) -> Option<DevProcessorVTable> {
        let symbol: Symbol<DevProcessorVTableFn> =
            unsafe { library.get(b"wavecraft_dev_create_processor\0").ok()? };

        let vtable = unsafe { symbol() };

        // Version check — catch ABI mismatches early
        if vtable.version != DEV_PROCESSOR_VTABLE_VERSION {
            tracing::warn!(
                "Plugin dev-audio vtable version {} != expected {}. \
                 Audio processing disabled. Update your SDK dependency.",
                vtable.version,
                DEV_PROCESSOR_VTABLE_VERSION
            );
            return None;
        }

        Some(vtable)
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
