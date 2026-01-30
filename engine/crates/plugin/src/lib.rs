//! Plugin crate - nih-plug integration and host glue.
//!
//! This crate bridges the DSP layer to the plugin host via nih-plug,
//! handling VST3/CLAP exports, parameter binding, and the placeholder UI.

mod editor;
mod params;

use std::sync::Arc;

use dsp::Processor;
use nih_plug::prelude::*;

use crate::editor::create_editor;
use crate::params::VstKitParams;

/// Main plugin struct for VstKit.
pub struct VstKitPlugin {
    params: Arc<VstKitParams>,
    processor: Processor,
}

impl Default for VstKitPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(VstKitParams::default()),
            processor: Processor::new(44100.0),
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
        Some(create_editor(self.params.clone()))
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
        // Read parameter once per buffer (real-time safe)
        let gain_db = self.params.gain.smoothed.next();

        // Process audio in-place
        for mut channel_samples in buffer.iter_samples() {
            let gain_db = self.params.gain.smoothed.next();
            let gain_linear = protocol::db_to_linear(gain_db);

            for sample in channel_samples.iter_mut() {
                *sample *= gain_linear;
            }
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
