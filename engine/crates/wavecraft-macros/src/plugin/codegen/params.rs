use proc_macro2::TokenStream;
use quote::quote;

/// Generate the `__WavecraftRuntimeParam` enum, `__WavecraftParams` struct,
/// and all associated `impl` blocks (`Default`, `Params`, `SignalChainOrderAccess`).
pub(super) fn build(
    krate: &syn::Path,
    runtime_param_blocks: &[TokenStream],
    s: &super::context::SharedSymbols,
    initial_order_state_slots: &[TokenStream],
) -> TokenStream {
    let n_lit = &s.n_lit;
    let t_lit = &s.t_lit;
    let total_slots_lit = &s.total_slots_lit;
    let proc_idx_u8 = &s.proc_idx_u8;
    let default_tap_boundary_u8 = &s.default_tap_boundary_u8;
    let proc_name_str_lits = &s.proc_name_str_lits;
    let tap_name_str_lits = &s.tap_name_str_lits;

    quote! {
        /// Generated params struct.
        ///
        /// Parameters are discovered at runtime from processor param_specs().
        enum __WavecraftRuntimeParam {
            Float(#krate::__nih::FloatParam),
            Int(#krate::__nih::IntParam),
        }

        impl __WavecraftRuntimeParam {
            fn as_ptr(&self) -> #krate::__nih::ParamPtr {
                use #krate::__nih::Param;
                match self {
                    Self::Float(param) => param.as_ptr(),
                    Self::Int(param) => param.as_ptr(),
                }
            }

            fn modulated_plain_value(&self) -> f32 {
                let ptr = self.as_ptr();
                // SAFETY: ParamPtr originates from `self` and remains valid for this call.
                unsafe { ptr.modulated_plain_value() }
            }
        }

        pub struct __WavecraftParams {
            // Runtime parameter values (flat, all processors in chain order)
            params: ::std::vec::Vec<__WavecraftRuntimeParam>,
            // Runtime IDs aligned with FFI-generated contract IDs (e.g. test_tone_frequency)
            ids: ::std::vec::Vec<::std::string::String>,
            // Optional parameter group names (empty string when none)
            groups: ::std::vec::Vec<::std::string::String>,
            // --- Lock-free SPSC order handoff (shared between UI and audio threads) ---
            // First N slots: processor execution order (u8 index into proc fields).
            // Next T slots: tap boundary values (count of procs before each tap).
            __has_pending_order: ::std::sync::atomic::AtomicBool,
            __pending_slots: ::std::vec::Vec<::std::sync::atomic::AtomicU8>,
            // --- Order state for IPC reads and persistence ---
            // Full unified SignalChainOrder (processors + taps in slot order).
            __order_state: ::std::sync::Mutex<::std::vec::Vec<#krate::__nih::SignalChainSlot>>,
        }

        impl __WavecraftParams {
            fn from_processor_specs() -> Self {
                use #krate::ParamRange;

                let mut params = ::std::vec::Vec::new();
                let mut ids = ::std::vec::Vec::new();
                let mut groups = ::std::vec::Vec::new();

                #(#runtime_param_blocks)*

                // Initialise pending-order slots:
                //   slots[0..N]   = identity processor order (0u8, 1u8, ...)
                //   slots[N..N+T] = default tap boundaries = N (taps after all procs)
                let mut __pending: ::std::vec::Vec<::std::sync::atomic::AtomicU8> =
                    ::std::vec::Vec::with_capacity(#total_slots_lit);
                #(
                    __pending.push(::std::sync::atomic::AtomicU8::new(#proc_idx_u8));
                )*
                // Tap boundary defaults: N = after all processors
                #(
                    __pending.push(::std::sync::atomic::AtomicU8::new(#default_tap_boundary_u8));
                )*

                // Initial order: all processors in declaration order, then all taps.
                let __initial_order: ::std::vec::Vec<#krate::__nih::SignalChainSlot> =
                    ::std::vec![#(#initial_order_state_slots),*];

                Self {
                    params,
                    ids,
                    groups,
                    __has_pending_order: ::std::sync::atomic::AtomicBool::new(false),
                    __pending_slots: __pending,
                    __order_state: ::std::sync::Mutex::new(__initial_order),
                }
            }
        }

        impl ::std::default::Default for __WavecraftParams {
            fn default() -> Self {
                Self::from_processor_specs()
            }
        }

        // Manual Params implementation (can't use derive due to Vec)
        unsafe impl #krate::__nih::Params for __WavecraftParams {
            fn param_map(&self) -> ::std::vec::Vec<(
                ::std::string::String,
                #krate::__nih::ParamPtr,
                ::std::string::String,
            )> {
                self.params
                    .iter()
                    .zip(self.ids.iter())
                    .zip(self.groups.iter())
                    .map(|((param, id), group)| (id.clone(), param.as_ptr(), group.clone()))
                    .collect()
            }

            fn serialize_fields(
                &self,
            ) -> ::std::collections::BTreeMap<::std::string::String, ::std::string::String>
            {
                // Serialize the unified SignalChainOrder as JSON.
                // Key: "signalChainOrder".
                let __order_guard = self
                    .__order_state
                    .lock()
                    .unwrap_or_else(|__e| __e.into_inner());

                // Serialize as [{"id":"TestTone","type":"processor"}, ...]
                let __json = #krate::__internal::serde_json::to_string(&*__order_guard)
                    .unwrap_or_default();

                let mut __map =
                    ::std::collections::BTreeMap::<::std::string::String, ::std::string::String>::new();
                __map.insert("signalChainOrder".to_string(), __json);
                __map
            }

            fn deserialize_fields(
                &self,
                fields: &::std::collections::BTreeMap<::std::string::String, ::std::string::String>,
            ) {
                if let Some(__json) = fields.get("signalChainOrder") {
                    if let Ok(__slots) =
                        #krate::__internal::serde_json::from_str::<
                            ::std::vec::Vec<#krate::__nih::SignalChainSlot>,
                        >(__json)
                    {
                        // Re-use set_order for validation + apply.
                        if let ::std::result::Result::Ok(()) =
                        <Self as #krate::__nih::SignalChainOrderAccess>::set_order(self, __slots) {
                            // Already applied by set_order.
                        } else {
                            eprintln!(
                                "[wavecraft] signalChainOrder restore failed: persisted order \
                                 is invalid. Falling back to default declaration order."
                            );
                        }
                    }
                }
            }
        }

        impl #krate::__nih::SignalChainOrderAccess for __WavecraftParams {
            fn get_order(&self) -> ::std::vec::Vec<#krate::__nih::SignalChainSlot> {
                // Return a clone of the current unified order (processors + taps by type name).
                self.__order_state
                    .lock()
                    .unwrap_or_else(|__e| __e.into_inner())
                    .clone()
            }

            fn set_order(
                &self,
                order: ::std::vec::Vec<#krate::__nih::SignalChainSlot>,
            ) -> ::std::result::Result<(), #krate::__nih::BridgeError> {
                // ── Validation ──────────────────────────────────────────────────────
                // Known processor names (in declaration order)
                let __proc_names: [&str; #n_lit] = [#(#proc_name_str_lits),*];
                // Known tap names (in declaration order)
                let __tap_names: [&str; #t_lit] = [#(#tap_name_str_lits),*];
                let __total: usize = #n_lit + #t_lit;

                if order.len() != __total {
                    return ::std::result::Result::Err(
                        #krate::__nih::BridgeError::InvalidSignalChainOrder {
                            reason: ::std::format!(
                                "expected {} total slots ({} processor(s) + {} tap(s)), got {}",
                                __total, #n_lit, #t_lit, order.len()
                            ),
                        },
                    );
                }

                // Check each slot: id must match a known name, type must match declaration.
                let mut __proc_seen = [false; #n_lit];
                let mut __tap_seen = [false; #t_lit];
                for __slot in order.iter() {
                    match __slot.slot_type {
                        #krate::__nih::SlotType::Processor => {
                            match __proc_names.iter().position(|&__n| __n == __slot.id) {
                                Some(__idx) => {
                                    if __proc_seen[__idx] {
                                        return ::std::result::Result::Err(
                                            #krate::__nih::BridgeError::InvalidSignalChainOrder {
                                                reason: ::std::format!(
                                                    "duplicate processor slot '{}'",
                                                    __slot.id
                                                ),
                                            },
                                        );
                                    }
                                    __proc_seen[__idx] = true;
                                }
                                None => {
                                    return ::std::result::Result::Err(
                                        #krate::__nih::BridgeError::InvalidSignalChainOrder {
                                            reason: ::std::format!(
                                                "unknown processor slot '{}'",
                                                __slot.id
                                            ),
                                        },
                                    );
                                }
                            }
                        }
                        #krate::__nih::SlotType::Tap => {
                            match __tap_names.iter().position(|&__n| __n == __slot.id) {
                                Some(__idx) => {
                                    if __tap_seen[__idx] {
                                        return ::std::result::Result::Err(
                                            #krate::__nih::BridgeError::InvalidSignalChainOrder {
                                                reason: ::std::format!(
                                                    "duplicate tap slot '{}'",
                                                    __slot.id
                                                ),
                                            },
                                        );
                                    }
                                    __tap_seen[__idx] = true;
                                }
                                None => {
                                    return ::std::result::Result::Err(
                                        #krate::__nih::BridgeError::InvalidSignalChainOrder {
                                            reason: ::std::format!(
                                                "unknown tap slot '{}'",
                                                __slot.id
                                            ),
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
                // All slots must be present (no omissions)
                if __proc_seen.iter().any(|&__s| !__s)
                    || __tap_seen.iter().any(|&__s| !__s)
                {
                    return ::std::result::Result::Err(
                        #krate::__nih::BridgeError::InvalidSignalChainOrder {
                            reason: "order is missing one or more declared slots".to_string(),
                        },
                    );
                }

                // ── Resolve ────────────────────────────────────────────────────────
                // Processor execution order: u8 index in declaration order for each proc slot.
                // Tap boundaries: count of Processor slots appearing before each tap in the order.
                let mut __proc_exec: [u8; #n_lit] = [0u8; #n_lit];
                let mut __tap_boundaries: [u8; #t_lit] = [0u8; #t_lit];
                let mut __proc_exec_pos: usize = 0;
                let mut __proc_count_so_far: u8 = 0;
                for __slot in order.iter() {
                    match __slot.slot_type {
                        #krate::__nih::SlotType::Processor => {
                            let __idx = __proc_names
                                .iter()
                                .position(|&__n| __n == __slot.id)
                                .unwrap() as u8;
                            __proc_exec[__proc_exec_pos] = __idx;
                            __proc_exec_pos += 1;
                            __proc_count_so_far += 1;
                        }
                        #krate::__nih::SlotType::Tap => {
                            let __tap_idx = __tap_names
                                .iter()
                                .position(|&__n| __n == __slot.id)
                                .unwrap();
                            __tap_boundaries[__tap_idx] = __proc_count_so_far;
                        }
                    }
                }

                // ── Atomic store ────────────────────────────────────────────────────
                use ::std::sync::atomic::Ordering;
                for (__i, &__b) in __proc_exec.iter().enumerate() {
                    self.__pending_slots[__i].store(__b, Ordering::Release);
                }
                for (__i, &__b) in __tap_boundaries.iter().enumerate() {
                    self.__pending_slots[#n_lit + __i].store(__b, Ordering::Release);
                }
                self.__has_pending_order.store(true, Ordering::Release);
                *self.__order_state.lock().unwrap_or_else(|__e| __e.into_inner()) = order;
                ::std::result::Result::Ok(())
            }
        }
    }
}
