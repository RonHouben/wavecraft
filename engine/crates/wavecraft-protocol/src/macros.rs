//! Macros for defining parameters with minimal boilerplate.
//!
//! The `vstkit_params!` macro generates parameter enums and `ParamSet` implementations
//! from a declarative syntax.

/// Define a parameter set with minimal boilerplate.
///
/// This macro generates:
/// - A parameter ID enum (with `#[repr(u32)]`)
/// - A `ParamSet` implementation
/// - Conversion from the ID enum to `ParamId`
///
/// # Syntax
///
/// ```text
/// vstkit_params! {
///     ParamSetName;
///
///     ParameterName {
///         id: 0,
///         name: "Display Name",
///         short_name: "Short",
///         unit: "unit",
///         default: 0.0,
///         min: -10.0,
///         max: 10.0,
///         step: 0.1,
///     },
///
///     // ... more parameters
/// }
/// ```
///
/// # Example
///
/// ```rust
/// use wavecraft_protocol::vstkit_params;
///
/// vstkit_params! {
///     MyParams;
///     
///     Volume {
///         id: 0,
///         name: "Volume",
///         short_name: "Vol",
///         unit: "dB",
///         default: 0.0,
///         min: -60.0,
///         max: 12.0,
///         step: 0.1,
///     },
///     
///     Pan {
///         id: 1,
///         name: "Pan",
///         short_name: "Pan",
///         unit: "",
///         default: 0.0,
///         min: -1.0,
///         max: 1.0,
///         step: 0.01,
///     },
/// }
///
/// // The macro generates:
/// // - enum MyParamsId { Volume = 0, Pan = 1 }
/// // - struct MyParams
/// // - impl ParamSet for MyParams
/// // - impl From<MyParamsId> for ParamId
/// ```
#[macro_export]
macro_rules! vstkit_params {
    // Main pattern: ParamSetName; ParameterName { fields }, ...
    (
        $set_name:ident;
        $(
            $param_name:ident {
                id: $id:expr,
                name: $name:expr,
                short_name: $short_name:expr,
                unit: $unit:expr,
                default: $default:expr,
                min: $min:expr,
                max: $max:expr,
                step: $step:expr $(,)?
            }
        ),* $(,)?
    ) => {
        // Generate the parameter ID enum
        paste::paste! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            #[repr(u32)]
            pub enum [<$set_name Id>] {
                $(
                    $param_name = $id,
                )*
            }

            // Generate conversion to ParamId
            impl ::std::convert::From<[<$set_name Id>]> for $crate::ParamId {
                fn from(id: [<$set_name Id>]) -> Self {
                    $crate::ParamId(id as u32)
                }
            }
        }

        // Generate the parameter set struct
        pub struct $set_name;

        // Generate ParamSet implementation
        paste::paste! {
            impl $crate::ParamSet for $set_name {
                type Id = [<$set_name Id>];

                const SPECS: &'static [$crate::ParamSpec] = &[
                    $(
                        $crate::ParamSpec {
                            id: $crate::ParamId($id),
                            name: $name,
                            short_name: $short_name,
                            unit: $unit,
                            default: $default,
                            min: $min,
                            max: $max,
                            step: $step,
                        },
                    )*
                ];

                fn spec(id: Self::Id) -> ::std::option::Option<&'static $crate::ParamSpec> {
                    Self::SPECS.iter().find(|s| s.id.0 == id as u32)
                }

                fn iter() -> impl ::std::iter::Iterator<Item = &'static $crate::ParamSpec> {
                    Self::SPECS.iter()
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::{ParamId, ParamSet};

    // Test the macro with a simple parameter set
    vstkit_params! {
        TestParams;

        Gain {
            id: 0,
            name: "Gain",
            short_name: "Gain",
            unit: "dB",
            default: 0.0,
            min: -24.0,
            max: 24.0,
            step: 0.1,
        },

        Frequency {
            id: 1,
            name: "Frequency",
            short_name: "Freq",
            unit: "Hz",
            default: 1000.0,
            min: 20.0,
            max: 20000.0,
            step: 1.0,
        },
    }

    #[test]
    fn test_generated_enum() {
        let gain_id = TestParamsId::Gain;
        let freq_id = TestParamsId::Frequency;

        assert_eq!(gain_id as u32, 0);
        assert_eq!(freq_id as u32, 1);
    }

    #[test]
    fn test_param_id_conversion() {
        let gain_id: ParamId = TestParamsId::Gain.into();
        assert_eq!(gain_id.0, 0);
    }

    #[test]
    fn test_param_set_specs() {
        assert_eq!(TestParams::SPECS.len(), 2);
        assert_eq!(TestParams::count(), 2);

        let gain_spec = &TestParams::SPECS[0];
        assert_eq!(gain_spec.name, "Gain");
        assert_eq!(gain_spec.unit, "dB");
        assert_eq!(gain_spec.default, 0.0);
    }

    #[test]
    fn test_param_set_spec_lookup() {
        let gain_spec = TestParams::spec(TestParamsId::Gain);
        assert!(gain_spec.is_some());
        assert_eq!(gain_spec.unwrap().name, "Gain");

        let freq_spec = TestParams::spec(TestParamsId::Frequency);
        assert!(freq_spec.is_some());
        assert_eq!(freq_spec.unwrap().name, "Frequency");
    }

    #[test]
    fn test_param_set_iter() {
        let names: Vec<&str> = TestParams::iter().map(|s| s.name).collect();
        assert_eq!(names, vec!["Gain", "Frequency"]);
    }
}
