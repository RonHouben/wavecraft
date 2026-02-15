//! Procedural macro for generating complete plugin implementations from DSL.
//!
//! Simplified API (0.9.0): Only requires `name` and `signal` properties.
//! Vendor and URL metadata are automatically derived from Cargo.toml.
//! Plugin email is not exposed in the DSL and defaults to an empty string.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Expr, Ident, LitStr, Path, Result, Token, Type,
    parse::Parser,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
};

/// Input structure for `wavecraft_plugin!` macro.
struct PluginDef {
    name: LitStr,
    signal: Expr,
    /// Optional crate path for nih-plug integration crate (default: `::wavecraft`).
    /// Use `crate: my_name` only if you've renamed the wavecraft dependency in Cargo.toml.
    krate: Option<Path>,
}

impl Parse for PluginDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name = None;
        let mut signal = None;
        let mut krate = None;

        // Parse key-value pairs
        while !input.is_empty() {
            // Handle `crate` keyword specially (it's a Rust keyword)
            if input.peek(Token![crate]) {
                input.parse::<Token![crate]>()?;
                input.parse::<Token![:]>()?;
                krate = Some(input.parse()?);

                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                }
                continue;
            }

            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "name" => name = Some(input.parse()?),
                "signal" => signal = Some(input.parse()?),
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown field: `{}`\n\
                             \n\
                             The wavecraft_plugin! macro only accepts:\n\
                             - name: \"Plugin Name\" (required)\n\
                             - signal: SignalChain![...] (required)\n\
                             - crate: custom_name (optional, for Cargo renames)",
                            key
                        ),
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let signal = signal.ok_or_else(|| {
            input.error(
                "missing required field: `signal`\n\
                 \n\
                 The signal field defines your DSP processing chain.\n\
                 \n\
                 Example:\n\
                 wavecraft_plugin! {\n\
                     name: \"My Plugin\",\n\
                     signal: SignalChain![MyGain],\n\
                 }\n\
                 \n\
                 For multiple processors:\n\
                 signal: SignalChain![InputGain, Filter, OutputGain]",
            )
        })?;

        // Validate signal is wrapped in SignalChain! (not a bare identifier)
        if let Expr::Path(ref path) = signal
            && path.path.segments.len() == 1
        {
            let span = signal.span();
            return Err(syn::Error::new(
                span,
                "signal must use `SignalChain!` wrapper.\n\
                 \n\
                 Did you mean:\n\
                 signal: SignalChain![YourProcessor]\n\
                 \n\
                 Or for multiple processors:\n\
                 signal: SignalChain![A, B, C]\n\
                 \n\
                 Note: Bare processor names are not allowed. Always wrap in SignalChain![]",
            ));
        }

        Ok(PluginDef {
            name: name.ok_or_else(|| {
                input.error(
                    "missing required field: `name`\n\
                     \n\
                     Example:\n\
                     wavecraft_plugin! {\n\
                         name: \"My Plugin\",\n\
                         signal: SignalChain![MyGain],\n\
                     }",
                )
            })?,
            signal,
            // Default krate to ::wavecraft if not specified
            krate: krate.or_else(|| Some(syn::parse_quote!(::wavecraft))),
        })
    }
}

