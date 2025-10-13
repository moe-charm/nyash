// extern_core.rs — Core extern handlers (time/string/map)
use std::collections::HashMap;

use crate::backend::vm_types::{VMError, VMValue};

pub fn register(map: &mut HashMap<(String, String), super::HandlerFn>) {
    // nyrt.time.now_ms(): i64
    map.insert(("nyrt.time".into(), "now_ms".into()), |args: &[VMValue]| {
        let _ = args; // no args
        use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_millis(0));
        let millis = duration.as_millis();
        let clamped = if millis > i64::MAX as u128 { i64::MAX } else { millis as i64 };
        if crate::runtime::env_gate_box::bool_any(&["NYASH_VM_TRACE"]) {
            crate::runtime::diagnostics::trace_event(
                "vm_extern",
                &format!("\"iface\":\"nyrt.time\",\"method\":\"now_ms\",\"value\":{}", clamped),
            );
        }
        Ok(VMValue::Integer(clamped))
    });

    // nyrt.string.length(recv:String): i64 (byte length)
    map.insert(("nyrt.string".into(), "length".into()), |args: &[VMValue]| {
        if args.is_empty() { return Err(VMError::InvalidInstruction("nyrt.string.length requires receiver".into())); }
        match &args[0] {
            VMValue::String(s) => Ok(VMValue::Integer(hako_core_string::length_bytes(s))),
            VMValue::BoxRef(b) => Ok(VMValue::Integer(hako_core_string::length_bytes(&b.to_string_box().value))),
            _ => Err(VMError::TypeError("nyrt.string.length expects String".into())),
        }
    });

    // nyrt.string.indexOf(recv:String, needle:String, from:i64=0) -> i64
    map.insert(("nyrt.string".into(), "indexOf".into()), |args: &[VMValue]| {
        if args.len() < 2 { return Err(VMError::InvalidInstruction("nyrt.string.indexOf requires 2 or 3 args".into())); }
        let s = match &args[0] { VMValue::String(s) => s.clone(), v => v.to_string() };
        let needle = match &args[1] { VMValue::String(s) => s.clone(), v => v.to_string() };
        let from = if args.len() >= 3 { match &args[2] { VMValue::Integer(i) => *i, v => v.to_string().parse::<i64>().unwrap_or(0) } } else { 0 };
        Ok(VMValue::Integer(hako_core_string::index_of(&s, &needle, from)))
    });

    // nyrt.string.lastIndexOf(recv:String, needle:String, from:i64=len) -> i64
    map.insert(("nyrt.string".into(), "lastIndexOf".into()), |args: &[VMValue]| {
        if args.len() < 2 { return Err(VMError::InvalidInstruction("nyrt.string.lastIndexOf requires 2 or 3 args".into())); }
        let s = match &args[0] { VMValue::String(s) => s.clone(), v => v.to_string() };
        let needle = match &args[1] { VMValue::String(s) => s.clone(), v => v.to_string() };
        let default_from = hako_core_string::length_bytes(&s);
        let from = if args.len() >= 3 { match &args[2] { VMValue::Integer(i) => *i, v => v.to_string().parse::<i64>().unwrap_or(default_from) } } else { default_from };
        Ok(VMValue::Integer(hako_core_string::last_index_of(&s, &needle, from)))
    });

    // nyrt.string.substring(recv:String, start:i64, end:i64) -> String
    map.insert(("nyrt.string".into(), "substring".into()), |args: &[VMValue]| {
        if args.len() < 3 { return Err(VMError::InvalidInstruction("nyrt.string.substring requires 3 args".into())); }
        let s = match &args[0] { VMValue::String(s) => s.clone(), v => v.to_string() };
        let start = match &args[1] { VMValue::Integer(i) => *i, v => v.to_string().parse::<i64>().unwrap_or(0) };
        let end   = match &args[2] { VMValue::Integer(i) => *i, v => v.to_string().parse::<i64>().unwrap_or(hako_core_string::length_bytes(&s)) };
        Ok(VMValue::String(hako_core_string::substring_bytes(&s, start, end)))
    });

    // nyrt.string.charAt(recv:String, idx:i64) -> String
    map.insert(("nyrt.string".into(), "charAt".into()), |args: &[VMValue]| {
        if args.len() < 2 { return Err(VMError::InvalidInstruction("nyrt.string.charAt requires 2 args".into())); }
        let s = match &args[0] { VMValue::String(s) => s.clone(), v => v.to_string() };
        let idx = match &args[1] { VMValue::Integer(i) => *i, v => v.to_string().parse::<i64>().unwrap_or(0) };
        Ok(VMValue::String(hako_core_string::char_at_byte(&s, idx)))
    });

    // nyrt.string.replace(recv:String, from:String, to:String) -> String
    map.insert(("nyrt.string".into(), "replace".into()), |args: &[VMValue]| {
        if args.len() < 3 { return Err(VMError::InvalidInstruction("nyrt.string.replace requires 3 args".into())); }
        let s = match &args[0] { VMValue::String(s) => s.clone(), v => v.to_string() };
        let from = match &args[1] { VMValue::String(s) => s.clone(), v => v.to_string() };
        let to   = match &args[2] { VMValue::String(s) => s.clone(), v => v.to_string() };
        Ok(VMValue::String(hako_core_string::replace_all(&s, &from, &to)))
    });

    // nyrt.map.size/keys/values
    map.insert(("nyrt.map".into(), "size".into()), |args: &[VMValue]| {
        if args.is_empty() { return Err(VMError::InvalidInstruction("nyrt.map.size requires receiver".into())); }
        match &args[0] {
            VMValue::BoxRef(b) => {
                #[cfg(feature = "legacy-boxes")]
                if let Some(map) = b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                    let n = map.get_data().read().unwrap().len();
                    return Ok(VMValue::Integer(hako_core_map::size(n)));
                }
                // plugin path fallbacks intentionally minimal
                Ok(VMValue::Integer(0))
            }
            _ => Err(VMError::TypeError("nyrt.map.size expects MapBox".into())),
        }
    });
    map.insert(("nyrt.map".into(), "keys".into()), |args: &[VMValue]| {
        if args.is_empty() { return Err(VMError::InvalidInstruction("nyrt.map.keys requires receiver".into())); }
        match &args[0] {
            VMValue::BoxRef(b) => {
                #[cfg(feature = "legacy-boxes")]
                if let Some(mapb) = b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                    return Ok(VMValue::from_nyash_box(mapb.keys()));
                }
                Ok(VMValue::Void)
            }
            _ => Err(VMError::TypeError("nyrt.map.keys expects MapBox".into())),
        }
    });
    map.insert(("nyrt.map".into(), "values".into()), |args: &[VMValue]| {
        if args.is_empty() { return Err(VMError::InvalidInstruction("nyrt.map.values requires receiver".into())); }
        match &args[0] {
            VMValue::BoxRef(b) => {
                #[cfg(feature = "legacy-boxes")]
                if let Some(mapb) = b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                    return Ok(VMValue::from_nyash_box(mapb.values()));
                }
                Ok(VMValue::Void)
            }
            _ => Err(VMError::TypeError("nyrt.map.values expects MapBox".into())),
        }
    });
}

