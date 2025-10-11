use super::super::host_bridge::BoxInvokeFn;
use super::super::types::{NyashTypeBoxFfi, NyashTypeBoxFinalFfi};
use super::super::host_bridge::FinalInvokeFn;
use super::util::dbg_on;
use super::PluginLoaderV2;
use crate::bid::{BidError, BidResult};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub(crate) struct LoadedBoxSpec {
    pub(crate) type_id: Option<u32>,
    pub(crate) capabilities: Option<u64>,
    pub(crate) methods: HashMap<String, MethodSpec>,
    pub(crate) fini_method_id: Option<u32>,
    pub(crate) invoke_id: Option<BoxInvokeFn>,
    pub(crate) resolve_fn: Option<extern "C" fn(*const std::os::raw::c_char) -> u32>,
    // Optional Final ABI per-Box invoke (Phase A)
    pub(crate) final_invoke: Option<FinalInvokeFn>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MethodSpec {
    pub(crate) method_id: u32,
    pub(crate) returns_result: bool,
}

pub(super) fn record_typebox_spec(
    loader: &PluginLoaderV2,
    lib_name: &str,
    box_type: &str,
    typebox: &NyashTypeBoxFfi,
) -> BidResult<()> {
    // Validate ABI tag 'TYBX' (0x54594258) and struct size
    let abi_ok = typebox.abi_tag == 0x5459_4258
        && typebox.struct_size as usize >= std::mem::size_of::<NyashTypeBoxFfi>();
    if !abi_ok {
        if dbg_on() {
            eprintln!(
                "[PluginLoaderV2] WARN: invalid TypeBox ABI for {}.{} (abi_tag=0x{:08x} size={} need>={})",
                lib_name,
                box_type,
                typebox.abi_tag,
                typebox.struct_size,
                std::mem::size_of::<NyashTypeBoxFfi>()
            );
        }
        return Ok(());
    }

    if let Some(invoke_id) = typebox.invoke_id {
        let key = (lib_name.to_string(), box_type.to_string());
        let mut map = loader
            .box_specs
            .write()
            .map_err(|_| BidError::PluginError)?;
        let entry = map.entry(key).or_insert_with(LoadedBoxSpec::default);
        entry.invoke_id = Some(invoke_id);
        entry.resolve_fn = typebox.resolve;
        entry.capabilities = Some(typebox.capabilities);
    } else if dbg_on() {
        eprintln!(
            "[PluginLoaderV2] WARN: TypeBox present but no invoke_id for {}.{} — plugin should export per-Box invoke",
            lib_name, box_type
        );
    }
    Ok(())
}

/// Record optional Final ABI spec if present (env-gated in loader)
pub(super) fn record_typebox_final_spec(
    loader: &PluginLoaderV2,
    lib_name: &str,
    box_type: &str,
    final_box: &NyashTypeBoxFinalFfi,
) -> BidResult<()> {
    // Basic sanity: struct size must be at least our header
    let ok = final_box.struct_size as usize >= std::mem::size_of::<NyashTypeBoxFinalFfi>();
    if !ok {
        return Ok(());
    }
    if let Some(final_invoke) = final_box.invoke_final {
        let key = (lib_name.to_string(), box_type.to_string());
        let mut map = loader
            .box_specs
            .write()
            .map_err(|_| BidError::PluginError)?;
        let entry = map.entry(key).or_insert_with(LoadedBoxSpec::default);
        entry.final_invoke = Some(final_invoke);
    }
    Ok(())
}

pub(super) fn ingest_box_specs_from_nyash_box(
    loader: &PluginLoaderV2,
    lib_name: &str,
    box_names: &[String],
    nyash_box_toml_path: &Path,
) {
    if !nyash_box_toml_path.exists() {
        return;
    }
    let Ok(text) = std::fs::read_to_string(nyash_box_toml_path) else {
        return;
    };
    let Ok(doc) = toml::from_str::<toml::Value>(&text) else {
        return;
    };
    if let Ok(mut map) = loader.box_specs.write() {
        for box_type in box_names {
            let key = (lib_name.to_string(), box_type.to_string());
            let mut spec = map.get(&key).cloned().unwrap_or_default();
            if let Some(tid) = doc
                .get(box_type)
                .and_then(|v| v.get("type_id"))
                .and_then(|v| v.as_integer())
            {
                spec.type_id = Some(tid as u32);
                super::util::dbg_once(
                    &format!("spec_tid:{}:{}", lib_name, box_type),
                    &format!(
                        "[PluginLoaderV2] spec ingest: {}.{} type_id={} from {}",
                        lib_name,
                        box_type,
                        tid,
                        nyash_box_toml_path.display()
                    ),
                );
            }
            if let Some(fini) = doc
                .get(box_type)
                .and_then(|v| v.get("lifecycle"))
                .and_then(|v| v.get("fini"))
                .and_then(|v| v.get("id"))
                .and_then(|v| v.as_integer())
            {
                spec.fini_method_id = Some(fini as u32);
            }
            if let Some(birth) = doc
                .get(box_type)
                .and_then(|v| v.get("lifecycle"))
                .and_then(|v| v.get("birth"))
                .and_then(|v| v.get("id"))
                .and_then(|v| v.as_integer())
            {
                spec.methods.insert(
                    "birth".to_string(),
                    MethodSpec {
                        method_id: birth as u32,
                        returns_result: false,
                    },
                );
            }
            if let Some(methods) = doc
                .get(box_type)
                .and_then(|v| v.get("methods"))
                .and_then(|v| v.as_table())
            {
                for (mname, mdef) in methods.iter() {
                    if let Some(id) = mdef
                        .get("id")
                        .and_then(|v| v.as_integer())
                        .map(|x| x as u32)
                    {
                        let returns_result = mdef
                            .get("returns_result")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        spec.methods.insert(
                            mname.to_string(),
                            MethodSpec {
                                method_id: id,
                                returns_result,
                            },
                        );
                        super::util::dbg_once(
                            &format!("spec_mid:{}:{}:{}", lib_name, box_type, mname),
                            &format!(
                                "[PluginLoaderV2] spec ingest: {}.{} method {} -> id={} returns_result={}",
                                lib_name,
                                box_type,
                                mname,
                                id,
                                returns_result
                            ),
                        );
                    }
                }
            }
            map.insert(key, spec);
        }
    }
}

pub(super) fn get_spec<'a>(
    loader: &'a PluginLoaderV2,
    lib_name: &str,
    box_type: &str,
) -> Option<LoadedBoxSpec> {
    loader.box_specs.read().ok().and_then(|map| {
        map.get(&(lib_name.to_string(), box_type.to_string()))
            .cloned()
    })
}

