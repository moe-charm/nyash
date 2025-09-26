use super::specs;
use super::PluginLoaderV2;
use crate::bid::{BidError, BidResult};
use crate::runtime::plugin_loader_v2::enabled::{errors, host_bridge, types};

pub(super) fn prebirth_singletons(loader: &PluginLoaderV2) -> BidResult<()> {
    let config = loader.config.as_ref().ok_or(BidError::PluginError)?;
    let cfg_path = loader.config_path.as_deref().unwrap_or("nyash.toml");
    let toml_content = errors::from_fs(std::fs::read_to_string(cfg_path))?;
    let toml_value: toml::Value = errors::from_toml(toml::from_str(&toml_content))?;
    for (lib_name, lib_def) in &config.libraries {
        for box_name in &lib_def.boxes {
            if let Some(bc) = config.get_box_config(lib_name, box_name, &toml_value) {
                if bc.singleton {
                    let _ = ensure_singleton_handle(loader, lib_name, box_name);
                }
            }
        }
    }
    Ok(())
}

pub(super) fn ensure_singleton_handle(
    loader: &PluginLoaderV2,
    lib_name: &str,
    box_type: &str,
) -> BidResult<()> {
    if loader
        .singletons
        .read()
        .unwrap()
        .contains_key(&(lib_name.to_string(), box_type.to_string()))
    {
        return Ok(());
    }
    let cfg_path = loader.config_path.as_deref().unwrap_or("nyash.toml");
    let toml_value: toml::Value =
        toml::from_str(&std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?)
            .map_err(|_| BidError::PluginError)?;
    let config = loader.config.as_ref().ok_or(BidError::PluginError)?;
    let plugins = loader.plugins.read().unwrap();
    let _plugin = plugins.get(lib_name).ok_or(BidError::PluginError)?;
    let type_id = if let Some(spec) = loader
        .box_specs
        .read()
        .unwrap()
        .get(&(lib_name.to_string(), box_type.to_string()))
    {
        spec.type_id
            .unwrap_or_else(|| config.box_types.get(box_type).copied().unwrap_or(0))
    } else {
        let box_conf = config
            .get_box_config(lib_name, box_type, &toml_value)
            .ok_or(BidError::InvalidType)?;
        box_conf.type_id
    };
    let tlv_args = crate::runtime::plugin_ffi_common::encode_empty_args();
    let (_status, _, out_vec) = host_bridge::invoke_alloc(
        super::super::nyash_plugin_invoke_v2_shim,
        type_id,
        0,
        0,
        &tlv_args,
    );
    if out_vec.len() < 4 {
        return Err(BidError::PluginError);
    }
    let instance_id = u32::from_le_bytes([out_vec[0], out_vec[1], out_vec[2], out_vec[3]]);
    let fini_id = if let Some(spec) = loader
        .box_specs
        .read()
        .unwrap()
        .get(&(lib_name.to_string(), box_type.to_string()))
    {
        spec.fini_method_id
    } else {
        let box_conf = config
            .get_box_config(lib_name, box_type, &toml_value)
            .ok_or(BidError::InvalidType)?;
        box_conf.methods.get("fini").map(|m| m.method_id)
    };
    let handle = Arc::new(types::PluginHandleInner {
        type_id,
        invoke_fn: super::super::nyash_plugin_invoke_v2_shim,
        instance_id,
        fini_method_id: fini_id,
        finalized: std::sync::atomic::AtomicBool::new(false),
    });
    loader
        .singletons
        .write()
        .unwrap()
        .insert((lib_name.to_string(), box_type.to_string()), handle);
    crate::runtime::cache_versions::bump_version(&format!("BoxRef:{}", box_type));
    Ok(())
}

use std::sync::Arc;
