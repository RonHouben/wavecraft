//! Plugin parameter loader using libloading for FFI.

use libloading::{Library, Symbol};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::Path;
use wavecraft_protocol::{
    DEV_PROCESSOR_VTABLE_VERSION, DevProcessorVTable, ParameterInfo, ProcessorInfo,
};

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
    /// Failed to read a file (e.g., sidecar JSON cache).
    FileRead(std::io::Error),
    /// Vtable ABI version mismatch.
    VtableVersionMismatch { found: u32, expected: u32 },
}

impl std::fmt::Display for PluginLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LibraryLoad(e) => write!(f, "Failed to load plugin library: {}", e),
            Self::SymbolNotFound(name) => write!(f, "Symbol not found: {}", name),
            Self::NullPointer(func) => write!(f, "FFI function {} returned null", func),
            Self::JsonParse(e) => write!(f, "Failed to parse JSON payload: {}", e),
            Self::InvalidUtf8(e) => write!(f, "Invalid UTF-8 in FFI response: {}", e),
            Self::FileRead(e) => write!(f, "Failed to read file: {}", e),
            Self::VtableVersionMismatch { found, expected } => write!(
                f,
                "Dev processor vtable version mismatch: found {}, expected {}. Rebuild the plugin with the current SDK.",
                found, expected
            ),
        }
    }
}

impl std::error::Error for PluginLoaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::LibraryLoad(e) => Some(e),
            Self::JsonParse(e) => Some(e),
            Self::InvalidUtf8(e) => Some(e),
            Self::FileRead(e) => Some(e),
            _ => None,
        }
    }
}

type GetParamsJsonFn = unsafe extern "C" fn() -> *mut c_char;
type GetProcessorsJsonFn = unsafe extern "C" fn() -> *mut c_char;
type FreeStringFn = unsafe extern "C" fn(*mut c_char);
type DevProcessorVTableFn = unsafe extern "C" fn() -> DevProcessorVTable;

struct FfiStringGuard {
    ptr: *mut c_char,
    free_string: FreeStringFn,
}

impl FfiStringGuard {
    fn new(ptr: *mut c_char, free_string: FreeStringFn) -> Self {
        Self { ptr, free_string }
    }
}

impl Drop for FfiStringGuard {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // SAFETY: `ptr` was returned by the corresponding FFI getter and
            // must be released by the paired `free_string` function exactly once.
            unsafe { (self.free_string)(self.ptr) };
        }
    }
}

fn parse_json_from_ffi<T>(
    json_ptr: *mut c_char,
    function_name: &'static str,
) -> Result<T, PluginLoaderError>
where
    T: serde::de::DeserializeOwned,
{
    if json_ptr.is_null() {
        return Err(PluginLoaderError::NullPointer(function_name));
    }

    // SAFETY: pointer was returned by FFI symbol using CString::into_raw.
    let c_str = unsafe { CStr::from_ptr(json_ptr) };
    let json_str = c_str.to_str().map_err(PluginLoaderError::InvalidUtf8)?;
    serde_json::from_str(json_str).map_err(PluginLoaderError::JsonParse)
}

fn load_json_via_ffi<T>(
    get_json: unsafe extern "C" fn() -> *mut c_char,
    free_string: unsafe extern "C" fn(*mut c_char),
    function_name: &'static str,
) -> Result<T, PluginLoaderError>
where
    T: serde::de::DeserializeOwned,
{
    // SAFETY: Both function pointers come from validated dynamic-library symbols
    // with matching ABI. The returned pointer is always released by
    // `FfiStringGuard`, including parse/utf8 error paths.
    unsafe {
        let json_ptr = get_json();
        let guard = FfiStringGuard::new(json_ptr, free_string);
        let parsed = parse_json_from_ffi::<T>(guard.ptr, function_name)?;
        Ok(parsed)
    }
}