impl super::PluginLoaderV2 {
    /// Ingest specs from a given spec file path for the specified library.
    pub(crate) fn ingest_specs_from_file(
        &self,
        lib_name: &str,
        box_names: &[String],
        spec_path: &std::path::Path,
    ) {
        ingest_box_specs_from_nyash_box(self, lib_name, box_names, spec_path);
    }

    /// Register a static (in-process) specification for a plugin box.
    ///
    /// This allows the runtime to know `type_id`/methods/invoke pointer without
    /// relying on external nyash.toml or dlopen.
    /// Duplicate registrations merge; existing fields are preserved.
    pub(crate) fn register_static_box(
        &self,
        lib_name: &str,
        box_type: &str,
        type_id: Option<u32>,
        capabilities: Option<u64>,
        fini_method_id: Option<u32>,
        methods: &[(&str, u32, bool)],
        _invoke_any: Option<*const ()>,
    ) {
        if let Ok(mut map) = self.box_specs.write() {
            let key = (lib_name.to_string(), box_type.to_string());
            let entry = map.entry(key).or_insert_with(LoadedBoxSpec::default);
            if entry.type_id.is_none() { entry.type_id = type_id; }
            if entry.capabilities.is_none() { entry.capabilities = capabilities; }
            if entry.fini_method_id.is_none() { entry.fini_method_id = fini_method_id; }
            
            for (name, id, retres) in methods.iter().copied() {
                entry.methods.insert(name.to_string(), MethodSpec { method_id: id, returns_result: retres });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::plugin_loader_v2::enabled::types;

    extern "C" fn dummy_final_invoke(
        _type_id: u32,
        _method_id: u32,
        _instance_id: u32,
        _args: *const types::NyValueFfi,
        _argc: usize,
        _out: *mut types::NyResultFfi,
    ) -> i32 {
        0
    }

    #[test]
    fn record_final_spec_registers_invoke() {
        let loader = PluginLoaderV2::new();
        let ffi = NyashTypeBoxFinalFfi {
            abi_tag: 0x5446494E, // 'TFIN' (arbitrary)
            version: 1,
            struct_size: std::mem::size_of::<NyashTypeBoxFinalFfi>() as u16,
            invoke_final: Some(dummy_final_invoke),
            get_method_meta: None,
            get_all_methods: None,
            get_type_info: None,
        };
        let _ = record_typebox_final_spec(&loader, "libX", "MyBox", &ffi);
        let spec = get_spec(&loader, "libX", "MyBox").expect("spec missing");
        assert!(spec.final_invoke.is_some());
    }

    #[test]
    fn record_final_spec_ignores_when_struct_too_small() {
        let loader = PluginLoaderV2::new();
        let ffi = NyashTypeBoxFinalFfi {
            abi_tag: 0,
            version: 0,
            struct_size: 4, // too small
            invoke_final: Some(dummy_final_invoke),
            get_method_meta: None,
            get_all_methods: None,
            get_type_info: None,
        };
        let _ = record_typebox_final_spec(&loader, "libY", "OtherBox", &ffi);
        let spec = get_spec(&loader, "libY", "OtherBox");
        assert!(spec.is_none() || spec.unwrap().final_invoke.is_none());
    }
}

// Separate impl to avoid touching existing block
impl super::PluginLoaderV2 {
    /// Register a static spec and per-Box invoke pointer.
    /// Same as `register_static_box`, but also records the `invoke` pointer used by
    /// the enabled loader to bypass dynamic lookup. Existing `invoke_id` is preserved
    /// when already present.
    pub(crate) fn register_static_with_invoke(
        &self,
        lib_name: &str,
        box_type: &str,
        type_id: Option<u32>,
        capabilities: Option<u64>,
        fini_method_id: Option<u32>,
        methods: &[(&str, u32, bool)],
        invoke: Option<super::super::host_bridge::BoxInvokeFn>,
    ) {
        if let Ok(mut map) = self.box_specs.write() {
            let key = (lib_name.to_string(), box_type.to_string());
            let entry = map.entry(key).or_insert_with(LoadedBoxSpec::default);
            if entry.type_id.is_none() { entry.type_id = type_id; }
            if entry.capabilities.is_none() { entry.capabilities = capabilities; }
            if entry.fini_method_id.is_none() { entry.fini_method_id = fini_method_id; }
            if entry.invoke_id.is_none() { entry.invoke_id = invoke; }
            for (name, id, retres) in methods.iter().copied() {
                entry.methods.insert(name.to_string(), MethodSpec { method_id: id, returns_result: retres });
            }
        }
    }
}
