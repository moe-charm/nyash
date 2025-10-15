use super::super::{
    host_bridge::BoxInvokeFn,
    types::{construct_plugin_box, PluginBoxMetadata},
};
use super::specs;
use libloading::Symbol;
use super::PluginLoaderV2;
use crate::box_trait::NyashBox;
use crate::config::nyash_toml_v2::NyashConfigV2;

type TomlValue = toml::Value;

fn find_box_by_type_id<'a>(
    config: &'a NyashConfigV2,
    toml_value: &'a TomlValue,
    type_id: u32,
) -> Option<(&'a str, &'a str)> {
    for (lib_name, lib_def) in &config.libraries {
        for box_name in &lib_def.boxes {
            if let Some(box_conf) = config.get_box_config(lib_name, box_name, toml_value) {
                if box_conf.type_id == type_id {
                    return Some((lib_name.as_str(), box_name.as_str()));
                }
            }
        }
    }
    None
}

pub(super) fn box_invoke_fn_for_type_id(
    loader: &PluginLoaderV2,
    type_id: u32,
) -> Option<BoxInvokeFn> {
    if let Some(config) = loader.config.as_ref() {
        let cfg_path = loader.config_path_str();
        if let (Ok(toml_str), Ok(toml_value)) = (
            std::fs::read_to_string(cfg_path),
            toml::from_str::<TomlValue>(&std::fs::read_to_string(cfg_path).unwrap_or_default()),
        ) {
            let _ = toml_str; // silence unused warning when feature off
            if let Some((lib_name, box_type)) = find_box_by_type_id(config, &toml_value, type_id) {
                if let Some(spec) = specs::get_spec(loader, lib_name, box_type) {
                    if super::util::dbg_on() {
                        eprintln!(
                            "[PluginLoaderV2] metadata: mapping type_id={} -> {}.{} invoke_present={}",
                            type_id, lib_name, box_type, spec.invoke_id.is_some()
                        );
                    }
                    if let Some(inv) = spec.invoke_id { return Some(inv); }
                }
                // Probe TypeBox symbol on-demand and record when missing
                if probe_and_record_typebox_invoke(loader, lib_name, box_type) {
                    if let Some(spec2) = specs::get_spec(loader, lib_name, box_type) {
                        if let Some(inv) = spec2.invoke_id { return Some(inv); }
                    }
                }
            }
        }
    }
    if let Ok(map) = loader.box_specs.read() {
        for ((_lib, _bt), spec) in map.iter() {
            if let Some(tid) = spec.type_id {
                if tid == type_id {
                    if super::util::dbg_on() {
                        eprintln!("[PluginLoaderV2] metadata(fallback): found tid={} invoke_present={}", tid, spec.invoke_id.is_some());
                    }
                    return spec.invoke_id;
                }
            }
        }
    }
    None
}

/// Utility: probe TypeBox symbol for (lib, box) and record its invoke id into loader.box_specs.
/// Returns true on success, false if symbol was not found or on errors.
pub(super) fn probe_and_record_typebox_invoke(
    loader: &PluginLoaderV2,
    lib_name: &str,
    box_type: &str,
) -> bool {
    if let Ok(plugins) = loader.plugins.read() {
        if let Some(loaded) = plugins.get(lib_name) {
            let sym_name = format!("nyash_typebox_{}\0", box_type);
            unsafe {
                if let Ok(tb_sym) = loaded
                    ._lib
                    .get::<Symbol<&super::super::types::NyashTypeBoxFfi>>(sym_name.as_bytes())
                {
                    if super::util::dbg_on() {
                        eprintln!(
                            "[PluginLoaderV2] metadata(probe): recorded invoke for {}.{} via symbol '{}'",
                            lib_name,
                            box_type,
                            sym_name.trim_end_matches('\0')
                        );
                    }
                    if let Ok(mut map) = loader.box_specs.write() {
                        let key = (lib_name.to_string(), box_type.to_string());
                        let entry = map.entry(key).or_default();
                        entry.invoke_id = tb_sym.invoke_id;
                    }
                    return true;
                } else if super::util::dbg_on() {
                    eprintln!(
                        "[PluginLoaderV2] metadata(probe): TypeBox not found for {}.{} (looked for '{}')",
                        lib_name,
                        box_type,
                        sym_name.trim_end_matches('\0')
                    );
                }
            }
        }
    }
    false
}

