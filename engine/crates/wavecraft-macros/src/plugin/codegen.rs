use quote::quote;
use syn::Type;

pub(super) struct CodegenInput<'a> {
    pub(super) name: &'a syn::LitStr,
    pub(super) processors: &'a [Type],
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
        processors,
        krate,
        runtime_param_blocks,
        processor_param_mappings,
        processor_info_entries,
        vendor,
        url,
        vst3_id,
        clap_id,
    } = input;

    let n = processors.len();
    let n_lit = syn::LitInt::new(&n.to_string(), proc_macro2::Span::call_site());
    let np1_lit = syn::LitInt::new(&(n + 1).to_string(), proc_macro2::Span::call_site());

    // Generate per-processor field names: __proc_0, __proc_1, ...
    let proc_field_names: Vec<syn::Ident> = (0..n)
        .map(|i| syn::Ident::new(&format!("__proc_{}", i), proc_macro2::Span::call_site()))
        .collect();

    // Generate u8 index literals: 0u8, 1u8, ...
    let proc_idx_u8: Vec<proc_macro2::TokenStream> = (0..n)
        .map(|i| {
            let lit = syn::LitInt::new(&format!("{}u8", i), proc_macro2::Span::call_site());
            quote! { #lit }
        })
        .collect();

    // Generate usize index literals: 0usize, 1usize, ...
    let _proc_idx_usize: Vec<proc_macro2::TokenStream> = (0..n)
        .map(|i| {
            let lit = syn::LitInt::new(&format!("{}usize", i), proc_macro2::Span::call_site());
            quote! { #lit }
        })
        .collect();

    // Per-processor struct field declarations
    let proc_struct_fields: Vec<proc_macro2::TokenStream> = processors
        .iter()
        .zip(proc_field_names.iter())
        .map(|(ty, fname)| {
            quote! { #fname: #krate::Bypassed<#ty>, }
        })
        .collect();

    // Per-processor default initialisers in Default impl
    let proc_defaults: Vec<proc_macro2::TokenStream> = processors
        .iter()
        .zip(proc_field_names.iter())
        .map(|(ty, fname)| {
            quote! {
                #fname: <#krate::Bypassed<#ty> as ::std::default::Default>::default(),
            }
        })
        .collect();

    // compile-time trait validation calls
    let proc_validations: Vec<proc_macro2::TokenStream> = processors
        .iter()
        .map(|ty| {
            quote! { assert_processor_traits::<#krate::Bypassed<#ty>>(); }
        })
        .collect();

    // set_sample_rate calls in initialize()
    let set_sample_rate_calls: Vec<proc_macro2::TokenStream> = proc_field_names
        .iter()
        .map(|fname| {
            quote! {
                #krate::Processor::set_sample_rate(&mut self.#fname, _buffer_config.sample_rate);
            }
        })
        .collect();

    // reset() calls
    let reset_calls: Vec<proc_macro2::TokenStream> = proc_field_names
        .iter()
        .map(|fname| {
            quote! { #krate::Processor::reset(&mut self.#fname); }
        })
        .collect();

    // Static dispatch arms for process() loop
    let dispatch_arms: Vec<proc_macro2::TokenStream> = processors
        .iter()
        .zip(proc_field_names.iter())
        .enumerate()
        .map(|(i, (ty, fname))| {
            let i_lit = syn::LitInt::new(&i.to_string(), proc_macro2::Span::call_site());
            let i_lit_usize = syn::LitInt::new(&format!("{}", i), proc_macro2::Span::call_site());
            quote! {
                #i_lit => {
                    let __start = self.__param_offsets[#i_lit_usize];
                    let __end   = self.__param_offsets[#i_lit_usize + 1];
                    let mut __pp =
                        <<#krate::Bypassed<#ty> as #krate::Processor>::Params
                            as #krate::ProcessorParams>::from_param_defaults();
                    <<#krate::Bypassed<#ty> as #krate::Processor>::Params
                        as #krate::ProcessorParams>::apply_plain_values(
                        &mut __pp,
                        &__all_values[__start..__end],
                    );
                    use #krate::Processor as _;
                    self.#fname.process(&mut __channel_slices[..__active_ch], &__transport, &__pp);
                }
            }
        })
        .collect();

    // Build index literals for proc_idx_usize used in quote! (needed after removing the
    // zip-based approach; regenerate from 0..n with usize suffix)
    let proc_idx_usize: Vec<proc_macro2::TokenStream> = (0..n)
        .map(|i| {
            let lit = syn::LitInt::new(&format!("{}usize", i), proc_macro2::Span::call_site());
            quote! { #lit }
        })
        .collect();

    // param_count expressions for __param_offsets initialisation
    let param_count_exprs: Vec<proc_macro2::TokenStream> = processors
        .iter()
        .map(|ty| {
            quote! {
                <<#krate::Bypassed<#ty> as #krate::Processor>::Params
                    as #krate::ProcessorParams>::plain_value_count()
            }
        })
        .collect();

    quote! {
        // Keep SignalChain type alias for dev-FFI vtable (uses static dispatch in registration order)
        type __ProcessorType = #krate::SignalChain![#(#processors),*];

        // Compile-time validation: every processor type must satisfy required trait bounds
        const _: () = {
            fn assert_processor_traits<T>()
            where
                T: #krate::Processor + ::std::default::Default + ::std::marker::Send + 'static,
                T::Params: #krate::ProcessorParams
                    + ::std::default::Default
                    + ::std::marker::Send
                    + ::std::marker::Sync
                    + 'static,
            {
            }
            fn validate() {
                #(#proc_validations)*
            }
        };

        /// Number of processors in the signal chain (compile-time constant).
        const __PROC_COUNT: usize = #n_lit;

        /// Number of samples to crossfade over when reordering processors.
        const __CROSSFADE_SAMPLES: usize = 256;

        /// Generated plugin struct.
        pub struct __WavecraftPlugin {
            params: ::std::sync::Arc<__WavecraftParams>,
            // --- Per-processor fields (Phase 2: runtime reordering) ---
            #(#proc_struct_fields)*
            // Audio-thread-local processing order (index array into per-proc fields)
            __current_order: [u8; #n_lit],
            // Cached cumulative parameter offsets for each processor
            __param_offsets: [usize; #np1_lit],
            // --- Crossfade state (Phase 4) ---
            __cf_pos: usize,
            __cf_dir: i8, // -1 = fade-down, 0 = normal, 1 = fade-up
            // Pre-allocated parameter scratch buffer (reused each block, zero allocation after init)
            __param_scratch: ::std::vec::Vec<f32>,
            // --- Metering ---
            oscilloscope_tap: #krate::OscilloscopeTap,
            meter_producer: #krate::MeterProducer,
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            meter_consumer: ::std::sync::Mutex<::std::option::Option<#krate::MeterConsumer>>,
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            oscilloscope_consumer:
                ::std::sync::Mutex<::std::option::Option<#krate::OscilloscopeFrameConsumer>>,
        }

        /// Generated params struct.
        ///
        /// Parameters are discovered at runtime from processor param_specs().
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
            // Runtime parameter values (flat, all processors in chain order)
            params: ::std::vec::Vec<__WavecraftRuntimeParam>,
            // Runtime IDs aligned with FFI-generated contract IDs (e.g. test_tone_frequency)
            ids: ::std::vec::Vec<::std::string::String>,
            // Optional parameter group names (empty string when none)
            groups: ::std::vec::Vec<::std::string::String>,
            // --- Phase 2: Lock-free SPSC order handoff (shared between UI and audio threads) ---
            __has_pending_order: ::std::sync::atomic::AtomicBool,
            __pending_slots: ::std::vec::Vec<::std::sync::atomic::AtomicU8>,
            // --- Phase 3: Order state for IPC reads and persistence ---
            __order_state: ::std::sync::Mutex<::std::vec::Vec<u8>>,
        }

        impl __WavecraftParams {
            fn from_processor_specs() -> Self {
                use #krate::ParamRange;

                let mut params = ::std::vec::Vec::new();
                let mut ids = ::std::vec::Vec::new();
                let mut groups = ::std::vec::Vec::new();

                #(#runtime_param_blocks)*

                // Initialise pending-order slots (identity permutation)
                let mut __pending: ::std::vec::Vec<::std::sync::atomic::AtomicU8> =
                    ::std::vec::Vec::with_capacity(#n_lit);
                #(
                    __pending.push(::std::sync::atomic::AtomicU8::new(#proc_idx_u8));
                )*

                Self {
                    params,
                    ids,
                    groups,
                    __has_pending_order: ::std::sync::atomic::AtomicBool::new(false),
                    __pending_slots: __pending,
                    __order_state: ::std::sync::Mutex::new(vec![#(#proc_idx_u8),*]),
                }
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
                    .map(|((param, id), group)| (id.clone(), param.as_ptr(), group.clone()))
                    .collect()
            }

            fn serialize_fields(
                &self,
            ) -> ::std::collections::BTreeMap<::std::string::String, ::std::string::String>
            {
                let __order: ::std::vec::Vec<::std::string::String> = self
                    .__order_state
                    .lock()
                    .unwrap_or_else(|__e| __e.into_inner())
                    .iter()
                    .map(|&__i| __i.to_string())
                    .collect();

                let __json = #krate::__internal::serde_json::to_string(&__order)
                    .unwrap_or_default();

                let mut __map =
                    ::std::collections::BTreeMap::<::std::string::String, ::std::string::String>::new();
                __map.insert("processorOrder".to_string(), __json);
                __map
            }

            fn deserialize_fields(
                &self,
                fields: &::std::collections::BTreeMap<::std::string::String, ::std::string::String>,
            ) {
                if let Some(__json) = fields.get("processorOrder") {
                    if let Ok(__order_strs) =
                        #krate::__internal::serde_json::from_str::<::std::vec::Vec<::std::string::String>>(
                            __json,
                        )
                    {
                        let __order: ::std::vec::Vec<u8> = __order_strs
                            .iter()
                            .filter_map(|__s| __s.parse::<u8>().ok())
                            .collect();

                        if __order.len() == __PROC_COUNT {
                            // Validate: must be a permutation of 0..N
                            let mut __seen = [false; #n_lit];
                            let mut __valid = true;
                            for &__slot in __order.iter() {
                                let __idx = __slot as usize;
                                if __idx >= __PROC_COUNT || __seen[__idx] {
                                    __valid = false;
                                    break;
                                }
                                __seen[__idx] = true;
                            }
                            if __valid {
                                for (__i, &__slot) in __order.iter().enumerate() {
                                    self.__pending_slots[__i].store(
                                        __slot,
                                        ::std::sync::atomic::Ordering::Release,
                                    );
                                }
                                self.__has_pending_order.store(
                                    true,
                                    ::std::sync::atomic::Ordering::Release,
                                );
                                *self
                                    .__order_state
                                    .lock()
                                    .unwrap_or_else(|__e| __e.into_inner()) = __order;
                            } else {
                                eprintln!(
                                    "[wavecraft] processorOrderRestoreFailed: persisted order \
                                     has wrong length or is not a valid permutation of slot \
                                     indices. Falling back to default registration order."
                                );
                            }
                        }
                    }
                }
            }
        }

        impl #krate::__nih::ProcessorOrderAccess for __WavecraftParams {
            fn get_order(&self) -> ::std::vec::Vec<::std::string::String> {
                self.__order_state
                    .lock()
                    .unwrap_or_else(|__e| __e.into_inner())
                    .iter()
                    .map(|__b| __b.to_string())
                    .collect()
            }

            fn set_order(
                &self,
                order: &[::std::string::String],
            ) -> ::std::result::Result<(), #krate::__nih::BridgeError> {
                let __n: usize = #n_lit;
                if order.len() != __n {
                    return ::std::result::Result::Err(
                        #krate::__nih::BridgeError::InvalidProcessorOrder {
                            reason: ::std::format!(
                                "expected {} slots, got {}",
                                __n,
                                order.len()
                            ),
                        },
                    );
                }
                let mut __parsed = ::std::vec::Vec::with_capacity(__n);
                for __s in order.iter() {
                    match __s.parse::<u8>() {
                        ::std::result::Result::Ok(__b) => __parsed.push(__b),
                        ::std::result::Result::Err(_) => {
                            return ::std::result::Result::Err(
                                #krate::__nih::BridgeError::InvalidProcessorOrder {
                                    reason: ::std::format!(
                                        "slot index '{}' is not a valid u8",
                                        __s
                                    ),
                                },
                            );
                        }
                    }
                }
                let mut __seen = ::std::vec![false; __n];
                for &__b in __parsed.iter() {
                    let __idx = __b as usize;
                    if __idx >= __n || __seen[__idx] {
                        return ::std::result::Result::Err(
                            #krate::__nih::BridgeError::InvalidProcessorOrder {
                                reason: ::std::format!(
                                    "invalid permutation: slot {} is out of range or duplicate",
                                    __b
                                ),
                            },
                        );
                    }
                    __seen[__idx] = true;
                }
                use ::std::sync::atomic::Ordering;
                for (__i, &__b) in __parsed.iter().enumerate() {
                    self.__pending_slots[__i].store(__b, Ordering::Release);
                }
                self.__has_pending_order.store(true, Ordering::Release);
                *self.__order_state.lock().unwrap_or_else(|__e| __e.into_inner()) = __parsed;
                ::std::result::Result::Ok(())
            }
        }

        impl ::std::default::Default for __WavecraftPlugin {
            fn default() -> Self {
                let (meter_producer, _meter_consumer) = #krate::create_meter_channel(64);
                let (oscilloscope_producer, _oscilloscope_consumer) =
                    #krate::create_oscilloscope_channel(8);

                // Compute cumulative param offsets at construction time (not audio-thread)
                let __param_offsets: [usize; #np1_lit] = {
                    let __counts: [usize; #n_lit] = [#(#param_count_exprs),*];
                    let mut __offs = [0usize; #np1_lit];
                    let mut __acc = 0usize;
                    let mut __i = 0usize;
                    while __i < #n_lit {
                        __acc += __counts[__i];
                        __offs[__i + 1] = __acc;
                        __i += 1;
                    }
                    __offs
                };

                Self {
                    params: ::std::sync::Arc::new(__WavecraftParams::default()),
                    #(#proc_defaults)*
                    __current_order: [#(#proc_idx_u8),*],
                    __param_offsets,
                    __cf_pos: 0,
                    __cf_dir: 0i8,
                    // Pre-grow to total param count so process() never reallocates.
                    __param_scratch: ::std::vec::Vec::with_capacity(__param_offsets[#n_lit]),
                    oscilloscope_tap: #krate::OscilloscopeTap::with_output(oscilloscope_producer),
                    meter_producer,
                    #[cfg(any(target_os = "macos", target_os = "windows"))]
                    meter_consumer: ::std::sync::Mutex::new(
                        ::std::option::Option::Some(_meter_consumer),
                    ),
                    #[cfg(any(target_os = "macos", target_os = "windows"))]
                    oscilloscope_consumer: ::std::sync::Mutex::new(
                        ::std::option::Option::Some(_oscilloscope_consumer),
                    ),
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
                },
            ];

            const MIDI_INPUT: #krate::__nih::MidiConfig = #krate::__nih::MidiConfig::None;
            const MIDI_OUTPUT: #krate::__nih::MidiConfig = #krate::__nih::MidiConfig::None;

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
                        .expect("meter_consumer mutex poisoned")
                        .take();
                    let oscilloscope_consumer = self
                        .oscilloscope_consumer
                        .lock()
                        .expect("oscilloscope_consumer mutex poisoned")
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
                #(#set_sample_rate_calls)*
                self.oscilloscope_tap
                    .set_sample_rate_hz(_buffer_config.sample_rate);
                true
            }

            fn reset(&mut self) {
                #(#reset_calls)*
                #krate::Processor::reset(&mut self.oscilloscope_tap);
            }

            fn process(
                &mut self,
                buffer: &mut #krate::__nih::Buffer,
                _aux: &mut #krate::__nih::AuxiliaryBuffers,
                _context: &mut impl #krate::__nih::ProcessContext<Self>,
            ) -> #krate::__nih::ProcessStatus {
                // ── Phase 2/4: Block-start order handoff ─────────────────────────────
                //
                // 1. If fade-down complete (cf_dir == -1, cf_pos == 0), apply the
                //    pending order and switch to fade-up.
                if self.__cf_dir == -1i8 && self.__cf_pos == 0 {
                    self.params.__has_pending_order
                        .store(false, ::std::sync::atomic::Ordering::Release);
                    #(
                        self.__current_order[#proc_idx_usize] = self
                            .params
                            .__pending_slots[#proc_idx_usize]
                            .load(::std::sync::atomic::Ordering::Acquire);
                    )*
                    self.__cf_dir = 1i8; // start fade-up from silence
                }
                // 2. If idle and a new pending order has arrived, start a fade-down.
                if self.__cf_dir == 0i8
                    && self.params.__has_pending_order
                        .load(::std::sync::atomic::Ordering::Acquire)
                {
                    self.__cf_pos = __CROSSFADE_SAMPLES;
                    self.__cf_dir = -1i8;
                }

                let __num_samples = buffer.samples();
                let __channels = buffer.channels();

                // Pre-collect all current parameter plain-values into pre-allocated scratch buffer.
                // clear() sets len to 0 without freeing; push() never reallocates because capacity
                // was pre-grown to the total param count at construction. Zero heap allocation per block.
                self.__param_scratch.clear();
                for __p in self.params.params.iter() {
                    self.__param_scratch.push(__p.modulated_plain_value());
                }
                let __all_values: &[f32] = &self.__param_scratch;

                for __sample_idx in 0..__num_samples {
                    let __transport = #krate::Transport::default();
                    // Load samples into a stack-allocated 2-channel buffer.
                    // Plugin audio layout is fixed stereo (2-in/2-out); no heap involved.
                    let mut __stereo: [f32; 2] = [
                        buffer.as_slice()[0][__sample_idx],
                        if __channels >= 2 {
                            buffer.as_slice()[1][__sample_idx]
                        } else {
                            0.0_f32
                        },
                    ];
                    let __active_ch = __channels.min(2);
                    {
                        // Create mutable slice refs over the stack array — both are stack-allocated.
                        let (__sl0, __sl1) = __stereo.split_at_mut(1);
                        let mut __channel_slices: [&mut [f32]; 2] = [__sl0, __sl1];
                        // ── Phase 2: Runtime-ordered static dispatch ──────────────────────
                        let mut __pos: usize = 0;
                        while __pos < __PROC_COUNT {
                            let __slot = self.__current_order[__pos] as usize;
                            match __slot {
                                #(#dispatch_arms)*
                                _ => {}
                            }
                            __pos += 1;
                        }
                    } // __channel_slices (and mutable borrow of __stereo) released here

                    // ── Phase 4: Per-sample crossfade gain ────────────────────────────
                    let __gain: f32 = if self.__cf_dir != 0i8 {
                        let __g = self.__cf_pos as f32 / __CROSSFADE_SAMPLES as f32;
                        if self.__cf_dir > 0i8 {
                            self.__cf_pos += 1;
                            if self.__cf_pos >= __CROSSFADE_SAMPLES {
                                self.__cf_pos = __CROSSFADE_SAMPLES;
                                self.__cf_dir = 0i8;
                            }
                        } else {
                            // fading down
                            if self.__cf_pos > 0 {
                                self.__cf_pos -= 1;
                            }
                        }
                        __g
                    } else {
                        1.0_f32
                    };

                    // Write processed (and gain-adjusted) samples back to the buffer.
                    // SAFETY: nih-plug process() guarantees exclusive buffer access;
                    // `__sample_idx` < `__num_samples` which came from `buffer.samples()`.
                    unsafe {
                        let __ptr = buffer.as_slice()[0].as_ptr() as *mut f32;
                        *__ptr.add(__sample_idx) = __stereo[0] * __gain;
                    }
                    if __channels >= 2 {
                        // SAFETY: channel 1 exists (__channels >= 2); same exclusive-access
                        // and bounds guarantees as channel 0.
                        unsafe {
                            let __ptr = buffer.as_slice()[1].as_ptr() as *mut f32;
                            *__ptr.add(__sample_idx) = __stereo[1] * __gain;
                        }
                    }
                }

                // ── Metering ─────────────────────────────────────────────────────────
                let mut __peak_l = 0.0_f32;
                let mut __peak_r = 0.0_f32;
                if __channels >= 1 {
                    __peak_l = buffer.as_slice()[0]
                        .iter()
                        .map(|&s| s.abs())
                        .fold(0.0_f32, f32::max);
                }
                if __channels >= 2 {
                    __peak_r = buffer.as_slice()[1]
                        .iter()
                        .map(|&s| s.abs())
                        .fold(0.0_f32, f32::max);
                }

                // Block-end snapshot: acceptable once-per-block allocation.
                if __channels >= 1 {
                    let __left_snap: ::std::vec::Vec<f32> = buffer.as_slice()[0].to_vec();
                    let __right_snap: ::std::vec::Vec<f32> = if __channels >= 2 {
                        buffer.as_slice()[1].to_vec()
                    } else {
                        __left_snap.clone()
                    };
                    self.oscilloscope_tap
                        .capture_stereo(&__left_snap, &__right_snap);
                }

                let _ = self.meter_producer.push(#krate::MeterFrame {
                    peak_l: __peak_l,
                    peak_r: __peak_r,
                    rms_l: __peak_l * 0.707,
                    rms_r: __peak_r * 0.707,
                    timestamp: 0,
                });

                #krate::__nih::ProcessStatus::Normal
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
            const VST3_SUBCATEGORIES: &'static [#krate::__nih::Vst3SubCategory] =
                &[#krate::__nih::Vst3SubCategory::Fx];
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
        /// # Safety
        /// The returned pointer must be freed with `wavecraft_free_string`.
        #[unsafe(no_mangle)]
        pub extern "C" fn wavecraft_get_params_json() -> *mut ::std::ffi::c_char {
            let mut params: ::std::vec::Vec<#krate::__internal::ParameterInfo> =
                ::std::vec::Vec::new();
            #(#processor_param_mappings)*
            let json = #krate::__internal::serde_json::to_string(&params)
                .unwrap_or_else(|_| "[]".to_string());
            ::std::ffi::CString::new(json)
                .map(|s| s.into_raw())
                .unwrap_or(::std::ptr::null_mut())
        }

        /// Returns JSON-serialized processor metadata.
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
                    if instance.is_null()
                        || channels.is_null()
                        || num_channels == 0
                        || num_samples == 0
                    {
                        return;
                    }
                    // SAFETY: `instance` is non-null (checked above), was created by
                    // `Box::into_raw` in the `create` function, and has not been freed.
                    // The cast is sound because the pointer type matches `__DevProcessorInstance`.
                    let instance =
                        unsafe { &mut *(instance as *mut __DevProcessorInstance) };
                    let num_ch = num_channels as usize;
                    let num_samp = num_samples as usize;
                    let transport = #krate::Transport::default();

                    match num_ch {
                        1 => {
                            // SAFETY: `channels` is non-null (checked above) and points to
                            // at least `num_channels` (== 1) valid f32 pointers; index 0 is valid.
                            let ch0_ptr = unsafe { *channels.add(0) };
                            if ch0_ptr.is_null() {
                                return;
                            }
                            // SAFETY: `ch0_ptr` is non-null (checked above), points to
                            // `num_samp` f32 values owned by the caller, valid for this call.
                            let ch0 = unsafe {
                                ::std::slice::from_raw_parts_mut(ch0_ptr, num_samp)
                            };
                            let mut channel_slices: [&mut [f32]; 1] = [ch0];
                            #krate::Processor::process(
                                &mut instance.processor,
                                &mut channel_slices,
                                &transport,
                                &instance.params,
                            );
                        }
                        2 => {
                            // SAFETY: `channels` is non-null (checked above) and points to
                            // at least `num_channels` (== 2) valid f32 pointers; indices 0
                            // and 1 are both valid.
                            let ch0_ptr = unsafe { *channels.add(0) };
                            let ch1_ptr = unsafe { *channels.add(1) };
                            if ch0_ptr.is_null() || ch1_ptr.is_null() {
                                return;
                            }
                            // SAFETY: `ch0_ptr` / `ch1_ptr` are non-null (checked above),
                            // point to `num_samp` f32 values owned by the caller, valid for
                            // this call. The two pointers are disjoint by caller contract.
                            let ch0 = unsafe {
                                ::std::slice::from_raw_parts_mut(ch0_ptr, num_samp)
                            };
                            let ch1 = unsafe {
                                ::std::slice::from_raw_parts_mut(ch1_ptr, num_samp)
                            };
                            let mut channel_slices: [&mut [f32]; 2] = [ch0, ch1];
                            #krate::Processor::process(
                                &mut instance.processor,
                                &mut channel_slices,
                                &transport,
                                &instance.params,
                            );
                        }
                        _ => {} // Unsupported channel topology in dev-FFI path
                    }
                }));
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
                    // SAFETY: `instance` is non-null (checked above), was created by
                    // `Box::into_raw` in `create`, and has not been freed.
                    let instance =
                        unsafe { &mut *(instance as *mut __DevProcessorInstance) };
                    let values: &[f32] = if len == 0 {
                        &[]
                    } else {
                        // SAFETY: `values_ptr` is non-null (checked above), points to `len`
                        // valid f32 values provided by the caller, valid for this call.
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
                    // SAFETY: `instance` is non-null (checked above), was created by
                    // `Box::into_raw` in `create`, and has not been freed.
                    let instance =
                        unsafe { &mut *(instance as *mut __DevProcessorInstance) };
                    #krate::Processor::set_sample_rate(&mut instance.processor, sample_rate);
                }));
            }

            extern "C" fn reset(instance: *mut c_void) {
                let _ = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    if instance.is_null() {
                        return;
                    }
                    // SAFETY: `instance` is non-null (checked above), was created by
                    // `Box::into_raw` in `create`, and has not been freed.
                    let instance =
                        unsafe { &mut *(instance as *mut __DevProcessorInstance) };
                    #krate::Processor::reset(&mut instance.processor);
                }));
            }

            extern "C" fn drop_fn(instance: *mut c_void) {
                let _ = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    if !instance.is_null() {
                        // SAFETY: `instance` was created by `Box::into_raw` in `create`,
                        // is non-null (checked above), and `drop_fn` is called exactly once
                        // per the vtable contract. Ownership is transferred back to Box.
                        let _ = unsafe {
                            ::std::boxed::Box::from_raw(
                                instance as *mut __DevProcessorInstance,
                            )
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
