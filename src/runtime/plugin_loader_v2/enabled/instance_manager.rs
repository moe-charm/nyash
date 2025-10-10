//! Instance management for plugin boxes

use crate::bid::{BidError, BidResult};
use crate::box_trait::NyashBox;
use crate::runtime::plugin_loader_v2::enabled::loader::util::dbg_on;
use crate::runtime::plugin_loader_v2::enabled::{PluginLoaderV2, types::{PluginBoxV2, PluginHandleInner}};
use std::sync::Arc;

impl PluginLoaderV2 {
    /// Create a new plugin box instance
    pub fn create_box(
        &self,
        box_type: &str,
        _args: &[Box<dyn NyashBox>],
    ) -> BidResult<Box<dyn NyashBox>> {
        // Non-recursive: directly call plugin 'birth' and construct PluginBoxV2

        // Fast-path: prefer specs (ingested from nyash_box.toml / TypeBox FFI) for type/method IDs.
        // This avoids strict dependency on a fully-loaded nyash.toml when partial plugin loads are used.
        let (type_id, birth_id_opt, fini_id) = if let Ok(map) = self.box_specs.read() {
            if let Some((_, spec)) = map.iter().find(|((_lib, bt), _)| bt == &box_type) {
                if let Some(tid) = spec.type_id {
                    let birth = spec
                        .methods
                        .get("birth")
                        .map(|m| m.method_id)
                        .or_else(|| spec.resolve_fn.and_then(|rf| {
                            std::ffi::CString::new("birth")
                                .ok()
                                .map(|s| rf(s.as_ptr()))
                                .filter(|&mid| mid != 0)
                        }));
                    (tid, birth, spec.fini_method_id)
                } else {
                    resolve_box_ids_optional(self, box_type)?
                }
            } else {
                resolve_box_ids_optional(self, box_type)?
            }
        } else {
            resolve_box_ids_optional(self, box_type)?
        };

        // Try to get per-Box invoke function directly from spec (lib+box) to avoid
        // dependency on type_id→invoke mapping when config is missing.
        let mut direct_invoke: Option<super::host_bridge::BoxInvokeFn> = None;
        let mut lib_name_for_box: Option<String> = None;
        // Prefer loader.config when available for lib_name
        if let Some(cfg) = self.config.as_ref() {
            if let Some((lib, _)) = cfg.find_library_for_box(box_type) {
                lib_name_for_box = Some(lib.to_string());
            }
        }
        // Fallback: ask global BoxFactoryRegistry for provider mapping
        if lib_name_for_box.is_none() {
            let reg = crate::runtime::get_global_registry();
            if let Some(crate::runtime::BoxProvider::Plugin(lib)) = reg.get_provider(box_type) {
                lib_name_for_box = Some(lib);
            }
        }
        if let Some(lib) = lib_name_for_box.as_ref() {
            if let Ok(map) = self.box_specs.read() {
                if let Some(spec) = map.get(&(lib.clone(), box_type.to_string())) {
                    direct_invoke = spec.invoke_id;
                }
            }
            if direct_invoke.is_none() {
                // Fallback: probe symbol from loaded library now
                if let Ok(plugins) = self.plugins.read() {
                    if let Some(loaded) = plugins.get(lib) {
                        let sym_name = format!("nyash_typebox_{}\0", box_type);
                        unsafe {
                            if let Ok(tb_sym) = loaded._lib.get::<libloading::Symbol<&super::types::NyashTypeBoxFfi>>(sym_name.as_bytes()) {
                                let inv = tb_sym.invoke_id;
                                direct_invoke = inv;
                                // Record into specs for future
                                if let Ok(mut specs) = self.box_specs.write() {
                                    let key = (lib.clone(), box_type.to_string());
                                    let entry = specs.entry(key).or_default();
                                    entry.invoke_id = inv;
                                    entry.type_id = Some(type_id);
                                }
                                if dbg_on() {
                                    eprintln!(
                                        "[PluginLoaderV2] TypeBox FFI fallback: recorded invoke for {}.{} (symbol '{}', invoke_id={:?})",
                                        lib,
                                        box_type,
                                        sym_name.trim_end_matches("\\0"),
                                        inv
                                    );
                                }
                            } else if dbg_on() {
                                eprintln!(
                                    "[PluginLoaderV2] TypeBox FFI not found for {}.{} (looked for '{}')",
                                    lib,
                                    box_type,
                                    sym_name.trim_end_matches("\\0")
                                );
                            }
                        }
                    }
                }
            }
        }
        // Final fallback: resolve invoke by type_id or deduce lib by loaded plugins
        if direct_invoke.is_none() {
            if let Some(inv) = self.box_invoke_fn_for_type_id(type_id) {
                if dbg_on() { eprintln!("[PluginLoaderV2] resolved invoke by type_id={} (no lib name)", type_id); }
                direct_invoke = Some(inv);
            } else if lib_name_for_box.is_none() {
                if let Ok(plugins) = self.plugins.read() {
                    for (lib, loaded) in plugins.iter() {
                        if loaded.box_types.iter().any(|bt| bt == box_type) {
                            if dbg_on() { eprintln!("[PluginLoaderV2] deduced library '{}' for box_type='{}' via loaded set", lib, box_type); }
                            if let Some(spec) = self.box_specs.read().ok().and_then(|m| m.get(&(lib.clone(), box_type.to_string())).cloned()) {
                                direct_invoke = spec.invoke_id;
                            }
                            if direct_invoke.is_none() {
                                // Probe symbol now (deduced lib) and record
                                let sym_name = format!("nyash_typebox_{}\0", box_type);
                                unsafe {
                                    if let Ok(tb_sym) = loaded._lib.get::<libloading::Symbol<&super::types::NyashTypeBoxFfi>>(sym_name.as_bytes()) {
                                        direct_invoke = tb_sym.invoke_id;
                                        if let Ok(mut specs) = self.box_specs.write() {
                                            let key = (lib.clone(), box_type.to_string());
                                            let entry = specs.entry(key).or_default();
                                            entry.invoke_id = tb_sym.invoke_id;
                                            entry.type_id = Some(type_id);
                                        }
                                        if dbg_on() { eprintln!("[PluginLoaderV2] TypeBox FFI probe (deduced lib): {}.{}", lib, box_type); }
                                    } else if dbg_on() {
                                        eprintln!("[PluginLoaderV2] TypeBox FFI not found for {}.{} (looked for '{}')", lib, box_type, sym_name.trim_end_matches('\0'));
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }


        // Call birth when available; otherwise synthesize no-op instance id (0)
        let mut instance_id: u32 = 0;
        if let Some(birth_id) = birth_id_opt {
            if dbg_on() {
                eprintln!(
                    "[PluginLoaderV2] invoking birth: box_type={} type_id={} birth_id={}",
                    box_type, type_id, birth_id
                );
            }
            // Encode provided constructor args for birth (if any)
            let tlv = crate::runtime::codec::TlvCodecBox::default().encode_args(_args);
            let (code, out_len, out_buf) = if let Some(box_invoke) = direct_invoke {
                // Direct per-Box call (no type_id dispatch)
                let mut out = vec![0u8; 1024];
                let mut out_len: usize = out.len();
                let code = (box_invoke)(0, birth_id, tlv.as_ptr(), tlv.len(), out.as_mut_ptr(), &mut out_len);
                (code, out_len, out)
            } else {
                super::host_bridge::invoke_alloc(
                    super::super::nyash_plugin_invoke_v2_shim,
                    type_id,
                    birth_id,
                    0,
                    &tlv,
                )
            };
            if dbg_on() {
                eprintln!(
                    "[PluginLoaderV2] create_box: box_type={} type_id={} birth_id={} code={} out_len={}",
                    box_type, type_id, birth_id, code, out_len
                );
                if out_len > 0 {
                    eprintln!(
                        "[PluginLoaderV2] create_box: out[0..min(8)]={:02x?}",
                        &out_buf[..out_len.min(8)]
                    );
                }
            }
            if code != 0 { return Err(BidError::PluginError); }
            // Accept both legacy raw-4-byte (instance_id) and TLV handle(tag=8,size=8)
            if out_len == 4 {
                let mut b = [0u8;4];
                b.copy_from_slice(&out_buf[..4]);
                instance_id = u32::from_le_bytes(b);
            } else if out_len >= 8 {
                if let Some((tag, sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(&out_buf[..out_len]) {
                    if tag != 8 || sz != 8 { return Err(BidError::PluginError); }
                    if let Some((_ret_tid, inst)) = crate::runtime::plugin_ffi_common::decode::plugin_handle(payload) {
                        instance_id = inst;
                    } else {
                        return Err(BidError::PluginError);
                    }
                } else {
                    return Err(BidError::PluginError);
                }
            } else {
                return Err(BidError::PluginError);
            }
        } else {
            // No birth provided: synthesize no-op (migration window)
            let warn = crate::runtime::env_gate_box::bool_any(&["NYASH_WARN_PLUGIN_NO_BIRTH","HAKO_WARN_PLUGIN_NO_BIRTH"]);
            if warn || dbg_on() {
                eprintln!("[PluginLoaderV2] info: box_type='{}' has no birth(); treating as no-op", box_type);
            }
            instance_id = 0;
        }

        let bx = PluginBoxV2 {
            box_type: box_type.to_string(),
            inner: Arc::new(PluginHandleInner {
                type_id,
                invoke_fn: super::super::nyash_plugin_invoke_v2_shim,
                instance_id,
                fini_method_id: fini_id,
                finalized: std::sync::atomic::AtomicBool::new(false),
            }),
        };

        // Diagnostics: register for leak tracking (optional)
        crate::runtime::leak_tracker::register_plugin(box_type, instance_id);
        Ok(Box::new(bx))
    }

    /// Shutdown singletons: finalize and clear all singleton handles
    pub fn shutdown_singletons(&self) {
        let mut map = self.singletons.write().unwrap();
        for (_, handle) in map.drain() {
            if let Ok(inner) = Arc::try_unwrap(handle) {
                inner.finalize_now();
            }
        }
    }
}

/// Resolve box IDs (type_id, birth_id, fini_id) from configuration or specs
fn resolve_box_ids(
    loader: &PluginLoaderV2,
    box_type: &str,
) -> BidResult<(u32, u32, Option<u32>)> {
    let (mut type_id_opt, mut birth_id_opt, mut fini_id) = (None, None, None);

    // Try config mapping first (when available) — be tolerant if cached TOML is missing
    if let Some(cfg) = loader.config.as_ref() {
        let toml_value_opt = loader.get_toml_value().ok();
        if let Some(toml_value) = toml_value_opt {
            if let Some((lib_name, _)) = cfg.find_library_for_box(box_type) {
                if let Some(box_conf) = cfg.get_box_config(lib_name, box_type, &toml_value) {
                    // Prefer spec-ingested type_id when available to avoid stale/incorrect config
                    let mut resolved_tid = box_conf.type_id;
                    // Sanity override for core boxes (dev): guard against stale/misparsed config
                    resolved_tid = match (box_type, resolved_tid) {
                        ("ArrayBox", tid) if tid != 12 => 12,
                        ("MapBox", tid) if tid != 11 => 11,
                        ("StringBox", tid) if tid != 13 => 13,
                        _ => resolved_tid,
                    };
                    if let Ok(map) = loader.box_specs.read() {
                        if let Some(spec) = map.get(&(lib_name.to_string(), box_type.to_string())) {
                            if let Some(tid) = spec.type_id {
                                resolved_tid = tid;
                            }
                        }
                    }
                    type_id_opt = Some(resolved_tid);
                    birth_id_opt = box_conf.methods.get("birth").map(|m| m.method_id);
                    fini_id = box_conf.methods.get("fini").map(|m| m.method_id);
                    if dbg_on() {
                        eprintln!(
                            "[PluginLoaderV2] resolve_box_ids(cfg): {} -> lib={} type_id={} birth_id={:?} fini_id={:?}",
                            box_type, lib_name, resolved_tid, birth_id_opt, fini_id
                        );
                    }
                }
            }
        }
    }

    // Fallback: use TypeBox FFI spec if config is missing for this box
    if type_id_opt.is_none() || birth_id_opt.is_none() {
        if let Ok(map) = loader.box_specs.read() {
            // Find any spec that matches this box_type
            if let Some(((lib, _), spec)) = map.iter().find(|((_, bt), _)| bt == &box_type) {
                if type_id_opt.is_none() {
                    type_id_opt = spec.type_id;
                }
                if birth_id_opt.is_none() {
                    if let Some(ms) = spec.methods.get("birth") {
                        birth_id_opt = Some(ms.method_id);
                    } else if let Some(res_fn) = spec.resolve_fn {
                        if let Ok(cstr) = std::ffi::CString::new("birth") {
                            let mid = res_fn(cstr.as_ptr());
                            if mid != 0 {
                                birth_id_opt = Some(mid);
                            }
                        }
                    }
                }
                if dbg_on() {
                    eprintln!(
                        "[PluginLoaderV2] resolve_box_ids(spec): {} -> lib={} type_id={:?} birth_id={:?}",
                        box_type, lib, type_id_opt, birth_id_opt
                    );
                }
            }
        }
    }

    // Final fallback: try reading nyash.toml/hako.toml directly when loader.config is None
    if type_id_opt.is_none() {
        for cfg_path in ["nyash.toml", "hako.toml", "hakorune.toml"].iter() {
            if let Ok(cfg) = crate::config::nyash_toml_v2::NyashConfigV2::from_file(cfg_path) {
                if let Some((lib_name, _)) = cfg.find_library_for_box(box_type) {
                    if let Ok(raw) = std::fs::read_to_string(cfg_path) {
                        if let Ok(toml_val) = toml::from_str::<toml::Value>(&raw) {
                            if let Some(bc) = cfg.get_box_config(lib_name, box_type, &toml_val) {
                                type_id_opt = Some(bc.type_id);
                                birth_id_opt = birth_id_opt.or(bc.methods.get("birth").map(|m| m.method_id));
                                fini_id = fini_id.or(bc.methods.get("fini").map(|m| m.method_id));
                                if dbg_on() { eprintln!("[PluginLoaderV2] resolve_box_ids(file): {} -> lib={} type_id={} birth_id={:?}", box_type, lib_name, bc.type_id, birth_id_opt); }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    let type_id = type_id_opt.ok_or(BidError::InvalidType)?;
    let birth_id = birth_id_opt.ok_or(BidError::InvalidMethod)?;
    
    Ok((type_id, birth_id, fini_id))
}

/// Optional-birth resolver: returns birth_id as Option
fn resolve_box_ids_optional(
    loader: &PluginLoaderV2,
    box_type: &str,
) -> BidResult<(u32, Option<u32>, Option<u32>)> {
    // Treat method id 0 as a valid birth id (plugins commonly use 0).
    // Only omit birth when we cannot resolve it at all.
    match resolve_box_ids(loader, box_type) {
        Ok((type_id, birth_id, fini_id)) => Ok((type_id, Some(birth_id), fini_id)),
        Err(_) => {
            // Fallback path when resolution fails entirely: resolve type_id even if birth_id is absent
            let (mut ty_opt, mut fi_opt) = (None, None);
            if let Some(cfg) = loader.config.as_ref() {
                if let Some((lib_name, _)) = cfg.find_library_for_box(box_type) {
                    if let Some(toml_value) = loader.cached_toml.clone() {
                        if let Some(box_conf) = cfg.get_box_config(lib_name, box_type, &toml_value) {
                            ty_opt = Some(box_conf.type_id);
                            fi_opt = box_conf.methods.get("fini").map(|m| m.method_id);
                        }
                    }
                }
            }
            let ty = ty_opt.ok_or(BidError::InvalidType)?;
            Ok((ty, None, fi_opt))
        }
    }
}