/// Plugin loader that extracts parameter metadata and the
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
/// Struct fields are dropped in declaration order (first declared = first
/// dropped). `_library` is the **last** field so it is dropped last,
/// ensuring the dynamic library remains loaded while `parameters` and
/// `dev_processor_vtable` are cleaned up.
///
/// **External invariant:** any `FfiProcessor` created from the vtable
/// must be dropped *before* this loader to keep vtable function pointers
/// valid. The caller must ensure this via local variable declaration
/// order (later declared = first dropped).
pub struct PluginParamLoader {
    parameters: Vec<ParameterInfo>,
    processors: Vec<ProcessorInfo>,
    /// Audio processor vtable for in-process dev audio.
    dev_processor_vtable: DevProcessorVTable,
    /// Dynamic library handle — must be the last field so it is dropped
    /// last, after all data that may reference library symbols.
    _library: Library,
}

impl PluginParamLoader {
    /// Load parameters from a sidecar JSON file (bypasses FFI/dlopen).
    ///
    /// Used by `wavecraft start` to read cached parameter metadata without
    /// loading the plugin dylib (which triggers nih-plug static initializers).
    pub fn load_params_from_file<P: AsRef<Path>>(
        json_path: P,
    ) -> Result<Vec<ParameterInfo>, PluginLoaderError> {
        let contents =
            std::fs::read_to_string(json_path.as_ref()).map_err(PluginLoaderError::FileRead)?;
        let params: Vec<ParameterInfo> =
            serde_json::from_str(&contents).map_err(PluginLoaderError::JsonParse)?;
        Ok(params)
    }

    /// Load a plugin from the given path and extract its parameter metadata.
    pub fn load<P: AsRef<Path>>(dylib_path: P) -> Result<Self, PluginLoaderError> {
        // SAFETY: Loading a dynamic library is inherently unsafe. The caller
        // must pass a valid path to a cdylib built with the wavecraft SDK.
        // The library is kept alive for the lifetime of this struct.
        let library =
            unsafe { Library::new(dylib_path.as_ref()) }.map_err(PluginLoaderError::LibraryLoad)?;

        // SAFETY: The symbol `wavecraft_get_params_json` is an `extern "C"`
        // function generated by the `wavecraft_plugin!` macro with the expected
        // signature (`GetParamsJsonFn`). The library is valid and loaded.
        let get_params_json: Symbol<GetParamsJsonFn> = unsafe {
            library.get(b"wavecraft_get_params_json\0").map_err(|e| {
                PluginLoaderError::SymbolNotFound(format!("wavecraft_get_params_json: {}", e))
            })?
        };

        // SAFETY: The symbol `wavecraft_free_string` is an `extern "C"`
        // function generated by the `wavecraft_plugin!` macro with the expected
        // signature (`FreeStringFn`). The library is valid and loaded.
        let free_string: Symbol<FreeStringFn> = unsafe {
            library.get(b"wavecraft_free_string\0").map_err(|e| {
                PluginLoaderError::SymbolNotFound(format!("wavecraft_free_string: {}", e))
            })?
        };

        // SAFETY: Symbol is generated by wavecraft_plugin! and uses GetProcessorsJsonFn ABI.
        let get_processors_json: Symbol<GetProcessorsJsonFn> = unsafe {
            library
                .get(b"wavecraft_get_processors_json\0")
                .map_err(|e| {
                    PluginLoaderError::SymbolNotFound(format!(
                        "wavecraft_get_processors_json: {}",
                        e
                    ))
                })?
        };

        // SAFETY: All FFI calls target `extern "C"` functions generated by
        // the `wavecraft_plugin!` macro:
        // - `get_params_json()` returns a heap-allocated C string (or null).
        //   We check for null before dereferencing.
        // - `CStr::from_ptr()` is safe because the pointer is non-null and
        //   the string is NUL-terminated (allocated by `CString::into_raw`).
        // - `free_string()` deallocates the string originally created by
        //   `CString::into_raw` — matching allocator, called exactly once.
        let params = load_json_via_ffi::<Vec<ParameterInfo>>(
            *get_params_json,
            *free_string,
            "wavecraft_get_params_json",
        )?;

        let processors = load_json_via_ffi::<Vec<ProcessorInfo>>(
            *get_processors_json,
            *free_string,
            "wavecraft_get_processors_json",
        )?;

        // Load and validate audio processor vtable (required for current SDK dev mode)
        let dev_processor_vtable = Self::load_processor_vtable(&library)?;

        Ok(Self {
            parameters: params,
            processors,
            dev_processor_vtable,
            _library: library,
        })
    }

