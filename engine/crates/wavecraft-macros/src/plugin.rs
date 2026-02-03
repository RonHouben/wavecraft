//! Procedural macro for generating complete plugin implementations from DSL.
//!
//! This is Phase 6 of the declarative plugin DSL feature.
//! Current implementation is a simplified version that generates working code
//! but requires further refinement for full feature parity.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Expr, Ident, LitStr, Result, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

/// Input structure for `wavecraft_plugin!` macro.
struct PluginDef {
    name: LitStr,
    vendor: LitStr,
    url: Option<LitStr>,
    email: Option<LitStr>,
    signal: Expr,
}

impl Parse for PluginDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name = None;
        let mut vendor = None;
        let mut url = None;
        let mut email = None;
        let mut signal = None;

        // Parse key-value pairs
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "name" => name = Some(input.parse()?),
                "vendor" => vendor = Some(input.parse()?),
                "url" => url = Some(input.parse()?),
                "email" => email = Some(input.parse()?),
                "signal" => signal = Some(input.parse()?),
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown field: `{}`", key),
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(PluginDef {
            name: name.ok_or_else(|| {
                input.error(
                    "missing required field: `name`\n\
                     \n\
                     Example:\n\
                     wavecraft_plugin! {\n\
                         name: \"My Plugin\",\n\
                         vendor: \"My Company\",\n\
                         signal: Chain![MyGain],\n\
                     }",
                )
            })?,
            vendor: vendor.ok_or_else(|| {
                input.error(
                    "missing required field: `vendor`\n\
                     \n\
                     Example:\n\
                     wavecraft_plugin! {\n\
                         name: \"My Plugin\",\n\
                         vendor: \"My Company\",\n\
                         signal: Chain![MyGain],\n\
                     }",
                )
            })?,
            url,
            email,
            signal: signal.ok_or_else(|| {
                input.error(
                    "missing required field: `signal`\n\
                     \n\
                     The signal field defines your DSP processing chain.\n\
                     \n\
                     Example:\n\
                     wavecraft_plugin! {\n\
                         name: \"My Plugin\",\n\
                         vendor: \"My Company\",\n\
                         signal: Chain![MyGain],\n\
                     }\n\
                     \n\
                     For multiple processors:\n\
                     signal: Chain![InputGain, Filter, OutputGain]",
                )
            })?,
        })
    }
}

