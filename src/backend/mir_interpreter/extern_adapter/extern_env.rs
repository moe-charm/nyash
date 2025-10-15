use std::collections::HashMap;
use crate::backend::vm_types::{VMError, VMValue};

pub fn register(map: &mut HashMap<(String, String), super::HandlerFn>) {
    // env.local.get(key:String) -> String
    map.insert(("env.local".into(), "get".into()), |args: &[VMValue]| {
        if args.len() < 1 { return Err(VMError::InvalidInstruction("env.local.get requires 1 arg".into())); }
        let key = match &args[0] {
            VMValue::String(s) => s.clone(),
            VMValue::BoxRef(b) => b.to_string_box().value,
            v => v.to_string(),
        };
        let val = std::env::var(&key).unwrap_or_else(|_| "".to_string());
        Ok(VMValue::String(val))
    });

    // nyash.json.canonicalize_h(json:String) -> String
    map.insert(("nyash.json".into(), "canonicalize_h".into()), |args: &[VMValue]| {
        if args.len() < 1 { return Err(VMError::InvalidInstruction("nyash.json.canonicalize_h requires 1 arg".into())); }
        let src = match &args[0] {
            VMValue::String(s) => s.clone(),
            VMValue::BoxRef(b) => b.to_string_box().value,
            v => v.to_string(),
        };
        match serde_json::from_str::<serde_json::Value>(&src) {
            Ok(v) => {
                let s = crate::common::json_canonical::to_canonical_string(&v);
                Ok(VMValue::String(s))
            }
            Err(_) => Ok(VMValue::String(src)),
        }
    });
}

