//! Procedural macro for generating complete plugin implementations from DSL.
//!
//! Simplified API (0.9.0): Only requires `name` and `signal` properties.
//! Vendor and URL metadata are automatically derived from Cargo.toml.
//! Plugin email is not exposed in the DSL and defaults to an empty string.

use proc_macro::TokenStream;
#[path = "plugin/codegen.rs"]
mod codegen;
#[path = "plugin/metadata.rs"]
mod metadata;
#[path = "plugin/naming.rs"]
mod naming;
#[path = "plugin/parse.rs"]
mod parse;
#[path = "plugin/runtime_params.rs"]
mod runtime_params;

use self::parse::PluginDef;
use syn::{Result, parse_macro_input};

pub fn wavecraft_plugin_impl(input: TokenStream) -> TokenStream {
    let plugin_def = parse_macro_input!(input as PluginDef);

    match expand_wavecraft_plugin(plugin_def) {
        Ok(expanded) => TokenStream::from(expanded),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

fn expand_wavecraft_plugin(plugin_def: PluginDef) -> Result<proc_macro2::TokenStream> {
    let name = &plugin_def.name;
    let signal_type = &plugin_def.signal;

    let signal_processors = parse::parse_signal_chain_processors(signal_type)?;

    // Default krate to ::wavecraft if not specified (should already be set by Parse)
    let krate = plugin_def
        .krate
        .unwrap_or_else(|| syn::parse_quote!(::wavecraft));

    let processor_param_mappings = metadata::processor_param_mappings(&signal_processors, &krate);

    let processor_info_entries = metadata::processor_info_entries(&signal_processors, &krate);

    let runtime_param_blocks = runtime_params::runtime_param_blocks(&signal_processors, &krate);

    let vendor = metadata::derive_vendor();
    let url = metadata::derive_url();
    let vst3_id = metadata::generate_vst3_id(&name.value());
    let clap_id = metadata::derive_clap_id();

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

    let expanded = codegen::generate_plugin_code(codegen::CodegenInput {
        name,
        signal_type,
        krate: &krate,
        runtime_param_blocks: &runtime_param_blocks,
        processor_param_mappings: &processor_param_mappings,
        processor_info_entries: &processor_info_entries,
        vendor,
        url,
        vst3_id: &vst3_id,
        clap_id: &clap_id,
    });

    Ok(expanded)
}

#[cfg(test)]
mod tests {
    use super::{expand_wavecraft_plugin, naming};
    use quote::quote;
    use syn::{Expr, Type, parse_quote};

    #[test]
    fn parses_signal_chain_processor_types() {
        let signal: Expr = parse_quote!(SignalChain![Oscillator, InputGain, OutputGain]);

        let processors = super::parse::parse_signal_chain_processors(&signal)
            .expect("signal chain should parse");

        assert_eq!(processors.len(), 3);
    }

    #[test]
    fn derives_snake_case_processor_id_from_type_name() {
        let processor_type: Type = parse_quote!(OscilloscopeTap);
        let id = naming::type_prefix(&processor_type);

        assert_eq!(id, "oscilloscope_tap");
    }

    #[test]
    fn derives_id_from_path_terminal_segment() {
        let processor_type: Type = parse_quote!(my::dsp::InputGain);
        let id = naming::type_prefix(&processor_type);

        assert_eq!(id, "input_gain");
    }

    #[test]
    fn derives_instance_id_prefixes_for_repeated_types() {
        let processor_types: Vec<Type> = vec![
            parse_quote!(Gain),
            parse_quote!(Gain),
            parse_quote!(my::dsp::Gain),
            parse_quote!(OutputGain),
        ];

        let prefixes = naming::instance_id_prefixes(&processor_types);

        assert_eq!(prefixes, vec!["gain", "gain_2", "gain_3", "output_gain"]);
    }

    #[test]
    fn generated_param_map_uses_prefixed_runtime_ids_instead_of_param_indexes() {
        let input_tokens = quote! {
            name: "Test Plugin",
            signal: SignalChain![Oscillator],
        };

        let plugin_def: super::parse::PluginDef =
            syn::parse2(input_tokens).expect("plugin definition should parse");
        let output = expand_wavecraft_plugin(plugin_def).expect("plugin should expand");
        let generated = output.to_string();
        let normalized = generated
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();

        assert!(
            !normalized.contains("format!(\"param_{}\",idx)"),
            "generated code should not use indexed placeholder parameter IDs"
        );
        assert!(
            normalized.contains("format!(\"{}_{}\",\"oscillator\",spec.id_suffix)"),
            "generated code should derive runtime IDs from processor prefix + param suffix"
        );
        assert!(
            normalized.contains("apply_plain_values("),
            "generated code should apply live plain values when building processor params"
        );
        assert!(
            normalized.contains("__WavecraftRuntimeParam::Int(")
                && normalized.contains("IntParam::new(param_name"),
            "generated code should use IntParam for stepped/enum params to expose step_count"
        );
        assert!(
            normalized.contains("with_value_to_string(::std::sync::Arc::new(move|value|"),
            "generated enum params should expose display labels through value_to_string"
        );
        assert!(
            normalized.contains("Bypassed<Oscillator>"),
            "generated runtime parameter discovery should include bypass wrapper params"
        );
        assert!(
            normalized.contains("spec.id_suffix==\"bypass\""),
            "generated runtime parameter naming should special-case bypass display names"
        );
    }

    #[test]
    fn generated_param_map_uses_unique_runtime_ids_for_repeated_processor_types() {
        let input_tokens = quote! {
            name: "Test Plugin",
            signal: SignalChain![Gain, Gain],
        };

        let plugin_def: super::parse::PluginDef =
            syn::parse2(input_tokens).expect("plugin definition should parse");
        let output = expand_wavecraft_plugin(plugin_def).expect("plugin should expand");
        let generated = output.to_string();
        let normalized = generated
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();

        assert!(
            normalized.contains("format!(\"{}_{}\",\"gain\",spec.id_suffix)"),
            "first repeated processor instance should keep legacy-compatible 'gain' prefix"
        );
        assert!(
            normalized.contains("format!(\"{}_{}\",\"gain_2\",spec.id_suffix)"),
            "second repeated processor instance should use deterministic unique 'gain_2' prefix"
        );
        assert!(
            normalized.contains("id:\"gain\".to_string()")
                && normalized.contains("id:\"gain_2\".to_string()"),
            "processor metadata IDs should be unique for repeated processor types"
        );
    }
}
