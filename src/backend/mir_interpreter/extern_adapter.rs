use std::collections::HashMap;
use std::sync::OnceLock;

use crate::backend::vm_types::VMValue;
use crate::mir::externs::registry as extreg;

use crate::backend::vm_types::VMError;

pub type HandlerFn = fn(&[VMValue]) -> Result<VMValue, VMError>;

struct VmExternAdapterBox {
    handlers: HashMap<(String, String), HandlerFn>,
}

static ADAPTER: OnceLock<VmExternAdapterBox> = OnceLock::new();

fn build_adapter() -> VmExternAdapterBox {
    let mut map: HashMap<(String, String), HandlerFn> = HashMap::new();

    // Register core externs (string/time/map ...)
    #[allow(unused)]
    {
        #[allow(unused_imports)]
        use crate as nyash_rust;
        #[path = "extern_adapter/extern_core.rs"]
        mod extern_core;
        extern_core::register(&mut map);
    }

    // string/time externs are registered via extern_adapter/extern_core.rs

    // Register future/async legacy externs (isolated for feature-gating/ownership)
    #[allow(unused)]
    {
        #[allow(unused_imports)]
        use crate as nyash_rust;
        #[allow(clippy::single_component_path_imports)]
        {
            // Delegate registration into a small submodule to reduce coupling
            #[path = "extern_adapter/extern_future_legacy.rs"]
            mod extern_future_legacy;
            extern_future_legacy::register(&mut map);
        }
    }

    // Final override pass removed: split modules own all handlers

    // array externs are registered via extern_adapter/extern_core.rs

    // map externs are registered via extern_adapter/extern_core.rs

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

    // env.future externs are registered via extern_adapter/extern_future_legacy.rs

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
