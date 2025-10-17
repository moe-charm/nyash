use std::collections::HashMap;

use crate::backend::vm_types::{VMError, VMValue};
use crate::runtime::host_handle_box::HostHandleBox;
use crate::runtime::method_router_box::host_slot;
use crate::runtime::method_router_box::tables::MAP_HOST_ROUTES;
use crate::runtime::plugin_host_box;
use crate::runtime::plugin_loader_v2::PluginBoxV2;

use super::collections;

pub fn register(map: &mut HashMap<(String, String), super::HandlerFn>) {
    // nyrt.map.size(recv:Map) -> i64
    map.insert(("nyrt.map".into(), "size".into()), |args: &[VMValue]| {
        if args.is_empty() { return Err(VMError::InvalidInstruction("nyrt.map.size requires receiver".into())); }
        match &args[0] {
            VMValue::BoxRef(b) => {
                let debug = crate::runtime::env_gate_box::debug_host_slot();
                if let Some((_route, value)) =
                    host_slot::try_invoke_arc(b, MAP_HOST_ROUTES, "size", &args[1..])
                {
                    if debug {
                        eprintln!("[extern_map] host size -> {:?}", value);
                    }
                    return Ok(value);
                }
                if let Some(plugin_box) = b.as_any().downcast_ref::<PluginBoxV2>() {
                    let out = plugin_host_box::invoke_instance_method(
                        "MapBox",
                        "size",
                        plugin_box.inner.instance_id,
                        &[],
                    );
                    let result = match out {
                        Ok(Some(ret)) => Ok(VMValue::from_nyash_box(ret)),
                        Ok(None) => Ok(VMValue::Void),
                        Err(e) => Err(VMError::InvalidInstruction(format!(
                            "Plugin method MapBox.size failed: {:?}",
                            e
                        ))),
                    };
                    if let Ok(ref val) = result {
                        if debug {
                            eprintln!("[extern_map] plugin size -> {:?}", val);
                        }
                    }
                    return result;
                }
                // plugin path fallbacks intentionally minimal
                Ok(VMValue::Integer(0))
            }
            _ => Err(VMError::TypeError("nyrt.map.size expects MapBox".into())),
        }
    });

    // nyrt.map.keys(recv:Map) -> Array
    map.insert(("nyrt.map".into(), "keys".into()), |args: &[VMValue]| {
        if args.is_empty() { return Err(VMError::InvalidInstruction("nyrt.map.keys requires receiver".into())); }
        match &args[0] {
            VMValue::BoxRef(b) => {
                let debug = crate::runtime::env_gate_box::debug_host_slot();
                if let Some((_route, value)) =
                    host_slot::try_invoke_arc(b, MAP_HOST_ROUTES, "keys", &args[1..])
                {
                    if debug {
                        if let VMValue::BoxRef(arc) = &value {
                            if let Some(handle) = arc.as_any().downcast_ref::<HostHandleBox>() {
                                eprintln!(
                                    "[extern_map] host keys handle id={}",
                                    handle.id
                                );
                            }
                        }
                    }
                    let normalized = collections::normalize_array_value(value)?;
                    if debug {
                        eprintln!("[extern_map] host keys -> {:?}", normalized);
                    }
                    return Ok(normalized);
                }
                if let Some(plugin_box) = b.as_any().downcast_ref::<PluginBoxV2>() {
                    let out = plugin_host_box::invoke_instance_method(
                        "MapBox",
                        "keys",
                        plugin_box.inner.instance_id,
                        &[],
                    );
                    let result = match out {
                        Ok(Some(ret)) => collections::normalize_array_value(VMValue::from_nyash_box(ret)),
                        Ok(None) => Ok(VMValue::Void),
                        Err(e) => Err(VMError::InvalidInstruction(format!(
                            "Plugin method MapBox.keys failed: {:?}",
                            e
                        ))),
                    };
                    if let Ok(ref val) = result {
                        if debug {
                            eprintln!("[extern_map] plugin keys -> {:?}", val);
                        }
                    }
                    return result;
                }
                Err(VMError::InvalidInstruction("Map.keys plugin returned None".into()))
            }
            _ => Err(VMError::TypeError("nyrt.map.keys expects MapBox".into())),
        }
    });

    // nyrt.map.values(recv:Map) -> Array
    map.insert(("nyrt.map".into(), "values".into()), |args: &[VMValue]| {
        if args.is_empty() { return Err(VMError::InvalidInstruction("nyrt.map.values requires receiver".into())); }
        match &args[0] {
            VMValue::BoxRef(b) => {
                let debug = crate::runtime::env_gate_box::debug_host_slot();
                if let Some((_route, value)) =
                    host_slot::try_invoke_arc(b, MAP_HOST_ROUTES, "values", &args[1..])
                {
                    if debug {
                        if let VMValue::BoxRef(arc) = &value {
                            if let Some(handle) = arc.as_any().downcast_ref::<HostHandleBox>() {
                                eprintln!(
                                    "[extern_map] host values handle id={}",
                                    handle.id
                                );
                            }
                        }
                    }
                    let normalized = collections::normalize_array_value(value)?;
                    if debug {
                        eprintln!("[extern_map] host values -> {:?}", normalized);
                    }
                    return Ok(normalized);
                }
                if let Some(plugin_box) = b.as_any().downcast_ref::<PluginBoxV2>() {
                    let out = plugin_host_box::invoke_instance_method(
                        "MapBox",
                        "values",
                        plugin_box.inner.instance_id,
                        &[],
                    );
                    let result = match out {
                        Ok(Some(ret)) => collections::normalize_array_value(VMValue::from_nyash_box(ret)),
                        Ok(None) => Ok(VMValue::Void),
                        Err(e) => Err(VMError::InvalidInstruction(format!(
                            "Plugin method MapBox.values failed: {:?}",
                            e
                        ))),
                    };
                    if let Ok(ref val) = result {
                        if debug {
                            eprintln!("[extern_map] plugin values -> {:?}", val);
                        }
                    }
                    return result;
                }
                Err(VMError::InvalidInstruction("Map.values plugin returned None".into()))
            }
            _ => Err(VMError::TypeError("nyrt.map.values expects MapBox".into())),
        }
    });
}
