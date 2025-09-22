#![allow(unused_unsafe)]
//! JIT externs bridging to NyRT host API (C symbols) via by-slot encoding.
//!
//! 目的: VM/JIT一致のため、JITからも host_api::nyrt_host_call_slot を使うPoC。

use crate::backend::vm::VMValue;

fn tlv_encode_values(args: &[VMValue]) -> Vec<u8> {
    use crate::runtime::plugin_ffi_common::encode as enc;
    let mut buf = crate::runtime::plugin_ffi_common::encode_tlv_header(args.len() as u16);
    for a in args {
        match a {
            VMValue::Integer(i) => enc::i64(&mut buf, *i),
            VMValue::Float(f) => enc::f64(&mut buf, *f),
            VMValue::Bool(b) => enc::bool(&mut buf, *b),
            VMValue::String(s) => enc::string(&mut buf, s),
            VMValue::BoxRef(arc) => {
                // Try to downcast common primitives for stable TLV
                if let Some(sb) = arc.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                    enc::string(&mut buf, &sb.value);
                } else if let Some(ib) = arc.as_any().downcast_ref::<crate::box_trait::IntegerBox>()
                {
                    enc::i64(&mut buf, ib.value);
                } else if let Some(bb) = arc.as_any().downcast_ref::<crate::box_trait::BoolBox>() {
                    enc::bool(&mut buf, bb.value);
                } else if let Some(fb) = arc
                    .as_any()
                    .downcast_ref::<crate::boxes::math_box::FloatBox>()
                {
                    enc::f64(&mut buf, fb.value);
                } else {
                    // Fallback: send HostHandle so host can operate on it if needed
                    let h = crate::runtime::host_handles::to_handle_arc(arc.clone());
                    enc::host_handle(&mut buf, h);
                }
            }
            VMValue::Future(fu) => {
                let bx: Box<dyn crate::box_trait::NyashBox> = Box::new(fu.clone());
                let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> = std::sync::Arc::from(bx);
                let h = crate::runtime::host_handles::to_handle_arc(arc);
                enc::host_handle(&mut buf, h);
            }
            VMValue::Void => enc::string(&mut buf, "void"),
        }
    }
    buf
}

fn call_slot(handle: u64, slot: u64, argv: &[VMValue]) -> VMValue {
    let tlv = tlv_encode_values(argv);
    let mut out = vec![0u8; 256];
    let mut out_len: usize = out.len();
    let code = unsafe {
        crate::runtime::host_api::nyrt_host_call_slot(
            handle,
            slot,
            tlv.as_ptr(),
            tlv.len(),
            out.as_mut_ptr(),
            &mut out_len,
        )
    };
    if code != 0 {
        return VMValue::Void;
    }
    if let Some((tag, _sz, payload)) =
        crate::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len])
    {
        match tag {
            6 | 7 => {
                let s = crate::runtime::plugin_ffi_common::decode::string(payload);
                let sb = crate::box_trait::StringBox::new(&s);
                let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> = std::sync::Arc::new(sb);
                VMValue::BoxRef(arc)
            }
            1 => crate::runtime::plugin_ffi_common::decode::bool(payload)
                .map(VMValue::Bool)
                .unwrap_or(VMValue::Void),
            2 => crate::runtime::plugin_ffi_common::decode::i32(payload)
                .map(|v| VMValue::Integer(v as i64))
                .unwrap_or(VMValue::Void),
            3 => crate::runtime::plugin_ffi_common::decode::u64(payload)
                .map(|v| VMValue::Integer(v as i64))
                .unwrap_or(VMValue::Void),
            5 => crate::runtime::plugin_ffi_common::decode::f64(payload)
                .map(VMValue::Float)
                .unwrap_or(VMValue::Void),
            9 => {
                if let Some(h) = crate::runtime::plugin_ffi_common::decode::u64(payload) {
                    if let Some(arc) = crate::runtime::host_handles::get(h) {
                        return VMValue::BoxRef(arc);
                    }
                }
                VMValue::Void
            }
            _ => VMValue::Void,
        }
    } else {
        VMValue::Void
    }
}

fn to_handle(recv: &VMValue) -> Option<u64> {
    match recv {
        VMValue::BoxRef(arc) => Some(crate::runtime::host_handles::to_handle_arc(arc.clone())),
        _ => None,
    }
}

