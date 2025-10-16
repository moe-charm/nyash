// extern_rune_dev.rs — nyrt.rune.* (dev mock evaluator)
use std::collections::HashMap;

use crate::backend::vm_types::VMValue;

pub fn register(map: &mut HashMap<(String, String), super::HandlerFn>) {
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
}
