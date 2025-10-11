//! Static plugins metadata registration (Phase 15.7)
//!
//! Registers specs from hako_box.toml embedded in plugin folders and
//! records type_id and method ids in the loader, so type/handle lookups
//! work even before dynamic config/loader are active.

use serde::Deserialize;

#[derive(Deserialize)]
struct BoxHeader { type_name: String, type_id: u32, provider: String }
#[derive(Deserialize)]
struct MethodSpecToml { name: String, slot: u32, #[allow(dead_code)] arity: Option<u32> }
#[derive(Deserialize)]
struct BoxSpecToml { r#box: BoxHeader, #[serde(default)] methods: Vec<MethodSpecToml> }

fn parse_box_spec(toml_str: &str) -> Option<(String, String, u32, Vec<(String, u32, bool)>)> {
    let spec: BoxSpecToml = toml::from_str(toml_str).ok()?;
    let methods: Vec<(String, u32, bool)> = spec
        .methods
        .iter()
        .map(|m| (m.name.clone(), m.slot, false))
        .collect();
    Some((spec.r#box.provider, spec.r#box.type_name, spec.r#box.type_id, methods))
}

fn register_from_toml(toml_str: &str) {
    let Some((provider, type_name, type_id, methods)) = parse_box_spec(toml_str) else { return; };
    let host = crate::runtime::get_global_plugin_host();
    let h = match host.read() { Ok(v) => v, Err(_) => return };
    let methods_ref: Vec<(&str, u32, bool)> = methods.iter().map(|(n, id, rr)| (n.as_str(), *id, *rr)).collect();
    h.register_static_box("libnyash_array_plugin.so", &type_name, Some(type_id), None, None, &methods_ref, None);
}

pub fn register_static_plugins() {
    // Array
    let arr = include_str!("../../../plugins/nyash-array-plugin/hako_box.toml");
    if let Some((_prov, type_name, type_id, methods)) = parse_box_spec(arr) {
        let host = crate::runtime::get_global_plugin_host();
        if let Ok(h) = host.read() {
            let methods_ref: Vec<(&str, u32, bool)> = methods.iter().map(|(n, id, rr)| (n.as_str(), *id, *rr)).collect();
            h.register_static_box("libnyash_array_plugin.so", &type_name, Some(type_id), None, None, &methods_ref, None);
        };
    }
    // Map
    let map = include_str!("../../../plugins/nyash-map-plugin/hako_box.toml");
    if let Some((_prov, type_name, type_id, methods)) = parse_box_spec(map) {
        let host = crate::runtime::get_global_plugin_host();
        if let Ok(h) = host.read() {
            let methods_ref: Vec<(&str, u32, bool)> = methods.iter().map(|(n, id, rr)| (n.as_str(), *id, *rr)).collect();
            h.register_static_box("libnyash_map_plugin.so", &type_name, Some(type_id), None, None, &methods_ref, None);
        };
    }
    // String
    let strb = include_str!("../../../plugins/nyash-string-plugin/hako_box.toml");
    if let Some((_prov, type_name, type_id, methods)) = parse_box_spec(strb) {
        let host = crate::runtime::get_global_plugin_host();
        if let Ok(h) = host.read() {
            let methods_ref: Vec<(&str, u32, bool)> = methods.iter().map(|(n, id, rr)| (n.as_str(), *id, *rr)).collect();
            h.register_static_box("libnyash_string_plugin.so", &type_name, Some(type_id), None, None, &methods_ref, None);
        };
    }

    // If statically linking plugins, also register per-Box invoke pointers.
    // This allows calling v2 per-Box invoke directly without dynamic loader.
    #[cfg(feature = "static-array-plugin")]
    unsafe {
        extern "C" {
            fn nyash_array_plugin_invoke_static(
                instance_id: u32,
                method_id: u32,
                args: *const u8,
                args_len: usize,
                result: *mut u8,
                result_len: *mut usize,
            ) -> i32;
        }
        if let Some((provider, type_name, type_id, methods)) = parse_box_spec(arr) {
            let host = crate::runtime::get_global_plugin_host();
            if let Ok(h) = host.read() {
                let mref: Vec<(&str, u32, bool)> = methods.iter().map(|(n, id, rr)| (n.as_str(), *id, *rr)).collect();
                h.register_static_with_spec(
                    "libnyash_array_plugin.so",
                    &type_name,
                    Some(type_id),
                    None,
                    None,
                    &mref,
                    Some(std::mem::transmute(nyash_array_plugin_invoke_static as extern "C" fn(u32,u32,*const u8,usize,*mut u8,*mut usize)->i32)),
                );
            }
        }
    }
    #[cfg(feature = "static-map-plugin")]
    unsafe {
        extern "C" {
            fn nyash_map_plugin_invoke_static(
                instance_id: u32,
                method_id: u32,
                args: *const u8,
                args_len: usize,
                result: *mut u8,
                result_len: *mut usize,
            ) -> i32;
        }
        if let Some((provider, type_name, type_id, methods)) = parse_box_spec(map) {
            let host = crate::runtime::get_global_plugin_host();
            if let Ok(h) = host.read() {
                let mref: Vec<(&str, u32, bool)> = methods.iter().map(|(n, id, rr)| (n.as_str(), *id, *rr)).collect();
                h.register_static_with_spec(
                    "libnyash_map_plugin.so",
                    &type_name,
                    Some(type_id),
                    None,
                    None,
                    &mref,
                    Some(std::mem::transmute(nyash_map_plugin_invoke_static as extern "C" fn(u32,u32,*const u8,usize,*mut u8,*mut usize)->i32)),
                );
            }
        }
    }
    #[cfg(feature = "static-string-plugin")]
    unsafe {
        extern "C" {
            fn nyash_string_plugin_invoke_static(
                instance_id: u32,
                method_id: u32,
                args: *const u8,
                args_len: usize,
                result: *mut u8,
                result_len: *mut usize,
            ) -> i32;
        }
        if let Some((provider, type_name, type_id, methods)) = parse_box_spec(strb) {
            let host = crate::runtime::get_global_plugin_host();
            if let Ok(h) = host.read() {
                let mref: Vec<(&str, u32, bool)> = methods.iter().map(|(n, id, rr)| (n.as_str(), *id, *rr)).collect();
                h.register_static_with_spec(
                    "libnyash_string_plugin.so",
                    &type_name,
                    Some(type_id),
                    None,
                    None,
                    &mref,
                    Some(std::mem::transmute(nyash_string_plugin_invoke_static as extern "C" fn(u32,u32,*const u8,usize,*mut u8,*mut usize)->i32)),
                );
            }
        }
    }
}
