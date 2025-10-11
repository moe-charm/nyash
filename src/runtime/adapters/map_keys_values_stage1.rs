//! MapKeysValuesStage1AdapterBox
//!
//! Responsibility
//! - Provide Stage-1 fallback for MapBox.keys()/values() when Stage-2 (HostHandle arrays)
//!   is disabled.
//! - Encapsulates the temporary compatibility path that asks the plugin for
//!   string form (keysS/valuesS) and converts newline-separated strings into
//!   an ArrayBox<String>.
//!
//! Inputs/Outputs
//! - Input: Plugin MapBox instance_id, method name ("keys"|"values")
//! - Output: VMValue::BoxRef(ArrayBox<String>) when available; None if not handled

use crate::backend::vm_types::VMValue;

pub fn route_map_keys_values_stage1(
    box_type: &str,
    instance_id: u32,
    method: &str,
) -> Option<VMValue> {
    if box_type != "MapBox" { return None; }
    if method != "keys" && method != "values" { return None; }
    // Only when Stage-2 is disabled
    let stage2 = std::env::var("NYASH_PLUGIN_MAP_ARRAY_HANDLE").ok().as_deref() == Some("1");
    if stage2 { return None; }

    let host = crate::runtime::plugin_loader_unified::get_global_plugin_host();
    let alt = if method == "keys" { "keysS" } else { "valuesS" };
    let s_opt: Option<String> = {
        if let Ok(ro) = host.read() {
            if let Ok(ret) = ro.invoke_instance_method("MapBox", alt, instance_id, &[]) {
                if let Some(vb) = ret { Some(vb.to_string_box().value) } else { None }
            } else { None }
        } else { None }
    };
    if let Some(s) = s_opt {
        let mut arr = crate::boxes::array::ArrayBox::new();
        for line in s.split('\n') {
            if !line.is_empty() {
                arr.push(Box::new(crate::box_trait::StringBox::new(line)));
            }
        }
        return Some(VMValue::from_nyash_box(Box::new(arr)));
    }
    None
}
