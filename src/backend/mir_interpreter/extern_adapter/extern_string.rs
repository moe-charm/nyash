use std::collections::HashMap;
use crate::backend::vm_types::{VMError, VMValue};

pub fn register(map: &mut HashMap<(String, String), super::HandlerFn>) {
    // nyrt.string.length(recv:String): i64 (byte length)
    map.insert(("nyrt.string".into(), "length".into()), |args: &[VMValue]| {
        if args.is_empty() { return Err(VMError::InvalidInstruction("nyrt.string.length requires receiver".into())); }
        match &args[0] {
            VMValue::String(s) => Ok(VMValue::Integer(hako_core_string::length_bytes(s))),
            VMValue::BoxRef(b) => Ok(VMValue::Integer(hako_core_string::length_bytes(&b.to_string_box().value))),
            other => {
                if std::env::var("NYASH_DEBUG_STRING_LEN").ok().as_deref() == Some("1") {
                    eprintln!("[debug:string.len] unexpected arg={:?}", other);
                }
                Err(VMError::TypeError("nyrt.string.length expects String".into()))
            }
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
        let end   = match &args[2] { VMValue::Integer(i) => *i, v => v.to_string().parse::<i64>().unwrap_or(start) };
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
}