pub(super) fn metadata_for_type_id(
    loader: &PluginLoaderV2,
    type_id: u32,
) -> Option<PluginBoxMetadata> {
    if let Ok(map) = loader.box_specs.read() {
        for ((lib, btype), spec) in map.iter() {
            if let Some(tid) = spec.type_id {
                if tid == type_id {
                    return Some(PluginBoxMetadata {
                        lib_name: lib.clone(),
                        box_type: btype.clone(),
                        type_id: tid,
                        invoke_fn: super::super::nyash_plugin_invoke_v2_shim,
                        fini_method_id: spec.fini_method_id,
                    });
                }
            }
        }
    }
    let config = loader.config.as_ref()?;
    let cfg_path = loader.config_path_str();
    let toml_str = std::fs::read_to_string(cfg_path).ok()?;
    let toml_value: TomlValue = toml::from_str(&toml_str).ok()?;
    let (lib_name, box_type) = find_box_by_type_id(config, &toml_value, type_id)?;
    let spec_key = (lib_name.to_string(), box_type.to_string());
    let mut resolved_type = type_id;
    let mut fini_method = None;
    if let Some(spec) = loader.box_specs.read().ok()?.get(&spec_key).cloned() {
        if let Some(tid) = spec.type_id { resolved_type = tid; }
        if let Some(fini) = spec.fini_method_id { fini_method = Some(fini); }
    }
    if resolved_type == type_id || fini_method.is_none() {
        if let Some(cfg) = config.get_box_config(lib_name, box_type, &toml_value) {
            if resolved_type == type_id { resolved_type = cfg.type_id; }
            if fini_method.is_none() { fini_method = cfg.methods.get("fini").map(|m| m.method_id); }
        }
    }
    Some(PluginBoxMetadata {
        lib_name: lib_name.to_string(),
        box_type: box_type.to_string(),
        type_id: resolved_type,
        invoke_fn: super::super::nyash_plugin_invoke_v2_shim,
        fini_method_id: fini_method,
    })
}

pub(super) fn construct_existing_instance(
    loader: &PluginLoaderV2,
    type_id: u32,
    instance_id: u32,
) -> Option<Box<dyn NyashBox>> {
    if let Ok(map) = loader.box_specs.read() {
        for ((_lib, btype), spec) in map.iter() {
            if let Some(tid) = spec.type_id {
                if tid == type_id {
                    let bx = construct_plugin_box(
                        btype.clone(),
                        type_id,
                        super::super::nyash_plugin_invoke_v2_shim,
                        instance_id,
                        spec.fini_method_id,
                    );
                    return Some(Box::new(bx));
                }
            }
        }
    }
    let config = loader.config.as_ref()?;
    let cfg_path = loader.config_path_str();
    let toml_str = std::fs::read_to_string(cfg_path).ok()?;
    let toml_value: TomlValue = toml::from_str(&toml_str).ok()?;
    let (lib_name, box_type) = find_box_by_type_id(config, &toml_value, type_id)?;
    let fini_method_id = if let Some(spec) = loader.box_specs.read().ok()?.get(&(lib_name.to_string(), box_type.to_string())) { spec.fini_method_id } else {
        let box_conf = config.get_box_config(lib_name, box_type, &toml_value)?;
        box_conf.methods.get("fini").map(|m| m.method_id)
    };
    let bx = construct_plugin_box(box_type.to_string(), type_id, super::super::nyash_plugin_invoke_v2_shim, instance_id, fini_method_id);
    Some(Box::new(bx))
}
