//! ProcessorParams derive macro implementation.
//!
//! Generates `ProcessorParams` trait implementation from struct fields
//! annotated with `#[param(...)]` attributes.

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Expr, ExprLit, Fields, Lit, parse_macro_input};

pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_derive(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_derive(input: &DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;

    // Only works on structs
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "ProcessorParams can only be derived for structs with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "ProcessorParams can only be derived for structs",
            ));
        }
    };

    // Extract parameter specifications from fields
    let mut param_specs = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().expect("named fields");
        let field_name_str = field_name.to_string();

        // Look for #[param(...)] attribute
        let param_attr = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("param"));

        if let Some(attr) = param_attr {
            let spec = parse_param_attr(&field_name_str, attr)?;
            param_specs.push(spec);
        }
    }

    // Generate the const array of ParamSpec
    let spec_count = param_specs.len();
    let spec_items = param_specs.iter().map(|spec| {
        let name = &spec.name;
        let id_suffix = &spec.id_suffix;
        let range = &spec.range;
        let default = spec.default;
        let unit = &spec.unit;
        let group = &spec.group;

        let group_token = if let Some(g) = group {
            quote! { Some(#g) }
        } else {
            quote! { None }
        };

        quote! {
            ::wavecraft::ParamSpec {
                name: #name,
                id_suffix: #id_suffix,
                range: #range,
                default: #default,
                unit: #unit,
                group: #group_token,
            }
        }
    });

    Ok(quote! {
        impl ::wavecraft::ProcessorParams for #struct_name {
            fn param_specs() -> &'static [::wavecraft::ParamSpec] {
                static SPECS: [::wavecraft::ParamSpec; #spec_count] = [
                    #(#spec_items),*
                ];
                &SPECS
            }
        }
    })
}

struct ParamSpecData {
    name: String,
    id_suffix: String,
    range: TokenStream,
    default: f64,
    unit: String,
    group: Option<String>,
}

fn parse_param_attr(field_name: &str, attr: &syn::Attribute) -> syn::Result<ParamSpecData> {
    let mut range_min: Option<f64> = None;
    let mut range_max: Option<f64> = None;
    let mut range_factor: Option<f64> = None;
    let mut default: Option<f64> = None;
    let mut unit: Option<String> = None;
    let mut group: Option<String> = None;

    // Parse nested meta items using parse_nested_meta
    attr.parse_nested_meta(|meta| {
        let ident = meta
            .path
            .get_ident()
            .ok_or_else(|| meta.error("Expected identifier"))?;

        match ident.to_string().as_str() {
            "range" => {
                // Parse range = "MIN..=MAX"
                let value: Expr = meta.value()?.parse()?;
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = value
                {
                    let range_str = lit_str.value();
                    let parts: Vec<&str> = range_str.split("..=").collect();
                    if parts.len() == 2 {
                        range_min = Some(
                            parts[0]
                                .trim()
                                .parse()
                                .map_err(|_| meta.error("Invalid range min"))?,
                        );
                        range_max = Some(
                            parts[1]
                                .trim()
                                .parse()
                                .map_err(|_| meta.error("Invalid range max"))?,
                        );
                    } else {
                        return Err(meta.error("Range must be in format \"MIN..=MAX\""));
                    }
                } else {
                    return Err(meta.error("Expected string literal for range"));
                }
            }
            "factor" => {
                let value: Expr = meta.value()?.parse()?;
                match value {
                    Expr::Lit(ExprLit {
                        lit: Lit::Float(lit_float),
                        ..
                    }) => {
                        range_factor = Some(lit_float.base10_parse()?);
                    }
                    Expr::Lit(ExprLit {
                        lit: Lit::Int(lit_int),
                        ..
                    }) => {
                        range_factor = Some(lit_int.base10_parse::<i64>()? as f64);
                    }
                    _ => return Err(meta.error("Expected number for factor")),
                }
            }
            "default" => {
                let value: Expr = meta.value()?.parse()?;
                match value {
                    Expr::Lit(ExprLit {
                        lit: Lit::Float(lit_float),
                        ..
                    }) => {
                        default = Some(lit_float.base10_parse()?);
                    }
                    Expr::Lit(ExprLit {
                        lit: Lit::Int(lit_int),
                        ..
                    }) => {
                        default = Some(lit_int.base10_parse::<i64>()? as f64);
                    }
                    _ => return Err(meta.error("Expected number for default")),
                }
            }
            "unit" => {
                let value: Expr = meta.value()?.parse()?;
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = value
                {
                    unit = Some(lit_str.value());
                } else {
                    return Err(meta.error("Expected string literal for unit"));
                }
            }
            "group" => {
                let value: Expr = meta.value()?.parse()?;
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = value
                {
                    group = Some(lit_str.value());
                } else {
                    return Err(meta.error("Expected string literal for group"));
                }
            }
            _ => return Err(meta.error("Unknown param attribute")),
        }
        Ok(())
    })?;

    // Validate required fields
    let min =
        range_min.ok_or_else(|| syn::Error::new_spanned(attr, "Missing 'range' attribute"))?;
    let max =
        range_max.ok_or_else(|| syn::Error::new_spanned(attr, "Missing 'range' attribute"))?;
    let default_val = default.unwrap_or((min + max) / 2.0);
    let unit_str = unit.unwrap_or_default();

    // Generate display name (Title Case from snake_case)
    let display_name = field_name.to_case(Case::Title);

    // Generate range TokenStream
    let range_tokens = if let Some(factor) = range_factor {
        quote! {
            ::wavecraft::ParamRange::Skewed {
                min: #min,
                max: #max,
                factor: #factor,
            }
        }
    } else {
        quote! {
            ::wavecraft::ParamRange::Linear {
                min: #min,
                max: #max,
            }
        }
    };

    Ok(ParamSpecData {
        name: display_name,
        id_suffix: field_name.to_string(),
        range: range_tokens,
        default: default_val,
        unit: unit_str,
        group,
    })
}
