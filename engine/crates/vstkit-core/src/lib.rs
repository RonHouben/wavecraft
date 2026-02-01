//! VstKit Core - Audio plugin framework
//!
//! This crate provides the main plugin framework for VstKit, including:
//! - nih-plug integration (VST3/CLAP/AU export)
//! - WebView-based UI editor
//! - Parameter management
//! - Real-time metering
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use vstkit_core::prelude::*;
//!
//! // Your plugin implementation here
//! ```

// Public modules for SDK users
pub mod editor;
pub mod prelude;
pub mod util;

// Internal modules
mod params;

use std::sync::Arc;

use vstkit_dsp::GainProcessor;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use vstkit_metering::MeterConsumer;
use vstkit_metering::{MeterFrame, MeterProducer, create_meter_channel};
use nih_plug::prelude::*;

#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::editor::create_webview_editor;
use crate::params::VstKitParams;
use crate::util::calculate_stereo_meters;

/// Main plugin struct for VstKit.
pub struct VstKitPlugin {
    params: Arc<VstKitParams>,
    processor: GainProcessor,
    meter_producer: MeterProducer,
    /// Meter consumer shared with the editor (UI thread only).
    ///
    /// THREAD-SAFETY: This Arc<Mutex> is ONLY accessed from the UI thread
    /// (editor creation/destruction and meter polling). It is NEVER touched
    /// from the audio thread, which uses `meter_producer` instead.
    /// The mutex protects against concurrent editor open/close operations.
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    meter_consumer: Arc<std::sync::Mutex<MeterConsumer>>,
}

impl Default for VstKitPlugin {
    fn default() -> Self {
        let (meter_producer, _meter_consumer) = create_meter_channel(64);
        Self {
            params: Arc::new(VstKitParams::default()),
            processor: GainProcessor::new(44100.0),
            meter_producer,
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            meter_consumer: Arc::new(std::sync::Mutex::new(_meter_consumer)),
        }
    }
}

impl Plugin for VstKitPlugin {
    const NAME: &'static str = "VstKit";
    const VENDOR: &'static str = "VstKit Team";
    const URL: &'static str = "https://github.com/vstkit/vstkit";
    const EMAIL: &'static str = "contact@vstkit.dev";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            create_webview_editor(self.params.clone(), self.meter_consumer.clone())
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            None
        }
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.processor.set_sample_rate(buffer_config.sample_rate);
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Process audio in-place with per-sample smoothing
        for mut channel_samples in buffer.iter_samples() {
            let gain_db = self.params.gain.smoothed.next();
            let gain_linear = vstkit_protocol::db_to_linear(gain_db);

            for sample in channel_samples.iter_mut() {
                *sample *= gain_linear;
            }
        }

        // Calculate and push meter data (after processing)
        if buffer.channels() >= 2 {
            let (peak_l, peak_r, rms_l, rms_r) = calculate_stereo_meters(buffer);
            self.meter_producer.push(MeterFrame {
                peak_l,
                peak_r,
                rms_l,
                rms_r,
                timestamp: 0, // TODO: Use proper timestamp when needed
            });
        }

        ProcessStatus::Normal
    }
}

impl Vst3Plugin for VstKitPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"VstKitPlug000001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

impl ClapPlugin for VstKitPlugin {
    const CLAP_ID: &'static str = "dev.vstkit.vstkit";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("VstKit audio plugin framework");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Utility];
}

nih_export_vst3!(VstKitPlugin);
nih_export_clap!(VstKitPlugin);
