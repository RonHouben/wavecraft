use super::naming;
use quote::quote;
use syn::{Path, Type};

/// Generate a deterministic VST3 ID from package name and plugin name.
///
/// Uses CARGO_PKG_NAME instead of vendor for:
/// - Stability: package names are canonical identifiers
/// - Uniqueness: enforced by Cargo/crates.io conventions
/// - Simplicity: one less parameter to manage
pub(super) fn generate_vst3_id(name: &str) -> proc_macro2::TokenStream {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let package_name = env!("CARGO_PKG_NAME");

    let mut hasher = DefaultHasher::new();
    format!("{}{}", package_name, name).hash(&mut hasher);
    let hash = hasher.finish();

    // Convert hash to 16 bytes
    let bytes: [u8; 16] = [
        (hash >> 56) as u8,
        (hash >> 48) as u8,
        (hash >> 40) as u8,
        (hash >> 32) as u8,
        (hash >> 24) as u8,
        (hash >> 16) as u8,
        (hash >> 8) as u8,
        hash as u8,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0, // Padding
    ];

    quote! { [#(#bytes),*] }
}

pub(super) fn derive_vendor() -> &'static str {
    let authors = env!("CARGO_PKG_AUTHORS");
    authors.split(',').next().unwrap_or("Unknown").trim()
}

pub(super) fn derive_url() -> &'static str {
    let homepage = env!("CARGO_PKG_HOMEPAGE");
    if homepage.is_empty() {
        env!("CARGO_PKG_REPOSITORY")
    } else {
        homepage
    }
}

pub(super) fn derive_clap_id() -> String {
    let package_name = env!("CARGO_PKG_NAME");
    format!("com.{}", package_name.replace('-', "_"))
}

pub(super) fn processor_param_mappings(
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
                    params.extend(specs
                        .iter()
                        .map(|spec| #krate::__internal::param_spec_to_info(spec, #id_prefix)));
                }
            }
        })
        .collect()
}

pub(super) fn processor_info_entries(
    signal_processors: &[Type],
    krate: &Path,
) -> Vec<proc_macro2::TokenStream> {
    signal_processors
        .iter()
        .map(|processor_type| {
            let processor_id = naming::processor_id_from_type(processor_type);
            quote! {
                #krate::__internal::ProcessorInfo {
                    id: #processor_id.to_string(),
                }
            }
        })
        .collect()
}
