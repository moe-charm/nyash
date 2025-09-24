use super::super::host_bridge::BoxInvokeFn;
use super::super::types::NyashTypeBoxFfi;
use super::util::dbg_on;
use super::PluginLoaderV2;
use crate::bid::{BidError, BidResult};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub(crate) struct LoadedBoxSpec {
    pub(crate) type_id: Option<u32>,
    pub(crate) methods: HashMap<String, MethodSpec>,
    pub(crate) fini_method_id: Option<u32>,
    pub(crate) invoke_id: Option<BoxInvokeFn>,
    pub(crate) resolve_fn: Option<extern "C" fn(*const std::os::raw::c_char) -> u32>,
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
    } else if dbg_on() {
        eprintln!(
            "[PluginLoaderV2] WARN: TypeBox present but no invoke_id for {}.{} — plugin should export per-Box invoke",
            lib_name, box_type
        );
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