    /// Load parameters only (skip processor vtable loading).
    ///
    /// Used by hot-reload where only parameter metadata is needed.
    /// Avoids potential side effects from vtable construction.
    pub fn load_params_only<P: AsRef<Path>>(
        dylib_path: P,
    ) -> Result<Vec<ParameterInfo>, PluginLoaderError> {
        // SAFETY: Loading a dynamic library is inherently unsafe. The caller
        // must pass a valid path to a cdylib built with the wavecraft SDK.
        let library =
            unsafe { Library::new(dylib_path.as_ref()) }.map_err(PluginLoaderError::LibraryLoad)?;

        // SAFETY: The symbol `wavecraft_get_params_json` is an `extern "C"`
        // function generated by the `wavecraft_plugin!` macro with the expected
        // signature (`GetParamsJsonFn`). The library is valid and loaded.
        let get_params_json: Symbol<GetParamsJsonFn> = unsafe {
            library.get(b"wavecraft_get_params_json\0").map_err(|e| {
                PluginLoaderError::SymbolNotFound(format!("wavecraft_get_params_json: {}", e))
            })?
        };

        // SAFETY: The symbol `wavecraft_free_string` is an `extern "C"`
        // function generated by the `wavecraft_plugin!` macro with the expected
        // signature (`FreeStringFn`). The library is valid and loaded.
        let free_string: Symbol<FreeStringFn> = unsafe {
            library.get(b"wavecraft_free_string\0").map_err(|e| {
                PluginLoaderError::SymbolNotFound(format!("wavecraft_free_string: {}", e))
            })?
        };

        // SAFETY: See `load()` for detailed rationale on FFI safety.
        let params = load_json_via_ffi::<Vec<ParameterInfo>>(
            *get_params_json,
            *free_string,
            "wavecraft_get_params_json",
        )?;

        Ok(params)
    }

    /// Load processor metadata only (skip processor vtable loading).
    pub fn load_processors_only<P: AsRef<Path>>(
        dylib_path: P,
    ) -> Result<Vec<ProcessorInfo>, PluginLoaderError> {
        // SAFETY: See load_params_only() rationale.
        let library =
            unsafe { Library::new(dylib_path.as_ref()) }.map_err(PluginLoaderError::LibraryLoad)?;

        // SAFETY: Symbol is generated by wavecraft_plugin! and uses GetProcessorsJsonFn ABI.
        let get_processors_json: Symbol<GetProcessorsJsonFn> = unsafe {
            library
                .get(b"wavecraft_get_processors_json\0")
                .map_err(|e| {
                    PluginLoaderError::SymbolNotFound(format!(
                        "wavecraft_get_processors_json: {}",
                        e
                    ))
                })?
        };

        // SAFETY: Symbol generated by macro with expected FreeStringFn ABI.
        let free_string: Symbol<FreeStringFn> = unsafe {
            library.get(b"wavecraft_free_string\0").map_err(|e| {
                PluginLoaderError::SymbolNotFound(format!("wavecraft_free_string: {}", e))
            })?
        };

        // SAFETY: See load() for FFI safety rationale.
        let processors = load_json_via_ffi::<Vec<ProcessorInfo>>(
            *get_processors_json,
            *free_string,
            "wavecraft_get_processors_json",
        )?;

        Ok(processors)
    }

    /// Get the loaded parameter information.
    pub fn parameters(&self) -> &[ParameterInfo] {
        &self.parameters
    }

    /// Get the loaded processor metadata.
    pub fn processors(&self) -> &[ProcessorInfo] {
        &self.processors
    }

    /// Get a parameter by ID.
    #[allow(dead_code)]
    pub fn get_parameter(&self, id: &str) -> Option<&ParameterInfo> {
        self.parameters.iter().find(|p| p.id == id)
    }

    /// Returns the validated dev processor vtable.
    pub fn dev_processor_vtable(&self) -> &DevProcessorVTable {
        &self.dev_processor_vtable
    }