/// Generate a deterministic VST3 ID from plugin name and vendor.
fn generate_vst3_id(name: &str, vendor: &str) -> proc_macro2::TokenStream {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    format!("{}{}", vendor, name).hash(&mut hasher);
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

pub fn wavecraft_plugin_impl(input: TokenStream) -> TokenStream {
    let plugin_def = parse_macro_input!(input as PluginDef);

    let name = &plugin_def.name;
    let vendor = &plugin_def.vendor;
    let url = plugin_def
        .url
        .unwrap_or_else(|| LitStr::new("", proc_macro2::Span::call_site()));
    let email = plugin_def
        .email
        .unwrap_or_else(|| LitStr::new("", proc_macro2::Span::call_site()));
    let signal_type = &plugin_def.signal;

    let vst3_id = generate_vst3_id(&name.value(), &vendor.value());

    // Phase 6 Steps 6.1-6.6 Complete:
    // - Input parsing ✓
    // - Plugin struct generation ✓
    // - Params struct with runtime parameter discovery ✓
    // - Plugin trait impl with audio processing ✓
    // - Format impls & exports ✓
    // - Error messages (compile-time validation) ✓

    let expanded = quote! {
        // Use the signal expression as the processor type
        type __ProcessorType = #signal_type;

        // Compile-time validation: ensure the processor type implements required traits
        const _: () = {
            fn assert_processor_traits<T>()
            where
                T: ::wavecraft_dsp::Processor + ::std::default::Default + ::std::marker::Send + 'static,
                T::Params: ::wavecraft_dsp::ProcessorParams + ::std::default::Default + ::std::marker::Send + ::std::marker::Sync + 'static,
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
            meter_producer: ::wavecraft_metering::MeterProducer,
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            meter_consumer: ::std::sync::Arc<::std::sync::Mutex<::wavecraft_metering::MeterConsumer>>,
        }

        /// Generated params struct.
        ///
        /// This struct bridges wavecraft-dsp ProcessorParams to nih-plug's Params trait.
        /// Parameters are discovered at runtime from the processor's param_specs().
        pub struct __WavecraftParams {
            // Store parameters as a vector for dynamic discovery
            params: ::std::vec::Vec<::nih_plug::prelude::FloatParam>,
        }

        impl __WavecraftParams {
            fn from_processor_specs() -> Self
            where
                <__ProcessorType as ::wavecraft_dsp::Processor>::Params: ::wavecraft_dsp::ProcessorParams,
            {
                let specs = <<__ProcessorType as ::wavecraft_dsp::Processor>::Params as ::wavecraft_dsp::ProcessorParams>::param_specs();

                let params = specs
                    .iter()
                    .map(|spec| {
                        use ::wavecraft_dsp::ParamRange;

                        let range = match &spec.range {
                            ParamRange::Linear { min, max } => {
                                ::nih_plug::prelude::FloatRange::Linear {
                                    min: *min as f32,
                                    max: *max as f32,
                                }
                            }
                            ParamRange::Skewed { min, max, factor } => {
                                ::nih_plug::prelude::FloatRange::Skewed {
                                    min: *min as f32,
                                    max: *max as f32,
                                    factor: *factor as f32,
                                }
                            }
                            ParamRange::Stepped { min, max } => {
                                // Convert stepped range to linear for now
                                ::nih_plug::prelude::FloatRange::Linear {
                                    min: *min as f32,
                                    max: *max as f32,
                                }
                            }
                        };

                        ::nih_plug::prelude::FloatParam::new(
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
        unsafe impl ::nih_plug::prelude::Params for __WavecraftParams {
            fn param_map(&self) -> ::std::vec::Vec<(
                ::std::string::String,
                ::nih_plug::prelude::ParamPtr,
                ::std::string::String,
            )> {
                use ::nih_plug::prelude::Param; // Import trait for as_ptr()

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
                    ::wavecraft_metering::create_meter_channel(64);
                Self {
                    params: ::std::sync::Arc::new(__WavecraftParams::default()),
                    processor: <__ProcessorType as ::std::default::Default>::default(),
                    meter_producer,
                    #[cfg(any(target_os = "macos", target_os = "windows"))]
                    meter_consumer: ::std::sync::Arc::new(::std::sync::Mutex::new(_meter_consumer)),
                }
            }
        }

        impl ::nih_plug::prelude::Plugin for __WavecraftPlugin {
            const NAME: &'static str = #name;
            const VENDOR: &'static str = #vendor;
            const URL: &'static str = #url;
            const EMAIL: &'static str = #email;
            const VERSION: &'static str = env!("CARGO_PKG_VERSION");

            const AUDIO_IO_LAYOUTS: &'static [::nih_plug::prelude::AudioIOLayout] = &[
                ::nih_plug::prelude::AudioIOLayout {
                    main_input_channels: ::std::num::NonZeroU32::new(2),
                    main_output_channels: ::std::num::NonZeroU32::new(2),
                    ..::nih_plug::prelude::AudioIOLayout::const_default()
                }
            ];

            const MIDI_INPUT: ::nih_plug::prelude::MidiConfig =
                ::nih_plug::prelude::MidiConfig::None;
            const MIDI_OUTPUT: ::nih_plug::prelude::MidiConfig =
                ::nih_plug::prelude::MidiConfig::None;

            type SysExMessage = ();
            type BackgroundTask = ();

            fn params(&self) -> ::std::sync::Arc<dyn ::nih_plug::prelude::Params> {
                self.params.clone()
            }

            fn editor(
                &mut self,
                _async_executor: ::nih_plug::prelude::AsyncExecutor<Self>,
            ) -> ::std::option::Option<::std::boxed::Box<dyn ::nih_plug::prelude::Editor>> {
                #[cfg(any(target_os = "macos", target_os = "windows"))]
                {
                    ::wavecraft_core::editor::create_webview_editor(
                        self.params.clone(),
                        self.meter_consumer.clone(),
                    )
                }

                #[cfg(not(any(target_os = "macos", target_os = "windows")))]
                {
                    None
                }
            }

            fn initialize(
                &mut self,
                _audio_io_layout: &::nih_plug::prelude::AudioIOLayout,
                _buffer_config: &::nih_plug::prelude::BufferConfig,
                _context: &mut impl ::nih_plug::prelude::InitContext<Self>,
            ) -> bool {
                true
            }

            fn reset(&mut self) {}

            fn process(
                &mut self,
                buffer: &mut ::nih_plug::prelude::Buffer,
                _aux: &mut ::nih_plug::prelude::AuxiliaryBuffers,
                _context: &mut impl ::nih_plug::prelude::ProcessContext<Self>,
            ) -> ::nih_plug::prelude::ProcessStatus {
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

                    let transport = ::wavecraft_dsp::Transport::default();

                    // Import Processor trait for process() method
                    use ::wavecraft_dsp::Processor as _;
                    self.processor.process(&mut sample_ptrs, &transport, &processor_params);

                    // Write processed samples back
                    for (ch, sample_buf) in sample_buffers.iter().enumerate() {
                        if let Some(channel) = buffer.as_slice().get(ch) {
                            if sample_idx < channel.len() {
                                // Safety: we're within bounds
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

                let frame = ::wavecraft_metering::MeterFrame {
                    peak_l: peak_left,
                    peak_r: peak_right,
                    rms_l: peak_left * 0.707, // Simplified RMS estimation
                    rms_r: peak_right * 0.707,
                    timestamp: 0, // TODO: Add proper timestamp
                };

                let _ = self.meter_producer.push(frame);

                ::nih_plug::prelude::ProcessStatus::Normal
            }
        }

        impl __WavecraftPlugin {
            /// Build processor parameters from current nih-plug parameter values.
            fn build_processor_params(&self) -> <__ProcessorType as ::wavecraft_dsp::Processor>::Params {
                // For now, use default params
                // TODO: Map nih-plug parameter values to processor params
                <<__ProcessorType as ::wavecraft_dsp::Processor>::Params as ::std::default::Default>::default()
            }
        }

        impl ::nih_plug::prelude::ClapPlugin for __WavecraftPlugin {
            const CLAP_ID: &'static str = concat!("com.", #vendor, ".", #name);
            const CLAP_DESCRIPTION: Option<&'static str> = None;
            const CLAP_MANUAL_URL: Option<&'static str> = None;
            const CLAP_SUPPORT_URL: Option<&'static str> = None;
            const CLAP_FEATURES: &'static [::nih_plug::prelude::ClapFeature] = &[
                ::nih_plug::prelude::ClapFeature::AudioEffect,
                ::nih_plug::prelude::ClapFeature::Stereo,
            ];
        }

        impl ::nih_plug::prelude::Vst3Plugin for __WavecraftPlugin {
            const VST3_CLASS_ID: [u8; 16] = #vst3_id;
            const VST3_SUBCATEGORIES: &'static [::nih_plug::prelude::Vst3SubCategory] = &[
                ::nih_plug::prelude::Vst3SubCategory::Fx,
            ];
        }

        ::nih_plug::nih_export_clap!(__WavecraftPlugin);
        ::nih_plug::nih_export_vst3!(__WavecraftPlugin);
    };

    TokenStream::from(expanded)
}
