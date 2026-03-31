use syn::{
    Ident, LitStr, Path, Result, Token, Type,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

/// Returns the last path segment identifier of a type as a string (PascalCase).
///
/// Used for type-name-based slot IDs and duplicate detection.
fn type_last_segment_name(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_else(|| quote::quote!(#ty).to_string()),
        _ => quote::quote!(#ty).to_string(),
    }
}

/// Input structure for `wavecraft_plugin!` macro.
pub(super) struct PluginDef {
    pub(super) name: LitStr,
    /// Ordered list of processor types in the signal chain.
    pub(super) processors: Vec<Type>,
    /// Optional ordered list of tap processor types (default: empty).
    ///
    /// Taps observe audio without modifying it. They must implement `TapProcessor`, not
    /// `Processor`. A type may not appear in both `processors` and `taps`.
    pub(super) taps: Vec<Type>,
    /// Optional crate path for nih-plug integration crate (default: `::wavecraft`).
    /// Use `crate: my_name` only if you've renamed the wavecraft dependency in Cargo.toml.
    pub(super) krate: Option<Path>,
}

impl Parse for PluginDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name = None;
        let mut processors: Option<Vec<Type>> = None;
        let mut taps: Option<Vec<Type>> = None;
        let mut krate = None;

        // Parse key-value pairs
        while !input.is_empty() {
            // Handle `crate` keyword specially (it's a Rust keyword)
            if input.peek(Token![crate]) {
                input.parse::<Token![crate]>()?;
                input.parse::<Token![:]>()?;
                krate = Some(input.parse()?);

                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                }
                continue;
            }

            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "name" => name = Some(input.parse()?),
                "processors" => {
                    let content;
                    syn::bracketed!(content in input);
                    let types = Punctuated::<Type, Token![,]>::parse_terminated(&content)?;
                    if types.is_empty() {
                        return Err(syn::Error::new(
                            key.span(),
                            "processors must contain at least one processor type",
                        ));
                    }
                    processors = Some(types.into_iter().collect());
                }
                "taps" => {
                    let content;
                    syn::bracketed!(content in input);
                    let types = Punctuated::<Type, Token![,]>::parse_terminated(&content)?;
                    let tap_vec: Vec<Type> = types.into_iter().collect();
                    // Reject duplicate tap type names within the taps list.
                    let mut seen_names = std::collections::HashSet::new();
                    for ty in &tap_vec {
                        let name_str = type_last_segment_name(ty);
                        if !seen_names.insert(name_str.clone()) {
                            return Err(syn::Error::new(
                                key.span(),
                                format!(
                                    "duplicate tap type `{}` in taps list — each tap type may \
                                     only appear once",
                                    name_str
                                ),
                            ));
                        }
                    }
                    taps = Some(tap_vec);
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown field: `{}`\n\
                             \n\
                             The wavecraft_plugin! macro only accepts:\n\
                             - name: \"Plugin Name\" (required)\n\
                             - processors: [A, B, C] (required)\n\
                             - taps: [D, E] (optional, for observation-only tap processors)\n\
                             - crate: custom_name (optional, for Cargo renames)",
                            key
                        ),
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
                         processors: [MyGain],\n\
                     }",
                )
            })?,
            processors: processors.ok_or_else(|| {
                input.error(
                    "missing required field: `processors`\n\
                     \n\
                     The processors field defines your DSP processing chain.\n\
                     \n\
                     Example:\n\
                     wavecraft_plugin! {\n\
                         name: \"My Plugin\",\n\
                         processors: [MyGain],\n\
                     }\n\
                     \n\
                     For multiple processors:\n\
                     processors: [InputGain, Filter, OutputGain]",
                )
            })?,
            // taps defaults to empty if not specified
            taps: taps.unwrap_or_default(),
            // Default krate to ::wavecraft if not specified
            krate: krate.or_else(|| Some(syn::parse_quote!(::wavecraft))),
        })
    }
}
