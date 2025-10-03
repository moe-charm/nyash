use serde_json::Value;

/// Minimal MIR JSON validator for harness v0/v1
/// Goal: fail-fast on missing required fields (e.g., unop.src) to catch
/// schema regressions before handing off to the Python harness.
pub fn validate_json_root(root: &Value) -> Result<(), String> {
    // Detect schema form
    // v1: { schema_version, functions: [...] }
    // v0: { functions: [...] }
    let funcs = match root.get("functions") {
        Some(v) => v,
        None => return Err("MIR JSON missing 'functions' array".to_string()),
    };
    let arr = funcs.as_array().ok_or("'functions' must be an array".to_string())?;
    for f in arr {
        let name = f.get("name").and_then(|v| v.as_str()).unwrap_or("<unknown>");
        let blocks = f.get("blocks").and_then(|v| v.as_array()).ok_or_else(|| format!("function '{}' missing 'blocks' array", name))?;
        for b in blocks {
            let bid = b.get("id").and_then(|v| v.as_i64()).unwrap_or(-1);
            let insts = b.get("instructions").and_then(|v| v.as_array()).ok_or_else(|| format!("function '{}' block {} missing 'instructions' array", name, bid))?;
            for (idx, inst) in insts.iter().enumerate() {
                let op = inst.get("op").and_then(|v| v.as_str()).ok_or_else(|| format!("function '{}' block {} inst#{} missing 'op'", name, bid, idx))?;
                match op {
                    "unop" => {
                        ensure_field(inst, "kind", name, bid, idx)?;
                        ensure_non_null_u32(inst, "src", name, bid, idx)?;
                        ensure_non_null_u32(inst, "dst", name, bid, idx)?;
                    }
                    "binop" => {
                        ensure_field(inst, "operation", name, bid, idx)?;
                        ensure_non_null_u32(inst, "lhs", name, bid, idx)?;
                        ensure_non_null_u32(inst, "rhs", name, bid, idx)?;
                        ensure_non_null_u32(inst, "dst", name, bid, idx)?;
                    }
                    "compare" => {
                        ensure_field(inst, "operation", name, bid, idx)?;
                        ensure_non_null_u32(inst, "lhs", name, bid, idx)?;
                        ensure_non_null_u32(inst, "rhs", name, bid, idx)?;
                        ensure_non_null_u32(inst, "dst", name, bid, idx)?;
                    }
                    "externcall" => {
                        // dst may be absent; name or callee must exist in v1
                        if inst.get("name").is_none() && inst.get("callee").is_none() {
                            return Err(format!("function '{}' block {} inst#{} externcall missing 'name'/'callee'", name, bid, idx));
                        }
                        ensure_array(inst, "args", name, bid, idx)?;
                    }
                    "typeop" => {
                        ensure_field(inst, "operation", name, bid, idx)?;
                        ensure_non_null_u32(inst, "src", name, bid, idx)?;
                        ensure_non_null_u32(inst, "dst", name, bid, idx)?;
                        ensure_field(inst, "target_type", name, bid, idx)?;
                    }
                    "newbox" => {
                        ensure_field(inst, "type", name, bid, idx)?;
                        ensure_array(inst, "args", name, bid, idx)?;
                        ensure_non_null_u32(inst, "dst", name, bid, idx)?;
                    }
                    "boxcall" => {
                        ensure_non_null_u32(inst, "box", name, bid, idx)?;
                        ensure_field(inst, "method", name, bid, idx)?;
                        ensure_array(inst, "args", name, bid, idx)?;
                    }
                    "call" => {
                        ensure_non_null_u32(inst, "func", name, bid, idx)?;
                        ensure_array(inst, "args", name, bid, idx)?;
                        ensure_optional_u32(inst, "dst", name, bid, idx)?;
                    }
                    "branch" => {
                        ensure_non_null_u32(inst, "cond", name, bid, idx)?;
                        ensure_non_null_u32(inst, "then", name, bid, idx)?;
                        ensure_non_null_u32(inst, "else", name, bid, idx)?;
                    }
                    "jump" => {
                        ensure_non_null_u32(inst, "target", name, bid, idx)?;
                    }
                    "ret" => {
                        ensure_optional_u32(inst, "value", name, bid, idx)?;
                    }
                    "copy" => {
                        ensure_non_null_u32(inst, "dst", name, bid, idx)?;
                        ensure_non_null_u32(inst, "src", name, bid, idx)?;
                    }
                    // Future ops (accept with minimal checks; expanded later)
                    "safepoint" => { /* no required fields for MVP */ }
                    "load" => {
                        // tolerate minimal forms: dst/addr present when emitted
                        let _ = inst.get("dst");
                        let _ = inst.get("addr");
                    }
                    "store" => {
                        // tolerate minimal forms: addr/value present when emitted
                        let _ = inst.get("addr");
                        let _ = inst.get("value");
                    }
                    _ => { /* tolerate others for now */ }
                }
            }
        }
    }
    Ok(())
}

fn ensure_field(inst: &Value, key: &str, fname: &str, bid: i64, idx: usize) -> Result<(), String> {
    if inst.get(key).is_none() { Err(format!("function '{}' block {} inst#{} missing '{}'", fname, bid, idx, key)) } else { Ok(()) }
}

fn ensure_non_null_u32(inst: &Value, key: &str, fname: &str, bid: i64, idx: usize) -> Result<(), String> {
    match inst.get(key) {
        Some(v) if v.is_u64() => Ok(()),
        Some(v) if v.is_number() => Ok(()),
        _ => Err(format!("function '{}' block {} inst#{} '{}' must be number (u32)", fname, bid, idx, key)),
    }
}

fn ensure_array(inst: &Value, key: &str, fname: &str, bid: i64, idx: usize) -> Result<(), String> {
    match inst.get(key) {
        Some(Value::Array(_)) => Ok(()),
        Some(v) if v.is_null() => Ok(()),
        Some(_) => Err(format!("function '{}' block {} inst#{} '{}' must be array", fname, bid, idx, key)),
        None => Err(format!("function '{}' block {} inst#{} missing '{}'", fname, bid, idx, key)),
    }
}

fn ensure_optional_u32(inst: &Value, key: &str, fname: &str, bid: i64, idx: usize) -> Result<(), String> {
    match inst.get(key) {
        None => Ok(()),
        Some(v) if v.is_null() => Ok(()),
        Some(v) if v.is_u64() => Ok(()),
        Some(v) if v.is_number() => Ok(()),
        _ => Err(format!(
            "function '{}' block {} inst#{} '{}' must be number (u32) or null",
            fname, bid, idx, key
        )),
    }
}