/// Generate a deterministic VST3 ID from package name and plugin name.
///
/// Uses CARGO_PKG_NAME instead of vendor for:
/// - Stability: package names are canonical identifiers
/// - Uniqueness: enforced by Cargo/crates.io conventions
/// - Simplicity: one less parameter to manage
fn generate_vst3_id(name: &str) -> proc_macro2::TokenStream {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let package_name = env!("CARGO_PKG_NAME");

    let mut hasher = DefaultHasher::new();
    format!("{}{}", package_name, name).hash(&mut hasher);
    let hash = hasher.finish();

    // Convert hash to 16 bytes
    let bytes: [u8; 16] = [
        (hash >> 56) as u8,
        (hash >> 48) as u8,
        (hash >> 40) as u8,
        (hash >> 32) as u8,
        (hash >> 24) as u8,
        (hash >> 16) as u8,
        (hash >> 8) as u8,
        hash as u8,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0, // Padding
    ];

    quote! { [#(#bytes),*] }
}

fn to_snake_case_identifier(name: &str) -> String {
    name.chars()
        .enumerate()
        .flat_map(|(i, c)| {
            if c.is_uppercase() && i > 0 {
                vec!['_', c.to_ascii_lowercase()]
            } else {
                vec![c.to_ascii_lowercase()]
            }
        })
        .collect()
}

fn type_prefix(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|segment| to_snake_case_identifier(&segment.ident.to_string()))
            .unwrap_or_else(|| to_snake_case_identifier(&quote::quote!(#ty).to_string())),
        _ => to_snake_case_identifier(&quote::quote!(#ty).to_string()),
    }
}

fn parse_signal_chain_processors(signal: &Expr) -> Result<Vec<Type>> {
    let expr_macro = match signal {
        Expr::Macro(expr_macro) => expr_macro,
        _ => {
            return Err(syn::Error::new(
                signal.span(),
                "signal must use SignalChain![...] macro syntax",
            ));
        }
    };

    let is_signal_chain = expr_macro
        .mac
        .path
        .segments
        .last()
        .map(|segment| segment.ident == "SignalChain")
        .unwrap_or(false);

    if !is_signal_chain {
        return Err(syn::Error::new(
            expr_macro.mac.path.span(),
            "signal must use SignalChain![...]",
        ));
    }

    let parser = Punctuated::<Type, Token![,]>::parse_terminated;
    let processors = parser.parse2(expr_macro.mac.tokens.clone())?;

    if processors.is_empty() {
        return Err(syn::Error::new(
            expr_macro.mac.tokens.span(),
            "SignalChain! must contain at least one processor type",
        ));
    }

    Ok(processors.into_iter().collect())
}

pub fn wavecraft_plugin_impl(input: TokenStream) -> TokenStream {
    let plugin_def = parse_macro_input!(input as PluginDef);

    let name = &plugin_def.name;
    let signal_type = &plugin_def.signal;

    let signal_processors = match parse_signal_chain_processors(signal_type) {
        Ok(processors) => processors,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    // Default krate to ::wavecraft if not specified (should already be set by Parse)
    let krate = plugin_def
        .krate
        .unwrap_or_else(|| syn::parse_quote!(::wavecraft));

    let processor_param_mappings = signal_processors.iter().map(|processor_type| {
        let id_prefix = type_prefix(processor_type);
        quote! {
            {
                let specs = <<#processor_type as #krate::Processor>::Params as #krate::ProcessorParams>::param_specs();
                params.extend(specs
                    .iter()
                    .map(|spec| #krate::__internal::param_spec_to_info(spec, #id_prefix)));
            }
        }
    });

    // Derive metadata from Cargo environment variables
    let vendor = {
        let authors = env!("CARGO_PKG_AUTHORS");
        authors.split(',').next().unwrap_or("Unknown").trim()
    };

    let url = {
        let homepage = env!("CARGO_PKG_HOMEPAGE");
        if homepage.is_empty() {
            env!("CARGO_PKG_REPOSITORY")
        } else {
            homepage
        }
    };

    let vst3_id = generate_vst3_id(&name.value());

    // CLAP ID now uses package name for consistency
    let clap_id = {
        let package_name = env!("CARGO_PKG_NAME");
        format!("com.{}", package_name.replace('-', "_"))
    };

    // Phase 6 Steps 6.1-6.6 Complete:
    // - Input parsing ✓
    // - Plugin struct generation ✓
    // - Params struct with runtime parameter discovery ✓
    // - Plugin trait impl with audio processing ✓
    // - Format impls & exports ✓
    // - Error messages (compile-time validation) ✓
    //
    // 0.9.0 Updates:
    // - Simplified API (name + signal only) ✓
    // - Vendor/URL derived from Cargo.toml ✓
    // - Email is internal default (not exposed in DSL) ✓
    // - VST3/CLAP IDs use package name ✓
    // - Signal validation (requires SignalChain!) ✓

    let expanded = quote! {
        // Use the signal expression as the processor type
        type __ProcessorType = #signal_type;

        // Compile-time validation: ensure the processor type implements required traits
        const _: () = {
            fn assert_processor_traits<T>()
            where
                T: #krate::Processor + ::std::default::Default + ::std::marker::Send + 'static,
                T::Params: #krate::ProcessorParams + ::std::default::Default + ::std::marker::Send + ::std::marker::Sync + 'static,
            {
            }

            fn validate() {
                assert_processor_traits::<__ProcessorType>();
            }
        };

        /// Generated plugin struct.
        pub struct __WavecraftPlugin {
            params: ::std::sync::Arc<__WavecraftParams>,
            processor: __ProcessorType,
            meter_producer: #krate::MeterProducer,
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            meter_consumer: ::std::sync::Mutex<::std::option::Option<#krate::MeterConsumer>>,
        }

        /// Generated params struct.
        ///
        /// This struct bridges wavecraft-dsp ProcessorParams to nih-plug's Params trait.
        /// Parameters are discovered at runtime from the processor's param_specs().
        pub struct __WavecraftParams {
            // Store parameters as a vector for dynamic discovery
            params: ::std::vec::Vec<#krate::__nih::FloatParam>,
        }

        impl __WavecraftParams {
            fn from_processor_specs() -> Self
            where
                <__ProcessorType as #krate::Processor>::Params: #krate::ProcessorParams,
            {
                let specs = <<__ProcessorType as #krate::Processor>::Params as #krate::ProcessorParams>::param_specs();

                let params = specs
                    .iter()
                    .map(|spec| {
                        use #krate::ParamRange;

                        let range = match &spec.range {
                            ParamRange::Linear { min, max } => {
                                #krate::__nih::FloatRange::Linear {
                                    min: *min as f32,
                                    max: *max as f32,
                                }
                            }
                            ParamRange::Skewed { min, max, factor } => {
                                #krate::__nih::FloatRange::Skewed {
                                    min: *min as f32,
                                    max: *max as f32,
                                    factor: *factor as f32,
                                }
                            }
                            ParamRange::Stepped { min, max } => {
                                // Convert stepped range to linear for now
                                #krate::__nih::FloatRange::Linear {
                                    min: *min as f32,
                                    max: *max as f32,
                                }
                            }
                        };

                        #krate::__nih::FloatParam::new(
                            spec.name,
                            spec.default as f32,
                            range,
                        )
                        .with_unit(spec.unit)
                    })
                    .collect();

                Self { params }
            }
        }

        impl ::std::default::Default for __WavecraftParams {
            fn default() -> Self {
                Self::from_processor_specs()
            }
        }

        // Manual Params implementation (can't use derive due to Vec)
        unsafe impl #krate::__nih::Params for __WavecraftParams {
            fn param_map(&self) -> ::std::vec::Vec<(
                ::std::string::String,
                #krate::__nih::ParamPtr,
                ::std::string::String,
            )> {
                use #krate::__nih::Param; // Import trait for as_ptr()

                self.params
                    .iter()
                    .enumerate()
                    .map(|(idx, param)| {
                        let id = format!("param_{}", idx);
                        let group = ::std::string::String::new();
                        (id, param.as_ptr(), group)
                    })
                    .collect()
            }
        }

        impl ::std::default::Default for __WavecraftPlugin {
            fn default() -> Self {
                let (meter_producer, _meter_consumer) =
                    #krate::create_meter_channel(64);
                Self {
                    params: ::std::sync::Arc::new(__WavecraftParams::default()),
                    processor: <__ProcessorType as ::std::default::Default>::default(),
                    meter_producer,
                    #[cfg(any(target_os = "macos", target_os = "windows"))]
                    meter_consumer: ::std::sync::Mutex::new(::std::option::Option::Some(_meter_consumer)),
                }
            }
        }

        impl #krate::__nih::Plugin for __WavecraftPlugin {
            const NAME: &'static str = #name;
            const VENDOR: &'static str = #vendor;
            const URL: &'static str = #url;
            const EMAIL: &'static str = "";
            const VERSION: &'static str = env!("CARGO_PKG_VERSION");

            const AUDIO_IO_LAYOUTS: &'static [#krate::__nih::AudioIOLayout] = &[
                #krate::__nih::AudioIOLayout {
                    main_input_channels: ::std::num::NonZeroU32::new(2),
                    main_output_channels: ::std::num::NonZeroU32::new(2),
                    ..#krate::__nih::AudioIOLayout::const_default()
                }
            ];

            const MIDI_INPUT: #krate::__nih::MidiConfig =
                #krate::__nih::MidiConfig::None;
            const MIDI_OUTPUT: #krate::__nih::MidiConfig =
                #krate::__nih::MidiConfig::None;

            type SysExMessage = ();
            type BackgroundTask = ();

            fn params(&self) -> ::std::sync::Arc<dyn #krate::__nih::Params> {
                self.params.clone()
            }

            fn editor(
                &mut self,
                _async_executor: #krate::__nih::AsyncExecutor<Self>,
            ) -> ::std::option::Option<::std::boxed::Box<dyn #krate::__nih::Editor>> {
                #[cfg(any(target_os = "macos", target_os = "windows"))]
                {
                    let meter_consumer = self
                        .meter_consumer
                        .lock()
                        .expect("meter_consumer mutex poisoned - previous panic in editor thread")
                        .take();
                    #krate::editor::create_webview_editor(
                        self.params.clone(),
                        meter_consumer,
                        800,
                        600,
                    )
                }

                #[cfg(not(any(target_os = "macos", target_os = "windows")))]
                {
                    None
                }
            }

            fn initialize(
                &mut self,
                _audio_io_layout: &#krate::__nih::AudioIOLayout,
                _buffer_config: &#krate::__nih::BufferConfig,
                _context: &mut impl #krate::__nih::InitContext<Self>,
            ) -> bool {
                true
            }

            fn reset(&mut self) {}

            fn process(
                &mut self,
                buffer: &mut #krate::__nih::Buffer,
                _aux: &mut #krate::__nih::AuxiliaryBuffers,
                _context: &mut impl #krate::__nih::ProcessContext<Self>,
            ) -> #krate::__nih::ProcessStatus {
                let num_samples = buffer.samples();
                let channels = buffer.channels();

                // Build processor params from current parameter values
                let processor_params = self.build_processor_params();

                // Convert nih-plug buffer to wavecraft-dsp format
                // We process sample-by-sample to properly handle the buffer format
                for sample_idx in 0..num_samples {
                    // Create a temporary buffer for this sample
                    let mut sample_buffers: ::std::vec::Vec<::std::vec::Vec<f32>> =
                        (0..channels).map(|ch| {
                            vec![buffer.as_slice()[ch][sample_idx]]
                        }).collect();

                    let mut sample_ptrs: ::std::vec::Vec<&mut [f32]> =
                        sample_buffers.iter_mut().map(|v| &mut v[..]).collect();

                    let transport = #krate::Transport::default();

                    // Import Processor trait for process() method
                    use #krate::Processor as _;
                    self.processor.process(&mut sample_ptrs, &transport, &processor_params);

                    // Write processed samples back
                    for (ch, sample_buf) in sample_buffers.iter().enumerate() {
                        if let Some(channel) = buffer.as_slice().get(ch) {
                            if sample_idx < channel.len() {
                                // SAFETY JUSTIFICATION:
                                //
                                // 1. Exclusive Access: nih-plug's process() callback guarantees exclusive
                                //    buffer access (no concurrent reads/writes from other threads).
                                //
                                // 2. Bounds Check: The `if` guards above ensure:
                                //    - `ch` is a valid channel index (within buffer.channels())
                                //    - `sample_idx < channel.len()` (within channel sample count)
                                //
                                // 3. Pointer Validity:
                                //    - `channel.as_ptr()` is from nih-plug's Buffer allocation (valid)
                                //    - `.add(sample_idx)` offset is within bounds (checked above)
                                //    - Pointer is properly aligned (f32 alignment guaranteed by host)
                                //
                                // 4. Write Safety:
                                //    - f32 is Copy (atomic write, no drop required)
                                //    - No aliasing: Buffer<'a> lifetime ensures no other refs exist
                                //    - Host expects in-place modification (plugin contract)
                                //
                                // 5. Why unsafe is necessary:
                                //    nih-plug's Buffer API only provides immutable refs (as_slice()).
                                //    However, the plugin contract allows (and expects) in-place writes.
                                //    Casting *const → *mut is sound because we have exclusive access
                                //    during process() callback (guaranteed by DAW host).
                                unsafe {
                                    let channel_ptr = channel.as_ptr() as *mut f32;
                                    *channel_ptr.add(sample_idx) = sample_buf[0];
                                }
                            }
                        }
                    }
                }

                // Update meters (simplified - just measure output peaks)
                let mut peak_left = 0.0_f32;
                let mut peak_right = 0.0_f32;

                if channels >= 1 {
                    peak_left = buffer.as_slice()[0].iter().map(|&s| s.abs()).fold(0.0, f32::max);
                }
                if channels >= 2 {
                    peak_right = buffer.as_slice()[1].iter().map(|&s| s.abs()).fold(0.0, f32::max);
                }

                let frame = #krate::MeterFrame {
                    peak_l: peak_left,
                    peak_r: peak_right,
                    // Simplified RMS estimation: peak * 1/√2 (0.707)
                    // This is exact for sine waves but approximate for other signals.
                    // Acceptable for basic metering; for accurate RMS, use sliding window average.
                    rms_l: peak_left * 0.707,
                    rms_r: peak_right * 0.707,
                    // Note: Timestamp not implemented for DSL plugins.
                    // Basic metering doesn't require sample-accurate timing.
                    // For advanced metering with sample position tracking,
                    // implement Plugin trait directly and use context.transport().
                    timestamp: 0,
                };

                let _ = self.meter_producer.push(frame);

                #krate::__nih::ProcessStatus::Normal
            }
        }

        impl __WavecraftPlugin {
            /// Build processor parameters from current nih-plug parameter values.
            ///
            /// # Known Limitation
            ///
            /// Full bidirectional parameter sync between nih-plug and processor
            /// params is not yet implemented. For now this initializes processor
            /// params from `ProcessorParams::from_param_defaults()`.
            ///
            /// For custom parameter behavior, implement the `Plugin` trait directly
            /// instead of using the `wavecraft_plugin!` macro.
            fn build_processor_params(&self) -> <__ProcessorType as #krate::Processor>::Params {
                <<__ProcessorType as #krate::Processor>::Params as #krate::ProcessorParams>::from_param_defaults()
            }
        }

        impl #krate::__nih::ClapPlugin for __WavecraftPlugin {
            const CLAP_ID: &'static str = #clap_id;
            const CLAP_DESCRIPTION: Option<&'static str> = None;
            const CLAP_MANUAL_URL: Option<&'static str> = None;
            const CLAP_SUPPORT_URL: Option<&'static str> = None;
            const CLAP_FEATURES: &'static [#krate::__nih::ClapFeature] = &[
                #krate::__nih::ClapFeature::AudioEffect,
                #krate::__nih::ClapFeature::Stereo,
            ];
        }

        impl #krate::__nih::Vst3Plugin for __WavecraftPlugin {
            const VST3_CLASS_ID: [u8; 16] = #vst3_id;
            const VST3_SUBCATEGORIES: &'static [#krate::__nih::Vst3SubCategory] = &[
                #krate::__nih::Vst3SubCategory::Fx,
            ];
        }

        // When building with `_param-discovery` feature, skip nih-plug's
        // static initializers (VST3/CLAP factory registration) to prevent
        // dlopen from hanging on macOS audio subsystem services.
        #[cfg(not(feature = "_param-discovery"))]
        #krate::__nih::nih_export_clap!(__WavecraftPlugin);
        #[cfg(not(feature = "_param-discovery"))]
        #krate::__nih::nih_export_vst3!(__WavecraftPlugin);

        // ================================================================
        // FFI Exports for Parameter Discovery (used by `wavecraft start`)
        // ================================================================

        /// Returns JSON-serialized parameter specifications.
        ///
        /// This function is called by the `wavecraft start` command to discover
        /// the plugin's parameters without loading it into a DAW.
        ///
        /// # Safety
        /// The returned pointer must be freed with `wavecraft_free_string`.
        #[unsafe(no_mangle)]
        pub extern "C" fn wavecraft_get_params_json() -> *mut ::std::ffi::c_char {
            let mut params: ::std::vec::Vec<#krate::__internal::ParameterInfo> = ::std::vec::Vec::new();
            #(#processor_param_mappings)*

            // Serialize parameter list to JSON for FFI export
            // Fallback to "[]" on serialization error (should never happen for ParameterInfo)
            let json = #krate::__internal::serde_json::to_string(&params)
                .unwrap_or_else(|_| "[]".to_string());

            // Convert to C string for FFI
            // Returns null pointer if JSON contains embedded null bytes (invalid UTF-8)
            // Caller (JS bridge) must check for null before dereferencing
            ::std::ffi::CString::new(json)
                .map(|s| s.into_raw())
                .unwrap_or(::std::ptr::null_mut())
        }

        /// Frees a string returned by `wavecraft_get_params_json`.
        ///
        /// # Safety
        /// The pointer must have been returned by `wavecraft_get_params_json`.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn wavecraft_free_string(ptr: *mut ::std::ffi::c_char) {
            if !ptr.is_null() {
                let _ = ::std::ffi::CString::from_raw(ptr);
            }
        }

        // ================================================================
        // FFI Exports for Dev Audio Processing (used by `wavecraft start`)
        // ================================================================

        /// Returns a C-ABI vtable for creating and driving the plugin's audio
        /// processor from the CLI dev server (in-process audio via FFI).
        ///
        /// Each inner function is wrapped in `catch_unwind` to prevent panics
        /// from unwinding across the FFI boundary.
        #[unsafe(no_mangle)]
        pub extern "C" fn wavecraft_dev_create_processor() -> #krate::__internal::DevProcessorVTable {
            use ::std::ffi::c_void;

            type __P = __ProcessorType;
            type __Params = <__P as #krate::Processor>::Params;

            extern "C" fn create() -> *mut c_void {
                let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    let processor = ::std::boxed::Box::new(<__P as ::std::default::Default>::default());
                    ::std::boxed::Box::into_raw(processor) as *mut c_void
                }));
                match result {
                    Ok(ptr) => ptr,
                    Err(_) => ::std::ptr::null_mut(),
                }
            }

            extern "C" fn process(
                instance: *mut c_void,
                channels: *mut *mut f32,
                num_channels: u32,
                num_samples: u32,
            ) {
                let _ = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    if instance.is_null() || channels.is_null() || num_channels == 0 || num_samples == 0 {
                        return;
                    }
                    let processor = unsafe { &mut *(instance as *mut __P) };
                    let num_ch = num_channels as usize;
                    let num_samp = num_samples as usize;

                    // Build &mut [&mut [f32]] from raw pointers.
                    // SAFETY: Caller guarantees valid pointers and bounds (documented in vtable).
                    // Note: Vec allocation is acceptable here — this runs in the dev audio server
                    // context (not a DAW audio thread), where latency requirements are softer.
                    let mut channel_slices: ::std::vec::Vec<&mut [f32]> = (0..num_ch)
                        .map(|ch| unsafe {
                            let ptr = *channels.add(ch);
                            ::std::slice::from_raw_parts_mut(ptr, num_samp)
                        })
                        .collect();

                    let transport = #krate::Transport::default();
                    let params = <__Params as #krate::ProcessorParams>::from_param_defaults();

                    #krate::Processor::process(processor, &mut channel_slices, &transport, &params);
                }));
                // If panic occurred, audio buffer is left unmodified (pass-through)
            }

            extern "C" fn set_sample_rate(instance: *mut c_void, sample_rate: f32) {
                let _ = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    if instance.is_null() {
                        return;
                    }
                    let processor = unsafe { &mut *(instance as *mut __P) };
                    #krate::Processor::set_sample_rate(processor, sample_rate);
                }));
            }

            extern "C" fn reset(instance: *mut c_void) {
                let _ = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    if instance.is_null() {
                        return;
                    }
                    let processor = unsafe { &mut *(instance as *mut __P) };
                    #krate::Processor::reset(processor);
                }));
            }

            extern "C" fn drop_fn(instance: *mut c_void) {
                let _ = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    if !instance.is_null() {
                        let _ = unsafe { ::std::boxed::Box::from_raw(instance as *mut __P) };
                    }
                }));
            }

            #krate::__internal::DevProcessorVTable {
                version: #krate::__internal::DEV_PROCESSOR_VTABLE_VERSION,
                create,
                process,
                set_sample_rate,
                reset,
                drop: drop_fn,
            }
        }
    };

    TokenStream::from(expanded)
}
