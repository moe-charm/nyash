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