// Public bridge helpers (symbol strings align with collections for PoC)
pub const SYM_HOST_ARRAY_GET: &str = "nyash.host.array.get"; // (ArrayBox, i64)
pub const SYM_HOST_ARRAY_SET: &str = "nyash.host.array.set"; // (ArrayBox, i64, val)
pub const SYM_HOST_ARRAY_LEN: &str = "nyash.host.array.len"; // (ArrayBox)
pub const SYM_HOST_MAP_GET: &str = "nyash.host.map.get"; // (MapBox, key)
pub const SYM_HOST_MAP_SET: &str = "nyash.host.map.set"; // (MapBox, key, val)
pub const SYM_HOST_MAP_SIZE: &str = "nyash.host.map.size"; // (MapBox)
pub const SYM_HOST_MAP_HAS: &str = "nyash.host.map.has"; // (MapBox, key)
pub const SYM_HOST_CONSOLE_LOG: &str = "nyash.host.console.log"; // (value)
pub const SYM_HOST_CONSOLE_WARN: &str = "nyash.host.console.warn"; // (value)
pub const SYM_HOST_CONSOLE_ERROR: &str = "nyash.host.console.error"; // (value)
pub const SYM_HOST_INSTANCE_GETFIELD: &str = "nyash.host.instance.getField"; // (InstanceBox, name)
pub const SYM_HOST_INSTANCE_SETFIELD: &str = "nyash.host.instance.setField"; // (InstanceBox, name, value)
                                                                             // Arity-stable variants for Cranelift imports (avoid signature conflicts)
pub const SYM_HOST_INSTANCE_GETFIELD2: &str = "nyash.host.instance.getField2"; // (InstanceBox, name)
pub const SYM_HOST_INSTANCE_SETFIELD3: &str = "nyash.host.instance.setField3"; // (InstanceBox, name, value)
pub const SYM_HOST_INSTANCE_FIELD3: &str = "nyash.host.instance.field3"; // (recv,name,val or sentinel)
pub const SYM_HOST_STRING_LEN: &str = "nyash.host.string.len"; // (StringBox)

pub fn array_get(args: &[VMValue]) -> VMValue {
    if let Some(h) = to_handle(args.get(0).unwrap_or(&VMValue::Void)) {
        call_slot(h, 100, &args[1..])
    } else {
        VMValue::Void
    }
}
pub fn array_set(args: &[VMValue]) -> VMValue {
    if let Some(h) = to_handle(args.get(0).unwrap_or(&VMValue::Void)) {
        call_slot(h, 101, &args[1..])
    } else {
        VMValue::Void
    }
}
pub fn array_len(args: &[VMValue]) -> VMValue {
    if let Some(h) = to_handle(args.get(0).unwrap_or(&VMValue::Void)) {
        call_slot(h, 102, &[])
    } else {
        VMValue::Integer(0)
    }
}
pub fn map_get(args: &[VMValue]) -> VMValue {
    if let Some(h) = to_handle(args.get(0).unwrap_or(&VMValue::Void)) {
        call_slot(h, 203, &args[1..])
    } else {
        VMValue::Void
    }
}
pub fn map_set(args: &[VMValue]) -> VMValue {
    if let Some(h) = to_handle(args.get(0).unwrap_or(&VMValue::Void)) {
        call_slot(h, 204, &args[1..])
    } else {
        VMValue::Void
    }
}
pub fn map_size(args: &[VMValue]) -> VMValue {
    if let Some(h) = to_handle(args.get(0).unwrap_or(&VMValue::Void)) {
        call_slot(h, 200, &[])
    } else {
        VMValue::Integer(0)
    }
}
pub fn map_has(args: &[VMValue]) -> VMValue {
    if let Some(h) = to_handle(args.get(0).unwrap_or(&VMValue::Void)) {
        call_slot(h, 202, &args[1..])
    } else {
        VMValue::Bool(false)
    }
}

pub fn console_log(args: &[VMValue]) -> VMValue {
    // JIT host-bridge簡易版: 最初の引数を文字列化してstdoutへ
    if let Some(a0) = args.get(0) {
        println!("{}", a0.to_string());
    }
    VMValue::Void
}

pub fn console_warn(args: &[VMValue]) -> VMValue {
    if let Some(a0) = args.get(0) {
        eprintln!("[warn] {}", a0.to_string());
    }
    VMValue::Void
}

pub fn console_error(args: &[VMValue]) -> VMValue {
    if let Some(a0) = args.get(0) {
        eprintln!("[error] {}", a0.to_string());
    }
    VMValue::Void
}

pub fn instance_getfield(args: &[VMValue]) -> VMValue {
    if let Some(h) = to_handle(args.get(0).unwrap_or(&VMValue::Void)) {
        call_slot(h, 1, &args[1..])
    } else {
        VMValue::Void
    }
}
pub fn instance_setfield(args: &[VMValue]) -> VMValue {
    if let Some(h) = to_handle(args.get(0).unwrap_or(&VMValue::Void)) {
        call_slot(h, 2, &args[1..])
    } else {
        VMValue::Void
    }
}

pub fn string_len(args: &[VMValue]) -> VMValue {
    if let Some(h) = to_handle(args.get(0).unwrap_or(&VMValue::Void)) {
        call_slot(h, 300, &[])
    } else {
        VMValue::Integer(0)
    }
}
