use super::naming;
use quote::quote;
use syn::{Path, Type};

pub(super) fn runtime_param_blocks(
    signal_processors: &[Type],
    krate: &Path,
) -> Vec<proc_macro2::TokenStream> {
    signal_processors
        .iter()
        .map(|processor_type| {
            let id_prefix = naming::type_prefix(processor_type);
            quote! {
                {
                    let specs = <<#processor_type as #krate::Processor>::Params as #krate::ProcessorParams>::param_specs();

                    for spec in specs.iter() {
                        match &spec.range {
                            ParamRange::Linear { min, max } => {
                                let range = #krate::__nih::FloatRange::Linear {
                                    min: *min as f32,
                                    max: *max as f32,
                                };
                                params.push(
                                    __WavecraftRuntimeParam::Float(
                                        #krate::__nih::FloatParam::new(spec.name, spec.default as f32, range)
                                            .with_unit(spec.unit)
                                    )
                                );
                            }
                            ParamRange::Skewed { min, max, factor } => {
                                let range = #krate::__nih::FloatRange::Skewed {
                                    min: *min as f32,
                                    max: *max as f32,
                                    factor: *factor as f32,
                                };
                                params.push(
                                    __WavecraftRuntimeParam::Float(
                                        #krate::__nih::FloatParam::new(spec.name, spec.default as f32, range)
                                            .with_unit(spec.unit)
                                    )
                                );
                            }
                            ParamRange::Stepped { min, max } => {
                                let default = (spec.default as i32).clamp(*min, *max);
                                params.push(
                                    __WavecraftRuntimeParam::Int(
                                        #krate::__nih::IntParam::new(
                                            spec.name,
                                            default,
                                            #krate::__nih::IntRange::Linear {
                                                min: *min,
                                                max: *max,
                                            },
                                        )
                                        .with_unit(spec.unit)
                                    )
                                );
                            }
                            ParamRange::Enum { variants } => {
                                let enum_max = variants.len().saturating_sub(1) as i32;
                                let default = (spec.default as i32).clamp(0, enum_max);
                                let labels_for_display = *variants;
                                let labels_for_parse = *variants;

                                params.push(
                                    __WavecraftRuntimeParam::Int(
                                        #krate::__nih::IntParam::new(
                                            spec.name,
                                            default,
                                            #krate::__nih::IntRange::Linear {
                                                min: 0,
                                                max: enum_max,
                                            },
                                        )
                                        .with_value_to_string(::std::sync::Arc::new(move |value| {
                                            labels_for_display
                                                .get(value as usize)
                                                .copied()
                                                .unwrap_or("")
                                                .to_string()
                                        }))
                                        .with_string_to_value(::std::sync::Arc::new(move |input| {
                                            let trimmed = input.trim();

                                            if let Some(index) = labels_for_parse
                                                .iter()
                                                .position(|label| label.eq_ignore_ascii_case(trimmed))
                                            {
                                                return Some(index as i32);
                                            }

                                            trimmed
                                                .parse::<i32>()
                                                .ok()
                                                .filter(|value| (0..=enum_max).contains(value))
                                        }))
                                        .with_unit(spec.unit)
                                    )
                                );
                            }
                        }

                        ids.push(format!("{}_{}", #id_prefix, spec.id_suffix));
                        groups.push(spec.group.unwrap_or_default().to_string());
                    }
                }
            }
        })
        .collect()
}
