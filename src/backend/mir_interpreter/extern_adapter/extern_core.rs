// extern_core.rs — Core extern handlers (time/string/map)
use std::collections::HashMap;

use crate::backend::vm_types::{VMError, VMValue};

fn extern_array_size_handler(args: &[VMValue]) -> Result<VMValue, VMError> {
    if args.is_empty() {
        return Err(VMError::InvalidInstruction(
            "nyrt.array.size requires receiver".into(),
        ));
    }
    match &args[0] {
        VMValue::BoxRef(b) => {
            #[cfg(feature = "legacy-boxes")]
            if let Some(arr) = b.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                let n = arr.len();
                return Ok(VMValue::Integer(n as i64));
            }
            let hh = crate::runtime::host_handles::to_handle_arc(b.clone());
            let mut out_buf = vec![0u8; 64];
            let mut out_len: usize = out_buf.len();
            let rc = crate::runtime::host_api::nyrt_host_call_slot(
                hh,
                102,
                std::ptr::null(),
                0,
                out_buf.as_mut_ptr(),
                &mut out_len,
            );
            if rc == 0 && out_len >= 6 {
                if let Some((tag, _sz, payload)) =
                    crate::runtime::plugin_ffi_common::decode::tlv_first(&out_buf[..out_len])
                {
                    if let Some(v) = crate::runtime::host_api::vmvalue_from_tlv(tag, payload) {
                        return Ok(v);
                    }
                }
            }
            Ok(VMValue::Integer(0))
        }
        _ => Err(VMError::TypeError(
            "nyrt.array.size expects ArrayBox".into(),
        )),
    }
}

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

    // nyrt.array.size(recv:Array) -> i64
    map.insert(("nyrt.array".into(), "size".into()), extern_array_size_handler as super::HandlerFn);
    // Alias: nyrt.array.length → size
    map.insert(("nyrt.array".into(), "length".into()), extern_array_size_handler as super::HandlerFn);

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

    // env.local.get(key:String) -> String (ENV value or empty string)
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
