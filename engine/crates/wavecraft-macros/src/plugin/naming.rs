use quote::quote;
use syn::Type;

fn to_snake_case_identifier(name: &str) -> String {
    name.chars()
        .enumerate()
        .flat_map(|(i, c)| {
            if c.is_uppercase() && i > 0 {
                vec!['_', c.to_ascii_lowercase()]
            } else {
                vec![c.to_ascii_lowercase()]
            }
        })
        .collect()
}

pub(super) fn type_prefix(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|segment| to_snake_case_identifier(&segment.ident.to_string()))
            .unwrap_or_else(|| to_snake_case_identifier(&quote!(#ty).to_string())),
        _ => to_snake_case_identifier(&quote!(#ty).to_string()),
    }
}

pub(super) fn processor_id_from_type(processor_type: &Type) -> String {
    type_prefix(processor_type)
}
