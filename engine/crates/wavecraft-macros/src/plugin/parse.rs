use syn::{
    Ident, LitStr, Path, Result, Token, Type,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

/// Input structure for `wavecraft_plugin!` macro.
pub(super) struct PluginDef {
    pub(super) name: LitStr,
    /// Ordered list of processor types in the signal chain.
    pub(super) processors: Vec<Type>,
    /// Optional crate path for nih-plug integration crate (default: `::wavecraft`).
    /// Use `crate: my_name` only if you've renamed the wavecraft dependency in Cargo.toml.
    pub(super) krate: Option<Path>,
}

impl Parse for PluginDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name = None;
        let mut processors: Option<Vec<Type>> = None;
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
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown field: `{}`\n\
                             \n\
                             The wavecraft_plugin! macro only accepts:\n\
                             - name: \"Plugin Name\" (required)\n\
                             - processors: [A, B, C] (required)\n\
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
            // Default krate to ::wavecraft if not specified
            krate: krate.or_else(|| Some(syn::parse_quote!(::wavecraft))),
        })
    }
}
