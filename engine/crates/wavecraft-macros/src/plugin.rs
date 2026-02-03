//! Procedural macro for generating complete plugin implementations from DSL.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, Ident, LitStr, Result, Token,
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

        // Parse key-value pairs: `name: "...", vendor: "...", signal: Chain![...]`
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "name" => {
                    name = Some(input.parse()?);
                }
                "vendor" => {
                    vendor = Some(input.parse()?);
                }
                "url" => {
                    url = Some(input.parse()?);
                }
                "email" => {
                    email = Some(input.parse()?);
                }
                "signal" => {
                    signal = Some(input.parse()?);
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown field: `{}`", key),
                    ));
                }
            }

            // Parse optional trailing comma
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(PluginDef {
            name: name.ok_or_else(|| input.error("missing required field: `name`"))?,
            vendor: vendor.ok_or_else(|| input.error("missing required field: `vendor`"))?,
            url,
            email,
            signal: signal.ok_or_else(|| input.error("missing required field: `signal`"))?,
        })
    }
}

pub fn wavecraft_plugin_impl(input: TokenStream) -> TokenStream {
    let plugin_def = parse_macro_input!(input as PluginDef);

    let name = plugin_def.name;
    let vendor = plugin_def.vendor;
    let url = plugin_def.url.unwrap_or_else(|| LitStr::new("", proc_macro2::Span::call_site()));
    let email = plugin_def.email.unwrap_or_else(|| LitStr::new("", proc_macro2::Span::call_site()));
    let signal_expr = plugin_def.signal;

    // For Phase 6.1-6.2 MVP: Generate a minimal working plugin
    // TODO: Parse signal_expr to extract processor types and generate proper params
    
    let expanded = quote! {
        /// Generated plugin struct.
        pub struct __WavecraftPlugin {
            params: ::std::sync::Arc<__WavecraftParams>,
            processor: <#signal_expr as ::std::default::Default>::Output,
            meter_producer: ::wavecraft_metering::MeterProducer,
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            meter_consumer: ::std::sync::Arc<::std::sync::Mutex<::wavecraft_metering::MeterConsumer>>,
        }

        /// Generated params struct.
        #[derive(::nih_plug::prelude::Params)]
        pub struct __WavecraftParams {
            // TODO: Generate actual parameter fields from signal processors
        }

        impl ::std::default::Default for __WavecraftParams {
            fn default() -> Self {
                Self {}
            }
        }

        impl ::std::default::Default for __WavecraftPlugin {
            fn default() -> Self {
                let (meter_producer, _meter_consumer) = ::wavecraft_metering::create_meter_channel(64);
                Self {
                    params: ::std::sync::Arc::new(__WavecraftParams::default()),
                    processor: <#signal_expr>::default(),
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

            const MIDI_INPUT: ::nih_plug::prelude::MidiConfig = ::nih_plug::prelude::MidiConfig::None;
            const MIDI_OUTPUT: ::nih_plug::prelude::MidiConfig = ::nih_plug::prelude::MidiConfig::None;

            type SysExMessage = ();
            type BackgroundTask = ();

            fn params(&self) -> ::std::sync::Arc<dyn ::nih_plug::prelude::Params> {
                self.params.clone()
            }

            fn editor(&mut self, _async_executor: ::nih_plug::prelude::AsyncExecutor<Self>) -> ::std::option::Option<::std::boxed::Box<dyn ::nih_plug::prelude::Editor>> {
                #[cfg(any(target_os = "macos", target_os = "windows"))]
                {
                    ::wavecraft_core::editor::create_webview_editor(
                        self.params.clone(),
                        self.meter_consumer.clone()
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
                buffer_config: &::nih_plug::prelude::BufferConfig,
                _context: &mut impl ::nih_plug::prelude::InitContext<Self>,
            ) -> bool {
                // TODO: Initialize processor with sample rate
                true
            }

            fn reset(&mut self) {
                // TODO: Reset processor state
            }

            fn process(
                &mut self,
                buffer: &mut ::nih_plug::prelude::Buffer,
                _aux: &mut ::nih_plug::prelude::AuxiliaryBuffers,
                _context: &mut impl ::nih_plug::prelude::ProcessContext<Self>,
            ) -> ::nih_plug::prelude::ProcessStatus {
                // TODO: Call processor.process() with proper buffer conversion
                ::nih_plug::prelude::ProcessStatus::Normal
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
            const VST3_CLASS_ID: [u8; 16] = *b"WavecraftPlgnXXX"; // TODO: Generate unique ID
            const VST3_SUBCATEGORIES: &'static [::nih_plug::prelude::Vst3SubCategory] = &[
                ::nih_plug::prelude::Vst3SubCategory::Fx,
            ];
        }

        ::nih_plug::export_clap!(__WavecraftPlugin);
        ::nih_plug::export_vst3!(__WavecraftPlugin);
    };

    TokenStream::from(expanded)
}
