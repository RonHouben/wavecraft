use quote::quote;

pub(super) struct CodegenInput<'a> {
    pub(super) name: &'a syn::LitStr,
    pub(super) signal_type: &'a syn::Expr,
    pub(super) krate: &'a syn::Path,
    pub(super) runtime_param_blocks: &'a [proc_macro2::TokenStream],
    pub(super) processor_param_mappings: &'a [proc_macro2::TokenStream],
    pub(super) processor_info_entries: &'a [proc_macro2::TokenStream],
    pub(super) vendor: &'a str,
    pub(super) url: &'a str,
    pub(super) vst3_id: &'a proc_macro2::TokenStream,
    pub(super) clap_id: &'a str,
}

pub(super) fn generate_plugin_code(input: CodegenInput<'_>) -> proc_macro2::TokenStream {
    let CodegenInput {
        name,
        signal_type,
        krate,
        runtime_param_blocks,
        processor_param_mappings,
        processor_info_entries,
        vendor,
        url,
        vst3_id,
        clap_id,
    } = input;

    quote! {
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
            oscilloscope_tap: #krate::OscilloscopeTap,
            meter_producer: #krate::MeterProducer,
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            meter_consumer: ::std::sync::Mutex<::std::option::Option<#krate::MeterConsumer>>,
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            oscilloscope_consumer: ::std::sync::Mutex<::std::option::Option<#krate::OscilloscopeFrameConsumer>>,
        }

        /// Generated params struct.
        ///
        /// This struct bridges wavecraft-dsp ProcessorParams to nih-plug's Params trait.
        /// Parameters are discovered at runtime from the processor's param_specs().
        enum __WavecraftRuntimeParam {
            Float(#krate::__nih::FloatParam),
            Int(#krate::__nih::IntParam),
        }

        impl __WavecraftRuntimeParam {
            fn as_ptr(&self) -> #krate::__nih::ParamPtr {
                use #krate::__nih::Param;

                match self {
                    Self::Float(param) => param.as_ptr(),
                    Self::Int(param) => param.as_ptr(),
                }
            }

            fn modulated_plain_value(&self) -> f32 {
                let ptr = self.as_ptr();
                // SAFETY: ParamPtr originates from `self` and remains valid for this call.
                unsafe { ptr.modulated_plain_value() }
            }
        }

        pub struct __WavecraftParams {
            // Store parameters as a vector for dynamic discovery
            params: ::std::vec::Vec<__WavecraftRuntimeParam>,
            // Runtime IDs aligned with FFI-generated contract IDs (e.g. oscillator_enabled)
            ids: ::std::vec::Vec<::std::string::String>,
            // Optional parameter group names (empty string when none)
            groups: ::std::vec::Vec<::std::string::String>,
        }

        impl __WavecraftParams {
            fn from_processor_specs() -> Self
            where
                <__ProcessorType as #krate::Processor>::Params: #krate::ProcessorParams,
            {
                use #krate::ParamRange;

                let mut params = ::std::vec::Vec::new();
                let mut ids = ::std::vec::Vec::new();
                let mut groups = ::std::vec::Vec::new();

                #(#runtime_param_blocks)*

                Self { params, ids, groups }
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
                self.params
                    .iter()
                    .zip(self.ids.iter())
                    .zip(self.groups.iter())
                    .map(|((param, id), group)| {
                        (id.clone(), param.as_ptr(), group.clone())
                    })
                    .collect()
            }
        }

        impl ::std::default::Default for __WavecraftPlugin {
            fn default() -> Self {
                let (meter_producer, _meter_consumer) =
                    #krate::create_meter_channel(64);
                let (oscilloscope_producer, _oscilloscope_consumer) =
                    #krate::create_oscilloscope_channel(8);
                Self {
                    params: ::std::sync::Arc::new(__WavecraftParams::default()),
                    processor: <__ProcessorType as ::std::default::Default>::default(),
                    oscilloscope_tap: #krate::OscilloscopeTap::with_output(oscilloscope_producer),
                    meter_producer,
                    #[cfg(any(target_os = "macos", target_os = "windows"))]
                    meter_consumer: ::std::sync::Mutex::new(::std::option::Option::Some(_meter_consumer)),
                    #[cfg(any(target_os = "macos", target_os = "windows"))]
                    oscilloscope_consumer: ::std::sync::Mutex::new(::std::option::Option::Some(_oscilloscope_consumer)),
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
                    let oscilloscope_consumer = self
                        .oscilloscope_consumer
                        .lock()
                        .expect("oscilloscope_consumer mutex poisoned - previous panic in editor thread")
                        .take();
                    #krate::editor::create_webview_editor(
                        self.params.clone(),
                        meter_consumer,
                        oscilloscope_consumer,
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
                #krate::Processor::set_sample_rate(
                    &mut self.processor,
                    _buffer_config.sample_rate,
                );
                self.oscilloscope_tap
                    .set_sample_rate_hz(_buffer_config.sample_rate);
                true
            }

            fn reset(&mut self) {
                #krate::Processor::reset(&mut self.processor);
                #krate::Processor::reset(&mut self.oscilloscope_tap);
            }

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

                if channels >= 1 {
                    let left_snapshot: ::std::vec::Vec<f32> = buffer.as_slice()[0].to_vec();
                    let right_snapshot: ::std::vec::Vec<f32> = if channels >= 2 {
                        buffer.as_slice()[1].to_vec()
                    } else {
                        left_snapshot.clone()
                    };

                    self.oscilloscope_tap
                        .capture_stereo(&left_snapshot, &right_snapshot);
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
            /// params is implemented by applying plain host values in the same
            /// order as `ProcessorParams::param_specs()`.
            fn build_processor_params(&self) -> <__ProcessorType as #krate::Processor>::Params {
                let mut params =
                    <<__ProcessorType as #krate::Processor>::Params as #krate::ProcessorParams>::from_param_defaults();
                let values: ::std::vec::Vec<f32> = self
                    .params
                    .params
                    .iter()
                    .map(|param| param.modulated_plain_value())
                    .collect();

                <<__ProcessorType as #krate::Processor>::Params as #krate::ProcessorParams>::apply_plain_values(
                    &mut params,
                    &values,
                );

                params
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

        /// Returns JSON-serialized processor metadata.
        ///
        /// This function is called by `wavecraft start` to discover processor IDs
        /// from the plugin signal chain at dev/build time.
        ///
        /// # Safety
        /// The returned pointer must be freed with `wavecraft_free_string`.
        #[unsafe(no_mangle)]
        pub extern "C" fn wavecraft_get_processors_json() -> *mut ::std::ffi::c_char {
            let processors: ::std::vec::Vec<#krate::__internal::ProcessorInfo> = vec![
                #(#processor_info_entries),*
            ];

            let json = #krate::__internal::serde_json::to_string(&processors)
                .unwrap_or_else(|_| "[]".to_string());

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
            struct __DevProcessorInstance {
                processor: __P,
                params: __Params,
            }

            extern "C" fn create() -> *mut c_void {
                let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    let instance = ::std::boxed::Box::new(__DevProcessorInstance {
                        processor: <__P as ::std::default::Default>::default(),
                        params: <__Params as #krate::ProcessorParams>::from_param_defaults(),
                    });
                    ::std::boxed::Box::into_raw(instance) as *mut c_void
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
                    let instance = unsafe { &mut *(instance as *mut __DevProcessorInstance) };
                    let num_ch = num_channels as usize;
                    let num_samp = num_samples as usize;

                    let transport = #krate::Transport::default();

                    // Build stack-local channel slices for the current block.
                    // Wavecraft dev audio currently targets mono/stereo.
                    match num_ch {
                        1 => {
                            let ch0_ptr = unsafe { *channels.add(0) };
                            if ch0_ptr.is_null() {
                                return;
                            }

                            let ch0 = unsafe { ::std::slice::from_raw_parts_mut(ch0_ptr, num_samp) };
                            let mut channel_slices: [&mut [f32]; 1] = [ch0];
                            #krate::Processor::process(
                                &mut instance.processor,
                                &mut channel_slices,
                                &transport,
                                &instance.params,
                            );
                        }
                        2 => {
                            let ch0_ptr = unsafe { *channels.add(0) };
                            let ch1_ptr = unsafe { *channels.add(1) };
                            if ch0_ptr.is_null() || ch1_ptr.is_null() {
                                return;
                            }

                            let ch0 = unsafe { ::std::slice::from_raw_parts_mut(ch0_ptr, num_samp) };
                            let ch1 = unsafe { ::std::slice::from_raw_parts_mut(ch1_ptr, num_samp) };
                            let mut channel_slices: [&mut [f32]; 2] = [ch0, ch1];
                            #krate::Processor::process(
                                &mut instance.processor,
                                &mut channel_slices,
                                &transport,
                                &instance.params,
                            );
                        }
                        _ => {
                            // Unsupported channel topology in dev-FFI path.
                        }
                    }
                }));
                // If panic occurred, audio buffer is left unmodified (pass-through)
            }

            unsafe extern "C" fn apply_plain_values(
                instance: *mut c_void,
                values_ptr: *const f32,
                len: usize,
            ) {
                let _ = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    if instance.is_null() {
                        return;
                    }

                    if values_ptr.is_null() && len != 0 {
                        return;
                    }

                    let instance = unsafe { &mut *(instance as *mut __DevProcessorInstance) };
                    let values: &[f32] = if len == 0 {
                        &[]
                    } else {
                        unsafe { ::std::slice::from_raw_parts(values_ptr, len) }
                    };

                    <__Params as #krate::ProcessorParams>::apply_plain_values(
                        &mut instance.params,
                        values,
                    );
                }));
            }

            extern "C" fn set_sample_rate(instance: *mut c_void, sample_rate: f32) {
                let _ = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    if instance.is_null() {
                        return;
                    }
                    let instance = unsafe { &mut *(instance as *mut __DevProcessorInstance) };
                    #krate::Processor::set_sample_rate(&mut instance.processor, sample_rate);
                }));
            }

            extern "C" fn reset(instance: *mut c_void) {
                let _ = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    if instance.is_null() {
                        return;
                    }
                    let instance = unsafe { &mut *(instance as *mut __DevProcessorInstance) };
                    #krate::Processor::reset(&mut instance.processor);
                }));
            }

            extern "C" fn drop_fn(instance: *mut c_void) {
                let _ = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    if !instance.is_null() {
                        let _ = unsafe {
                            ::std::boxed::Box::from_raw(instance as *mut __DevProcessorInstance)
                        };
                    }
                }));
            }

            #krate::__internal::DevProcessorVTable {
                version: #krate::__internal::DEV_PROCESSOR_VTABLE_VERSION,
                create,
                process,
                apply_plain_values,
                set_sample_rate,
                reset,
                drop: drop_fn,
            }
        }
    }
}
