use syn::{
    Expr, Ident, LitStr, Path, Result, Token, Type,
    parse::Parser,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
};

/// Input structure for `wavecraft_plugin!` macro.
pub(super) struct PluginDef {
    pub(super) name: LitStr,
    pub(super) signal: Expr,
    /// Optional crate path for nih-plug integration crate (default: `::wavecraft`).
    /// Use `crate: my_name` only if you've renamed the wavecraft dependency in Cargo.toml.
    pub(super) krate: Option<Path>,
}

impl Parse for PluginDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name = None;
        let mut signal = None;
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
                "signal" => signal = Some(input.parse()?),
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown field: `{}`\n\
                             \n\
                             The wavecraft_plugin! macro only accepts:\n\
                             - name: \"Plugin Name\" (required)\n\
                             - signal: SignalChain![...] (required)\n\
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

        let signal = signal.ok_or_else(|| {
            input.error(
                "missing required field: `signal`\n\
                 \n\
                 The signal field defines your DSP processing chain.\n\
                 \n\
                 Example:\n\
                 wavecraft_plugin! {\n\
                     name: \"My Plugin\",\n\
                     signal: SignalChain![MyGain],\n\
                 }\n\
                 \n\
                 For multiple processors:\n\
                 signal: SignalChain![InputGain, Filter, OutputGain]",
            )
        })?;

        // Validate signal is wrapped in SignalChain! (not a bare identifier)
        if let Expr::Path(ref path) = signal
            && path.path.segments.len() == 1
        {
            let span = signal.span();
            return Err(syn::Error::new(
                span,
                "signal must use `SignalChain!` wrapper.\n\
                 \n\
                 Did you mean:\n\
                 signal: SignalChain![YourProcessor]\n\
                 \n\
                 Or for multiple processors:\n\
                 signal: SignalChain![A, B, C]\n\
                 \n\
                 Note: Bare processor names are not allowed. Always wrap in SignalChain![]",
            ));
        }

        Ok(PluginDef {
            name: name.ok_or_else(|| {
                input.error(
                    "missing required field: `name`\n\
                     \n\
                     Example:\n\
                     wavecraft_plugin! {\n\
                         name: \"My Plugin\",\n\
                         signal: SignalChain![MyGain],\n\
                     }",
                )
            })?,
            signal,
            // Default krate to ::wavecraft if not specified
            krate: krate.or_else(|| Some(syn::parse_quote!(::wavecraft))),
        })
    }
}

pub(super) fn parse_signal_chain_processors(signal: &Expr) -> Result<Vec<Type>> {
    let expr_macro = match signal {
        Expr::Macro(expr_macro) => expr_macro,
        _ => {
            return Err(syn::Error::new(
                signal.span(),
                "signal must use SignalChain![...] macro syntax",
            ));
        }
    };

    let is_signal_chain = expr_macro
        .mac
        .path
        .segments
        .last()
        .map(|segment| segment.ident == "SignalChain")
        .unwrap_or(false);

    if !is_signal_chain {
        return Err(syn::Error::new(
            expr_macro.mac.path.span(),
            "signal must use SignalChain![...]",
        ));
    }

    let parser = Punctuated::<Type, Token![,]>::parse_terminated;
    let processors = parser.parse2(expr_macro.mac.tokens.clone())?;

    if processors.is_empty() {
        return Err(syn::Error::new(
            expr_macro.mac.tokens.span(),
            "SignalChain! must contain at least one processor type",
        ));
    }

    Ok(processors.into_iter().collect())
}
