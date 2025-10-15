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

pub fn register_from_toml(toml_str: &str) {
    let Some((provider, type_name, type_id, methods)) = parse_box_spec(toml_str) else { return; };
    if let Ok(h) = crate::runtime::get_global_plugin_host().read() {
        let methods_ref: Vec<(&str, u32, bool)> = methods
            .iter()
            .map(|(n, id, rr)| (n.as_str(), *id, *rr))
            .collect();
        h.register_static_box(&provider, &type_name, Some(type_id), None, None, &methods_ref, None);
    }
}

// Build-script generated registration entry
include!(concat!(env!("OUT_DIR"), "/static_plugins_generated.rs"));
