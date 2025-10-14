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
    // Boxed: nyrt.rune.*
    #[allow(unused)]
    {
        #[path = "extern_adapter/extern_rune_dev.rs"]
        mod extern_rune_dev;
        extern_rune_dev::register(&mut map);
    }

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

    // Boxed: nyrt.file.*
    #[allow(unused)]
    {
        #[path = "extern_adapter/extern_file_dev.rs"]
        mod extern_file_dev;
        extern_file_dev::register(&mut map);
    }


    // --- nykernel.* (dev stub for wasm std Array) ---
    // Phase 1 Cleanup (2025-10-11):
    // - ✅ nykernel_enabled() removed (3 lines) - unused
    // - ✅ heap_state() removed (4 lines) - unused
    // - ✅ as_i64() removed (8 lines) - unused
    // - ✅ use std::sync::Mutex removed (1 line) - unused after heap_state removal

    // env.future externs are registered via extern_adapter/extern_future_legacy.rs

    // Boxed: nykernel.* dev stub
    #[allow(unused)]
    {
        #[path = "extern_adapter/extern_nykernel_stub.rs"]
        mod extern_nykernel_stub;
        extern_nykernel_stub::register(&mut map);
    }

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
