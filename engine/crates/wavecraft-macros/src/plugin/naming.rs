use quote::quote;
use std::collections::HashMap;
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

pub(super) fn instance_id_prefixes(processor_types: &[Type]) -> Vec<String> {
    let mut total_counts = HashMap::<String, usize>::new();
    for processor_type in processor_types {
        let base_id = type_prefix(processor_type);
        *total_counts.entry(base_id).or_insert(0) += 1;
    }

    let mut seen_counts = HashMap::<String, usize>::new();
    processor_types
        .iter()
        .map(|processor_type| {
            let base_id = type_prefix(processor_type);
            let total = total_counts.get(&base_id).copied().unwrap_or(0);
            let seen = seen_counts.entry(base_id.clone()).or_insert(0);
            *seen += 1;

            if total <= 1 || *seen == 1 {
                base_id
            } else {
                format!("{}_{}", base_id, seen)
            }
        })
        .collect()
}

pub(super) fn processor_display_name_from_type(processor_type: &Type) -> String {
    let raw = match processor_type {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_else(|| quote!(#processor_type).to_string()),
        _ => quote!(#processor_type).to_string(),
    };

    let mut display = String::with_capacity(raw.len() + 4);
    let mut prev_lower_or_digit = false;

    for ch in raw.chars() {
        let is_upper = ch.is_ascii_uppercase();
        if is_upper && prev_lower_or_digit {
            display.push(' ');
        }
        display.push(ch);
        prev_lower_or_digit = ch.is_ascii_lowercase() || ch.is_ascii_digit();
    }

    display
}
