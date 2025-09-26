use super::super::{
    host_bridge::BoxInvokeFn,
    types::{construct_plugin_box, PluginBoxMetadata},
};
use super::specs;
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
    if let (Some(config), Some(cfg_path)) = (loader.config.as_ref(), loader.config_path.as_ref()) {
        if let (Ok(toml_str), Ok(toml_value)) = (
            std::fs::read_to_string(cfg_path),
            toml::from_str::<TomlValue>(&std::fs::read_to_string(cfg_path).unwrap_or_default()),
        ) {
            let _ = toml_str; // silence unused warning when feature off
            if let Some((lib_name, box_type)) = find_box_by_type_id(config, &toml_value, type_id) {
                if let Some(spec) = specs::get_spec(loader, lib_name, box_type) {
                    if spec.invoke_id.is_none() && super::util::dbg_on() {
                        eprintln!(
                            "[PluginLoaderV2] WARN: no per-Box invoke for {}.{} (type_id={}). Calls will fail with E_PLUGIN until plugin migrates to v2.",
                            lib_name, box_type, type_id
                        );
                    }
                    return spec.invoke_id;
                }
            }
        }
    }
    if let Ok(map) = loader.box_specs.read() {
        for ((_lib, _bt), spec) in map.iter() {
            if let Some(tid) = spec.type_id {
                if tid == type_id {
                    return spec.invoke_id;
                }
            }
        }
    }
    None
}

pub(super) fn metadata_for_type_id(
    loader: &PluginLoaderV2,
    type_id: u32,
) -> Option<PluginBoxMetadata> {
    let config = loader.config.as_ref()?;
    let cfg_path = loader.config_path.as_ref()?;
    let toml_str = std::fs::read_to_string(cfg_path).ok()?;
    let toml_value: TomlValue = toml::from_str(&toml_str).ok()?;
    let (lib_name, box_type) = find_box_by_type_id(config, &toml_value, type_id)?;
    let plugins = loader.plugins.read().ok()?;
    let _plugin = plugins.get(lib_name)?.clone();
    let spec_key = (lib_name.to_string(), box_type.to_string());
    let mut resolved_type = type_id;
    let mut fini_method = None;
    if let Some(spec) = loader.box_specs.read().ok()?.get(&spec_key).cloned() {
        if let Some(tid) = spec.type_id {
            resolved_type = tid;
        }
        if let Some(fini) = spec.fini_method_id {
            fini_method = Some(fini);
        }
    }
    if resolved_type == type_id || fini_method.is_none() {
        if let Some(cfg) = config.get_box_config(lib_name, box_type, &toml_value) {
            if resolved_type == type_id {
                resolved_type = cfg.type_id;
            }
            if fini_method.is_none() {
                fini_method = cfg.methods.get("fini").map(|m| m.method_id);
            }
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
    let config = loader.config.as_ref()?;
    let cfg_path = loader.config_path.as_ref()?;
    let toml_str = std::fs::read_to_string(cfg_path).ok()?;
    let toml_value: TomlValue = toml::from_str(&toml_str).ok()?;
    let (lib_name, box_type) = find_box_by_type_id(config, &toml_value, type_id)?;
    let plugins = loader.plugins.read().ok()?;
    let _plugin = plugins.get(lib_name)?.clone();
    let fini_method_id = if let Some(spec) = loader
        .box_specs
        .read()
        .ok()?
        .get(&(lib_name.to_string(), box_type.to_string()))
    {
        spec.fini_method_id
    } else {
        let box_conf = config.get_box_config(lib_name, box_type, &toml_value)?;
        box_conf.methods.get("fini").map(|m| m.method_id)
    };
    let bx = construct_plugin_box(
        box_type.to_string(),
        type_id,
        super::super::nyash_plugin_invoke_v2_shim,
        instance_id,
        fini_method_id,
    );
    Some(Box::new(bx))
}
