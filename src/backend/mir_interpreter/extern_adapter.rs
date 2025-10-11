use std::collections::HashMap;
use std::sync::OnceLock;

use crate::backend::vm_types::VMValue;
use crate::mir::externs::registry as extreg;

use crate::backend::vm_types::VMError;

type HandlerFn = fn(&[VMValue]) -> Result<VMValue, VMError>;

struct VmExternAdapterBox {
    handlers: HashMap<(String, String), HandlerFn>,
}

static ADAPTER: OnceLock<VmExternAdapterBox> = OnceLock::new();

fn build_adapter() -> VmExternAdapterBox {
    let mut map: HashMap<(String, String), HandlerFn> = HashMap::new();

    // nyrt.time.now_ms(): i64
    map.insert(("nyrt.time".into(), "now_ms".into()), |args: &[VMValue]| {
        let _ = args; // no args
        use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_millis(0));
        let millis = duration.as_millis();
        let clamped = if millis > i64::MAX as u128 {
            i64::MAX
        } else {
            millis as i64
        };
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
            VMValue::BoxRef(b) => {
                // Best-effort string view
                Ok(VMValue::Integer(hako_core_string::length_bytes(&b.to_string_box().value)))
            }
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

    // nyrt.string.charAt(recv:String, idx:i64) -> String (byte-based)
    map.insert(("nyrt.string".into(), "charAt".into()), |args: &[VMValue]| {
        if args.len() < 2 { return Err(VMError::InvalidInstruction("nyrt.string.charAt requires 2 args".into())); }
        let s = match &args[0] { VMValue::String(s) => s.clone(), v => v.to_string() };
        let idx = match &args[1] { VMValue::Integer(i) => *i, v => v.to_string().parse::<i64>().unwrap_or(0) };
        Ok(VMValue::String(hako_core_string::char_at_byte(&s, idx)))
    });

    // nyrt.string.replace(recv:String, from:String, to:String) -> String (replace all)
    map.insert(("nyrt.string".into(), "replace".into()), |args: &[VMValue]| {
        if args.len() < 3 { return Err(VMError::InvalidInstruction("nyrt.string.replace requires 3 args".into())); }
        let s = match &args[0] { VMValue::String(s) => s.clone(), v => v.to_string() };
        let from = match &args[1] { VMValue::String(s) => s.clone(), v => v.to_string() };
        let to   = match &args[2] { VMValue::String(s) => s.clone(), v => v.to_string() };
        Ok(VMValue::String(hako_core_string::replace_all(&s, &from, &to)))
    });

    // nyrt.array.size(recv): i64
    map.insert(("nyrt.array".into(), "size".into()), |args: &[VMValue]| {
        if args.is_empty() {
            return Err(VMError::InvalidInstruction(
                "nyrt.array.size requires receiver".into(),
            ));
        }
        match &args[0] {
            VMValue::BoxRef(b) => {
                // Builtin ArrayBox
                if let Some(arr) = b.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                    if std::env::var("HAKO_TRACE_ARRAY_LEN").ok().as_deref() == Some("1") {
                        eprintln!("[trace] nyrt.array.size builtin -> len={}", arr.len());
                    }
                    return Ok(VMValue::Integer(hako_core_array::length(arr.len()) ));
                }
                // Plugin ArrayBox (TypeBox v2)
                if let Some(pb) = b
                    .as_any()
                    .downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>()
                {
                    if pb.box_type == "ArrayBox" {
                        let host = crate::runtime::plugin_loader_unified::get_global_plugin_host();
                        if let Ok(ro) = host.read() {
                            if let Ok(ret) = ro.invoke_instance_method("ArrayBox", "length", pb.instance_id(), &[]) {
                                if let Some(vb) = ret {
                                    if let Some(ii) = vb
                                        .as_any()
                                        .downcast_ref::<crate::box_trait::IntegerBox>()
                                    {
                                        return Ok(VMValue::Integer(ii.value));
                                    }
                                }
                            }
                        }
                        return Ok(VMValue::Integer(0));
                    }
                }
                {
                    if std::env::var("HAKO_TRACE_ARRAY_LEN").ok().as_deref() == Some("1") {
                        eprintln!("[trace] nyrt.array.size recv type_name={}", b.type_name());
                    }
                    Err(VMError::TypeError(
                        "nyrt.array.size expects ArrayBox".into(),
                    ))
                }
            }
            _ => Err(VMError::TypeError(
                "nyrt.array.size expects ArrayBox".into(),
            )),
        }
    });

    // nyrt.map.size(recv): i64
    map.insert(("nyrt.map".into(), "size".into()), |args: &[VMValue]| {
        if args.is_empty() {
            return Err(VMError::InvalidInstruction(
                "nyrt.map.size requires receiver".into(),
            ));
        }
        match &args[0] {
            VMValue::BoxRef(b) => {
                // Builtin MapBox
                if let Some(map) = b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                    let n = map.get_data().read().unwrap().len();
                    return Ok(VMValue::Integer(hako_core_map::size(n)));
                }
                // Plugin MapBox (TypeBox v2)
                if let Some(pb) = b
                    .as_any()
                    .downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>()
                {
                    if pb.box_type == "MapBox" {
                        let host = crate::runtime::plugin_loader_unified::get_global_plugin_host();
                        if let Ok(ro) = host.read() {
                            if let Ok(ret) = ro.invoke_instance_method("MapBox", "size", pb.instance_id(), &[]) {
                                if let Some(vb) = ret {
                                    if let Some(ii) = vb
                                        .as_any()
                                        .downcast_ref::<crate::box_trait::IntegerBox>()
                                    {
                                        return Ok(VMValue::Integer(ii.value));
                                    }
                                }
                            }
                        }
                        return Ok(VMValue::Integer(0));
                    }
                }
                Err(VMError::TypeError(
                    "nyrt.map.size expects MapBox".into(),
                ))
            }
            _ => Err(VMError::TypeError(
                "nyrt.map.size expects MapBox".into(),
            )),
        }
    });

    // nyrt.map.keys(recv): ArrayBox (builtin) or plugin (Stage-2)/shim(keysS)
    map.insert(("nyrt.map".into(), "keys".into()), |args: &[VMValue]| {
        if args.is_empty() {
            return Err(VMError::InvalidInstruction(
                "nyrt.map.keys requires receiver".into(),
            ));
        }
        match &args[0] {
            VMValue::BoxRef(b) => {
                // Builtin MapBox
                if let Some(mapb) = b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                    let out = mapb.keys();
                    if std::env::var("HAKO_TRACE_ARRAY_LEN").ok().as_deref() == Some("1") {
                        if let Some(arr) = out.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                            eprintln!("[trace] nyrt.map.keys builtin -> len={}", arr.len());
                        }
                    }
                    return Ok(VMValue::from_nyash_box(out));
                }
                // Plugin MapBox
                if let Some(pb) = b
                    .as_any()
                    .downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>()
                {
                    if pb.box_type == "MapBox" {
                        let host = crate::runtime::plugin_loader_unified::get_global_plugin_host();
                        if let Ok(ro) = host.read() {
                            let stage2 = std::env::var("NYASH_PLUGIN_MAP_ARRAY_HANDLE").ok().as_deref() == Some("1");
                            if stage2 {
                                if let Ok(ret) = ro.invoke_instance_method("MapBox", "keys", pb.instance_id(), &[]) {
                                    if let Some(vb) = ret { return Ok(VMValue::from_nyash_box(vb)); }
                                }
                            } else {
                                if let Ok(ret) = ro.invoke_instance_method("MapBox", "keysS", pb.instance_id(), &[]) {
                                    if let Some(vb) = ret {
                                        let s = vb.to_string_box().value;
                                        let arr = crate::boxes::array::ArrayBox::new();
                                        for line in s.split('\n') { if !line.is_empty() { arr.push(Box::new(crate::box_trait::StringBox::new(line))); } }
                                        return Ok(VMValue::from_nyash_box(Box::new(arr)));
                                    }
                                }
                            }
                        }
                        return Ok(VMValue::from_nyash_box(Box::new(crate::boxes::array::ArrayBox::new())));
                    }
                }
                Err(VMError::TypeError(
                    "nyrt.map.keys expects MapBox".into(),
                ))
            }
            _ => Err(VMError::TypeError(
                "nyrt.map.keys expects MapBox".into(),
            )),
        }
    });

    // nyrt.map.values(recv): ArrayBox (builtin) or shim via valuesS
    map.insert(("nyrt.map".into(), "values".into()), |args: &[VMValue]| {
        if args.is_empty() {
            return Err(VMError::InvalidInstruction(
                "nyrt.map.values requires receiver".into(),
            ));
        }
        match &args[0] {
            VMValue::BoxRef(b) => {
                // Builtin MapBox
                if let Some(mapb) = b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                    let out = mapb.values();
                    if std::env::var("HAKO_TRACE_ARRAY_LEN").ok().as_deref() == Some("1") {
                        if let Some(arr) = out.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                            eprintln!("[trace] nyrt.map.values builtin -> len={}", arr.len());
                        }
                    }
                    return Ok(VMValue::from_nyash_box(out));
                }
                // Plugin MapBox
                if let Some(pb) = b
                    .as_any()
                    .downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>()
                {
                    if pb.box_type == "MapBox" {
                        let host = crate::runtime::plugin_loader_unified::get_global_plugin_host();
                        if let Ok(ro) = host.read() {
                            let stage2 = std::env::var("NYASH_PLUGIN_MAP_ARRAY_HANDLE").ok().as_deref() == Some("1");
                            if stage2 {
                                if let Ok(ret) = ro.invoke_instance_method("MapBox", "values", pb.instance_id(), &[]) {
                                    if let Some(vb) = ret { return Ok(VMValue::from_nyash_box(vb)); }
                                }
                            } else {
                                if let Ok(ret) = ro.invoke_instance_method(
                                    "MapBox",
                                    "valuesS",
                                    pb.instance_id(),
                                    &[],
                                ) {
                                    if let Some(vb) = ret {
                                        let s = vb.to_string_box().value;
                                        let arr = crate::boxes::array::ArrayBox::new();
                                        for line in s.split('\n') { if !line.is_empty() { arr.push(Box::new(crate::box_trait::StringBox::new(line))); } }
                                        return Ok(VMValue::from_nyash_box(Box::new(arr)));
                                    }
                                }
                            }
                        }
                        return Ok(VMValue::from_nyash_box(Box::new(crate::boxes::array::ArrayBox::new())));
                    }
                }
                Err(VMError::TypeError(
                    "nyrt.map.values expects MapBox".into(),
                ))
            }
            _ => Err(VMError::TypeError(
                "nyrt.map.values expects MapBox".into(),
            )),
        }
    });

    // nyrt.rune.eval(code: String) -> i64 (skeleton mock)
    map.insert(("nyrt.rune".into(), "eval".into()), |args: &[VMValue]| {
        // Disabled-by-default; return -1 when not enabled
        let enabled = crate::runtime::env_gate_box::bool_any(&["HAKO_RUNE_ENABLE"]);
        if !enabled {
            return Ok(VMValue::Integer(-1));
        }
        let provider = std::env::var("HAKO_RUNE_PROVIDER").unwrap_or_else(|_| "mock".to_string());
        // Expect first arg as code string (best-effort)
        let code = if let Some(VMValue::String(s)) = args.get(0) { s.clone() } else { String::new() };
        if provider == "mock" {
            // Very small evaluator: support "A+B" with non-negative integers, ignore spaces
            let c = code.replace(" ", "");
            if let Some(pos) = c.find('+') {
                let (a, b) = c.split_at(pos);
                let b = &b[1..];
                let pa = a.parse::<i64>().unwrap_or(0);
                let pb = b.parse::<i64>().unwrap_or(0);
                return Ok(VMValue::Integer(pa + pb));
            }
            // Fallback: try integer literal
            if let Ok(n) = c.parse::<i64>() { return Ok(VMValue::Integer(n)); }
            return Ok(VMValue::Integer(-2)); // unsupported form
        }
        // Unknown provider
        Ok(VMValue::Integer(-3))
    });

    // nyrt.ops.op_eq(a, b): bool - Equality operator
    // Delegates to op_handlers::op_eq_static() for logic reuse
    // Note: This static version only supports pointer equality for boxes,
    //       not user-defined equals(). For full support, use handlers/externals.rs
    map.insert(("nyrt.ops".into(), "op_eq".into()), |args: &[VMValue]| {
        if args.len() < 2 {
            return Err(VMError::InvalidInstruction(
                "nyrt.ops.op_eq requires 2 arguments".into(),
            ));
        }

        crate::backend::mir_interpreter::handlers::op_handlers::op_eq_static(&args[0], &args[1])
    });

    // --- File I/O (nyrt.file.*) ---
    // nyrt.file.read(path: String) -> String
    map.insert(("nyrt.file".into(), "read".into()), |args: &[VMValue]| {
        if args.is_empty() {
            return Err(VMError::InvalidInstruction(
                "nyrt.file.read requires path argument".into(),
            ));
        }
        let path = match &args[0] {
            VMValue::String(s) => s.clone(),
            v => v.to_string(),
        };

        match std::fs::read_to_string(&path) {
            Ok(content) => Ok(VMValue::String(content)),
            Err(e) => Err(VMError::IoError(format!(
                "Failed to read file '{}': {}",
                path, e
            ))),
        }
    });

    // nyrt.file.write(path: String, content: String) -> Void
    map.insert(("nyrt.file".into(), "write".into()), |args: &[VMValue]| {
        if args.len() < 2 {
            return Err(VMError::InvalidInstruction(
                "nyrt.file.write requires path and content arguments".into(),
            ));
        }
        let path = match &args[0] {
            VMValue::String(s) => s.clone(),
            v => v.to_string(),
        };
        let content = match &args[1] {
            VMValue::String(s) => s.clone(),
            v => v.to_string(),
        };

        match std::fs::write(&path, content) {
            Ok(_) => Ok(VMValue::Void),
            Err(e) => Err(VMError::IoError(format!(
                "Failed to write file '{}': {}",
                path, e
            ))),
        }
    });


    // --- nykernel.* (dev stub for wasm std Array) ---
    // Phase 1 Cleanup (2025-10-11):
    // - ✅ nykernel_enabled() removed (3 lines) - unused
    // - ✅ heap_state() removed (4 lines) - unused
    // - ✅ as_i64() removed (8 lines) - unused
    // - ✅ use std::sync::Mutex removed (1 line) - unused after heap_state removal

    fn value_as_future(value: &VMValue) -> Option<crate::boxes::future::FutureBox> {
        match value {
            VMValue::Future(f) => Some(f.clone()),
            VMValue::BoxRef(arc) => arc
                .as_any()
                .downcast_ref::<crate::boxes::future::FutureBox>()
                .map(|f| f.clone()),
            _ => None,
        }
    }

    map.insert(("env.future".into(), "new".into()), |args: &[VMValue]| {
        let fut = crate::boxes::future::FutureBox::new();
        if let Some(val) = args.get(0) {
            fut.set_result(val.to_nyash_box());
        }
        Ok(VMValue::Future(fut))
    });

    map.insert(("env.future".into(), "set".into()), |args: &[VMValue]| {
        if args.len() >= 2 {
            if let Some(fut) = value_as_future(&args[0]) {
                fut.set_result(args[1].to_nyash_box());
            }
        }
        Ok(VMValue::Void)
    });

    map.insert(("env.future".into(), "await".into()), |args: &[VMValue]| {
        if let Some(first) = args.get(0) {
            if let Some(fut) = value_as_future(first) {
                let boxed = fut.get();
                return Ok(VMValue::from_nyash_box(boxed));
            }
            return Ok(VMValue::from_nyash_box(first.to_nyash_box()));
        }
        Ok(VMValue::Void)
    });

    map.insert(("env.future".into(), "spawn_instance".into()), |args: &[VMValue]| {
        let fut = crate::boxes::future::FutureBox::new();
        if let Some(val) = args.get(2) {
            fut.set_result(val.to_nyash_box());
        } else {
            fut.set_result(Box::new(crate::box_trait::VoidBox::new()));
        }
        Ok(VMValue::Future(fut))
    });

    // nykernel.malloc(size: i64) -> i64 (byte address)
    map.insert(("nykernel".into(), "malloc".into()), |args: &[VMValue]| {
        let size = args.get(0).map(crate::runtime::nykernel_stub::vmvalue_to_i64).unwrap_or(0);
        crate::runtime::nykernel_stub::malloc_bytes(size)
    });
    // nykernel.load_i64(addr: i64) -> i64
    map.insert(("nykernel".into(), "load_i64".into()), |args: &[VMValue]| {
        let addr = args.get(0).map(crate::runtime::nykernel_stub::vmvalue_to_i64).unwrap_or(0);
        crate::runtime::nykernel_stub::load_i64(addr)
    });
    // nykernel.store_i64(addr: i64, value: i64) -> void
    map.insert(("nykernel".into(), "store_i64".into()), |args: &[VMValue]| {
        if args.len() < 2 { return Err(VMError::InvalidInstruction("need 2 args".into())); }
        let addr = crate::runtime::nykernel_stub::vmvalue_to_i64(&args[0]);
        let val  = crate::runtime::nykernel_stub::vmvalue_to_i64(&args[1]);
        crate::runtime::nykernel_stub::store_i64(addr, val)
    });

    VmExternAdapterBox { handlers: map }
}

fn adapter() -> &'static VmExternAdapterBox {
    ADAPTER.get_or_init(build_adapter)
}

/// Try dispatch extern via VM adapter. Returns Some(Result) if known; None if unknown.
pub fn try_call(iface: &str, method: &str, loaded_args: &[VMValue]) -> Option<Result<VMValue, VMError>> {
    let key = (iface.to_string(), method.to_string());
    if let Some(h) = adapter().handlers.get(&key) {
        return Some(h(loaded_args));
    }
    // Fallback: consult externs registry as authoritative spec (in case a handler is added later)
    if extreg::registry().get(iface, method).is_some() {
        // Known spec but no handler — treat as unsupported for now
        return Some(Err(VMError::InvalidInstruction(format!(
            "Extern {}.{} has spec but no handler",
            iface, method
        ))));
    }
    None
}
