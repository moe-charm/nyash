use std::collections::HashMap as StdHashMap;

/// Load box type-id mapping from `nyash_box.toml`.
pub fn load_box_type_ids() -> StdHashMap<String, i64> {
    let mut map = StdHashMap::new();
    if let Ok(cfg) = std::fs::read_to_string("nyash_box.toml") {
        if let Ok(doc) = toml::from_str::<toml::Value>(&cfg) {
            if let Some(table) = doc.as_table() {
                for (box_name, box_val) in table {
                    if let Some(id) = box_val.get("type_id").and_then(|v| v.as_integer()) {
                        map.insert(box_name.clone(), id as i64);
                    }
                }
            }
        }
    }
    map
}
