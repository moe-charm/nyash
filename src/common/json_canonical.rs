use serde_json::{Map, Value};

/// Recursively canonicalize a JSON value by sorting object keys lexicographically
/// and preserving array order. Numbers/booleans/null/strings are left as-is.
pub fn canonicalize(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            let mut out = Map::new();
            for k in keys {
                if let Some(v) = map.get(&k) {
                    out.insert(k, canonicalize(v));
                }
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(canonicalize).collect()),
        Value::String(s) => Value::String(s.clone()),
        Value::Number(n) => Value::Number(n.clone()),
        Value::Bool(b) => Value::Bool(*b),
        Value::Null => Value::Null,
    }
}

/// Produce a compact, deterministic JSON string from any serde_json::Value.
pub fn to_canonical_string(value: &Value) -> String {
    let v = canonicalize(value);
    serde_json::to_string(&v).unwrap_or_else(|_| "null".to_string())
}