    /// Load and validate the dev audio processor vtable from the library.
    fn load_processor_vtable(library: &Library) -> Result<DevProcessorVTable, PluginLoaderError> {
        // SAFETY: `library` is a valid loaded Library handle. The symbol name
        // matches the `extern "C"` function generated by the `wavecraft_plugin!`
        // macro with identical signature (`DevProcessorVTableFn`).
        let symbol: Symbol<DevProcessorVTableFn> = unsafe {
            library
                .get(b"wavecraft_dev_create_processor\0")
                .map_err(|e| {
                    PluginLoaderError::SymbolNotFound(format!(
                        "wavecraft_dev_create_processor: {}",
                        e
                    ))
                })?
        };

        // SAFETY: The symbol points to a valid `extern "C"` function generated
        // by the `wavecraft_plugin!` macro with matching ABI and return type.
        // The function constructs and returns a `DevProcessorVTable` by value
        // (no heap allocation, no side effects beyond initialization).
        // The Library remains loaded for the duration of this call.
        let vtable = unsafe { symbol() };

        // Version check — v2-only contract.
        if vtable.version != DEV_PROCESSOR_VTABLE_VERSION {
            return Err(PluginLoaderError::VtableVersionMismatch {
                found: vtable.version,
                expected: DEV_PROCESSOR_VTABLE_VERSION,
            });
        }

        Ok(vtable)
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

    #[test]
    fn test_file_read_error() {
        let err = PluginLoaderError::FileRead(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        assert!(err.to_string().contains("Failed to read file"));
    }

    #[test]
    fn test_vtable_version_mismatch_error_display() {
        let err = PluginLoaderError::VtableVersionMismatch {
            found: 1,
            expected: 2,
        };
        assert!(err.to_string().contains("version mismatch"));
        assert!(err.to_string().contains("found 1"));
        assert!(err.to_string().contains("expected 2"));
    }

    #[test]
    fn test_load_params_from_file() {
        use wavecraft_protocol::ParameterType;

        let dir = std::env::temp_dir().join("wavecraft_test_sidecar");
        let _ = std::fs::create_dir_all(&dir);
        let json_path = dir.join("wavecraft-params.json");

        let params = vec![ParameterInfo {
            id: "gain".to_string(),
            name: "Gain".to_string(),
            param_type: ParameterType::Float,
            value: 0.5,
            default: 0.5,
            min: 0.0,
            max: 1.0,
            unit: Some("dB".to_string()),
            group: Some("Main".to_string()),
            variants: None,
        }];

        let json = serde_json::to_string_pretty(&params).unwrap();
        std::fs::write(&json_path, &json).unwrap();

        let loaded = PluginParamLoader::load_params_from_file(&json_path).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "gain");
        assert_eq!(loaded[0].name, "Gain");
        assert!((loaded[0].default - 0.5).abs() < f32::EPSILON);

        // Cleanup
        let _ = std::fs::remove_file(&json_path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn test_load_params_from_file_not_found() {
        let result = PluginParamLoader::load_params_from_file("/nonexistent/path.json");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, PluginLoaderError::FileRead(_)));
    }

    #[test]
    fn test_load_params_from_file_invalid_json() {
        let dir = std::env::temp_dir().join("wavecraft_test_bad_json");
        let _ = std::fs::create_dir_all(&dir);
        let json_path = dir.join("bad-params.json");

        std::fs::write(&json_path, "not valid json").unwrap();

        let result = PluginParamLoader::load_params_from_file(&json_path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PluginLoaderError::JsonParse(_)
        ));

        // Cleanup
        let _ = std::fs::remove_file(&json_path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn test_parse_processors_json() {
        let json = r#"[{"id":"oscillator"},{"id":"output_gain"}]"#;
        let processors: Vec<ProcessorInfo> =
            serde_json::from_str(json).expect("json should deserialize");

        assert_eq!(processors.len(), 2);
        assert_eq!(processors[0].id, "oscillator");
        assert_eq!(processors[1].id, "output_gain");
    }

    #[test]
    fn test_parse_processors_json_invalid() {
        let json = "not valid json";
        let parsed: Result<Vec<ProcessorInfo>, _> = serde_json::from_str(json);
        assert!(parsed.is_err());
    }
}
