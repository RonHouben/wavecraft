use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

/// Returns the last path segment identifier of a type as a PascalCase string.
///
/// Used as slot ID (e.g., "TestTone", "OscilloscopeTap") in runtime order.
/// Also used for compile-time overlap checks between processors and taps.
///
/// Will be consolidated into `naming.rs` in a later cleanup step.
pub(super) fn type_last_segment_name(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_else(|| quote!(#ty).to_string()),
        _ => quote!(#ty).to_string(),
    }
}

/// Pre-computed cross-cutting invariants derived from the processors and taps lists.
///
/// All fields are computed once in `SharedSymbols::new()` and passed by reference to
/// each code-generation sub-module, avoiding repeated computation and reducing the
/// boilerplate in `generate_plugin_code`.
pub(super) struct SharedSymbols {
    #[allow(dead_code)]
    pub(super) n: usize,
    #[allow(dead_code)]
    pub(super) t: usize,
    pub(super) total_slots: usize,
    pub(super) n_lit: syn::LitInt,
    pub(super) t_lit: syn::LitInt,
    pub(super) total_slots_lit: syn::LitInt,
    pub(super) np1_lit: syn::LitInt,
    pub(super) proc_field_names: Vec<syn::Ident>,
    pub(super) tap_field_names: Vec<syn::Ident>,
    pub(super) tap_scratch_l_names: Vec<syn::Ident>,
    pub(super) tap_scratch_r_names: Vec<syn::Ident>,
    pub(super) tap_capt_l_names: Vec<syn::Ident>,
    pub(super) tap_capt_r_names: Vec<syn::Ident>,
    pub(super) proc_idx_u8: Vec<TokenStream>,
    pub(super) proc_idx_usize: Vec<TokenStream>,
    pub(super) tap_idx_usize: Vec<TokenStream>,
    pub(super) proc_name_str_lits: Vec<syn::LitStr>,
    pub(super) tap_name_str_lits: Vec<syn::LitStr>,
    pub(super) default_tap_boundary_u8: Vec<TokenStream>,
    pub(super) tap_boundary_slot_idx: Vec<TokenStream>,
}

impl SharedSymbols {
    pub(super) fn new(processors: &[Type], taps: &[Type]) -> Self {
        let n = processors.len();
        let t = taps.len();
        let total_slots = n + t;

        let n_lit = syn::LitInt::new(&n.to_string(), proc_macro2::Span::call_site());
        let t_lit = syn::LitInt::new(&t.to_string(), proc_macro2::Span::call_site());
        let total_slots_lit =
            syn::LitInt::new(&total_slots.to_string(), proc_macro2::Span::call_site());
        let np1_lit = syn::LitInt::new(&(n + 1).to_string(), proc_macro2::Span::call_site());

        // ── Processor field names: __proc_0, __proc_1, ... ───────────────────
        let proc_field_names: Vec<syn::Ident> = (0..n)
            .map(|i| syn::Ident::new(&format!("__proc_{}", i), proc_macro2::Span::call_site()))
            .collect();

        // ── u8 / usize index literals ─────────────────────────────────────────
        let proc_idx_u8: Vec<TokenStream> = (0..n)
            .map(|i| {
                let lit =
                    syn::LitInt::new(&format!("{}u8", i), proc_macro2::Span::call_site());
                quote! { #lit }
            })
            .collect();

        let proc_idx_usize: Vec<TokenStream> = (0..n)
            .map(|i| {
                let lit =
                    syn::LitInt::new(&format!("{}usize", i), proc_macro2::Span::call_site());
                quote! { #lit }
            })
            .collect();

        // ── Processor type names (PascalCase strings for slot IDs) ───────────
        let proc_type_names: Vec<String> =
            processors.iter().map(type_last_segment_name).collect();
        let proc_name_str_lits: Vec<syn::LitStr> = proc_type_names
            .iter()
            .map(|s| syn::LitStr::new(s, proc_macro2::Span::call_site()))
            .collect();

        // ── Tap field names and type names ───────────────────────────────────
        let tap_field_names: Vec<syn::Ident> = (0..t)
            .map(|i| syn::Ident::new(&format!("__tap_{}", i), proc_macro2::Span::call_site()))
            .collect();
        let tap_scratch_l_names: Vec<syn::Ident> = (0..t)
            .map(|i| {
                syn::Ident::new(
                    &format!("__tap_{}_scratch_l", i),
                    proc_macro2::Span::call_site(),
                )
            })
            .collect();
        let tap_scratch_r_names: Vec<syn::Ident> = (0..t)
            .map(|i| {
                syn::Ident::new(
                    &format!("__tap_{}_scratch_r", i),
                    proc_macro2::Span::call_site(),
                )
            })
            .collect();
        let tap_idx_usize: Vec<TokenStream> = (0..t)
            .map(|i| {
                let lit =
                    syn::LitInt::new(&format!("{}usize", i), proc_macro2::Span::call_site());
                quote! { #lit }
            })
            .collect();

        // Per-tap local capture variable names: __tap_0_capt_l, __tap_0_capt_r, ...
        let tap_capt_l_names: Vec<syn::Ident> = (0..t)
            .map(|i| {
                syn::Ident::new(
                    &format!("__tap_{}_capt_l", i),
                    proc_macro2::Span::call_site(),
                )
            })
            .collect();
        let tap_capt_r_names: Vec<syn::Ident> = (0..t)
            .map(|i| {
                syn::Ident::new(
                    &format!("__tap_{}_capt_r", i),
                    proc_macro2::Span::call_site(),
                )
            })
            .collect();

        let tap_type_names: Vec<String> = taps.iter().map(type_last_segment_name).collect();
        let tap_name_str_lits: Vec<syn::LitStr> = tap_type_names
            .iter()
            .map(|s| syn::LitStr::new(s, proc_macro2::Span::call_site()))
            .collect();

        // Default boundary = N (all taps start after all processors = final output)
        let default_tap_boundary_u8: Vec<TokenStream> = (0..t)
            .map(|_| {
                let lit = syn::LitInt::new(&format!("{}u8", n), proc_macro2::Span::call_site());
                quote! { #lit }
            })
            .collect();

        // Indices for pending_slots[N + tap_idx]
        let tap_boundary_slot_idx: Vec<TokenStream> = (0..t)
            .map(|i| {
                let idx = n + i;
                let lit =
                    syn::LitInt::new(&format!("{}usize", idx), proc_macro2::Span::call_site());
                quote! { #lit }
            })
            .collect();

        Self {
            n,
            t,
            total_slots,
            n_lit,
            t_lit,
            total_slots_lit,
            np1_lit,
            proc_field_names,
            tap_field_names,
            tap_scratch_l_names,
            tap_scratch_r_names,
            tap_capt_l_names,
            tap_capt_r_names,
            proc_idx_u8,
            proc_idx_usize,
            tap_idx_usize,
            proc_name_str_lits,
            tap_name_str_lits,
            default_tap_boundary_u8,
            tap_boundary_slot_idx,
        }
    }
}
