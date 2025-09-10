use std::collections::HashMap;

use super::super::MirType;

/// Load plugin method signatures from `nyash_box.toml`.
///
/// Returns mapping `(BoxName, MethodName) -> MirType`.
pub fn load_plugin_method_sigs() -> HashMap<(String, String), MirType> {
    let mut sigs = HashMap::new();
    if let Ok(content) = std::fs::read_to_string("nyash_box.toml") {
        if let Ok(root) = toml::from_str::<toml::Value>(&content) {
            if let Some(table) = root.as_table() {
                for (box_name, box_val) in table {
                    if let Some(methods) = box_val.get("methods").and_then(|v| v.as_table()) {
                        for (mname, mval) in methods {
                            if let Some(ret) = mval.get("returns") {
                                let ty_str = ret.as_str().map(|s| s.to_string()).or_else(|| {
                                    ret.get("type")
                                        .and_then(|t| t.as_str())
                                        .map(|s| s.to_string())
                                });
                                if let Some(ts) = ty_str {
                                    let mir_ty = match ts.to_lowercase().as_str() {
                                        "i64" | "int" | "integer" => MirType::Integer,
                                        "f64" | "float" => MirType::Float,
                                        "bool" | "boolean" => MirType::Bool,
                                        "string" => MirType::String,
                                        "void" | "unit" => MirType::Void,
                                        other => MirType::Box(other.to_string()),
                                    };
                                    sigs.insert((box_name.clone(), mname.clone()), mir_ty);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    sigs
}
