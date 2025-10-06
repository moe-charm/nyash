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
        if std::env::var("NYASH_VM_TRACE").ok().as_deref() == Some("1") {
            eprintln!("[vm.extern] nyrt.time.now_ms -> {}", clamped);
        }
        Ok(VMValue::Integer(clamped))
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
                if let Some(arr) = b
                    .as_any()
                    .downcast_ref::<crate::boxes::array::ArrayBox>()
                {
                    Ok(VMValue::Integer(arr.len() as i64))
                } else {
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
                if let Some(map) = b
                    .as_any()
                    .downcast_ref::<crate::boxes::map_box::MapBox>()
                {
                    Ok(VMValue::Integer(map.get_data().read().unwrap().len() as i64))
                } else {
                    Err(VMError::TypeError(
                        "nyrt.map.size expects MapBox".into(),
                    ))
                }
            }
            _ => Err(VMError::TypeError(
                "nyrt.map.size expects MapBox".into(),
            )),
        }
    });

    // nyrt.rune.eval(code: String) -> i64 (skeleton mock)
    map.insert(("nyrt.rune".into(), "eval".into()), |args: &[VMValue]| {
        // Disabled-by-default; return -1 when not enabled
        let enabled = std::env::var("HAKO_RUNE_ENABLE").ok().as_deref() == Some("1");
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

    

    // --- nykernel.* (dev stub for wasm std Array) ---
    // Enabled only when NYASH_ENABLE_NYKERNEL_STUB=1
    fn nykernel_enabled() -> bool {
        std::env::var("NYASH_ENABLE_NYKERNEL_STUB").ok().as_deref() == Some("1")
    }
    use std::sync::Mutex;
    fn heap_state() -> &'static (Mutex<Vec<i64>>, Mutex<i64>) {
        static HEAP: OnceLock<(Mutex<Vec<i64>>, Mutex<i64>)> = OnceLock::new();
        HEAP.get_or_init(|| (Mutex::new(Vec::with_capacity(1024)), Mutex::new(1)))
    }
    fn as_i64(v: &VMValue) -> i64 {
        match v {
            VMValue::Integer(i) => *i,
            VMValue::Float(f) => *f as i64,
            VMValue::String(s) => s.parse::<i64>().unwrap_or(0),
            VMValue::Bool(b) => if *b {1} else {0},
            _ => 0,
        }
    }
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
    // Ensure spec exists in registry (keeps behavior predictable)
    if extreg::registry().get(iface, method).is_none() {
        return None;
    }
    let key = (iface.to_string(), method.to_string());
    if let Some(h) = adapter().handlers.get(&key) {
        Some(h(loaded_args))
    } else {
        None
    }
}
