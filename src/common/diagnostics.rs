/*!
 * diagnostics – shared error/warn message builders
 */

pub mod msg {
    pub fn unborn_op() -> &'static str { "operation on unborn instance (call birth() first)" }
    pub fn reentrant_birth() -> &'static str { "reentrant birth()" }
    pub fn circular_tail_resolution(pick_base: &str) -> String {
        format!("circular resolution (tail fallback): {}", pick_base)
    }
    pub fn no_method_arity(cls: &str, method: &str, want: usize, known: &[usize]) -> String {
        format!("No matching method: {}.{}({} args). Available arities: {:?}", cls, method, want, known)
    }
    pub fn unknown_slot(cls: &str, method: &str, slot: u16) -> String {
        format!("Unknown slot {} for {}.{}", slot, cls, method)
    }
}



pub fn legacy_call_json(from: &str, to: &str, arity: usize) -> String {
    serde_json::json!({
        "kind": "legacy_call",
        "from": from,
        "to": to,
        "arity": arity,
    }).to_string()
}


pub fn plugin_invoke_non_plugin_warn(method: &str) -> String {
    serde_json::json!({
        "kind": "contracts_warn",
        "what": "plugin_invoke_non_plugin",
        "method": method,
    }).to_string()
}


pub fn provider_lock_warn() -> String {
    serde_json::json!({
        "kind": "provider_lock",
        "level": "warn",
        "what": "new_before_lock",
        "hint": "Set NYASH_PROVIDER_LOCK_STRICT=1 to error"
    }).to_string()
}

/// Runner: user requested PyVM but bridge feature is disabled.
pub fn runner_pyvm_bridge_disabled_warn() -> String {
    serde_json::json!({
        "kind": "runner_warn",
        "what": "pyvm_bridge_disabled",
        "action": "continue_llvm",
    }).to_string()
}

/// Dev fallback: user instance BoxCall routed via VM instance-dispatch.
pub fn dev_fallback_instance_boxcall(class: &str, method: &str) -> String {
    serde_json::json!({
        "kind": "dev_fallback",
        "what": "instance_boxcall_vm_dispatch",
        "class": class,
        "method": method,
    }).to_string()
}

/// P2P FunctionBox handler requires legacy interpreter; skipped.
pub fn p2p_functionbox_legacy_required_warn() -> String {
    serde_json::json!({
        "kind": "p2p_warn",
        "what": "functionbox_requires_interpreter_legacy",
        "action": "skipped_execution",
    }).to_string()
}

/// Builder dev-verify: NewBox not followed by birth (per-case).
pub fn dev_verify_newbox_missing_birth(class: &str, value: &str, expect: &str) -> String {
    serde_json::json!({
        "kind": "dev_verify",
        "what": "newbox_missing_birth",
        "class": class,
        "value": value,
        "expect": expect,
    }).to_string()
}

/// Builder dev-verify: summary count for NewBox→birth invariant warnings.
pub fn dev_verify_birth_invariant_summary(count: usize) -> String {
    serde_json::json!({
        "kind": "dev_verify",
        "what": "newbox_birth_invariant_warnings",
        "count": count,
    }).to_string()
}

/// Provider verify in warn mode — emit a stable JSON line with missing methods summary.
pub fn provider_verify_warn(missing: &[String]) -> String {
    serde_json::json!({
        "kind": "provider_verify",
        "level": "warn",
        "missing": missing,
        "count": missing.len(),
    }).to_string()
}

/// Build a consistent ambiguous-resolution error message for module functions.
/// `displayed` should contain the (possibly re-ordered) subset to show; `total` is full candidate count.
pub mod msg_ambiguous {
    pub fn module_function(name: &str, arity: usize, displayed: &[String], total: usize) -> String {
        let shown = displayed.len().min(10);
        let mut msg = format!(
            "Ambiguous module function resolution for '{}', arity={} ({} candidates, showing {}):\n",
            name, arity, total, shown
        );
        for k in displayed.iter().take(shown) {
            msg.push_str("  - ");
            msg.push_str(k);
            msg.push('\n');
        }
        if total > shown { msg.push_str(&format!("  ... and {} more\n", total - shown)); }
        msg.push_str("Hint: qualify with Class.method/Arity, or set NYASH_MIR_CALL_MODULE_FN_STRICT=0 to fallback.");
        msg
    }
}

/// Modules/Using system errors (unified JSON diagnostics)
pub mod modules_error {
    pub fn missing_dep(module: &str, dep: &str, req: &str) -> String {
        serde_json::json!({
            "kind": "modules_error",
            "code": "missing_dep",
            "module": module,
            "dep": dep,
            "require": req,
        }).to_string()
    }
    pub fn cycle(path: &[String]) -> String {
        serde_json::json!({
            "kind": "modules_error",
            "code": "cycle",
            "path": path,
        }).to_string()
    }
    pub fn conflict(ns: &str, paths: &[String]) -> String {
        serde_json::json!({
            "kind": "modules_error",
            "code": "conflict",
            "ns": ns,
            "paths": paths,
        }).to_string()
    }
    pub fn unresolved(target: &str, candidates: &[String]) -> String {
        serde_json::json!({
            "kind": "modules_error",
            "code": "unresolved",
            "target": target,
            "candidates": candidates,
        }).to_string()
    }
    pub fn ambiguous(target: &str, candidates: &[String]) -> String {
        serde_json::json!({
            "kind": "modules_error",
            "code": "ambiguous",
            "target": target,
            "candidates": candidates,
        }).to_string()
    }
    pub fn private_access(path: &str, pattern: &str) -> String {
        serde_json::json!({
            "kind": "modules_error",
            "code": "private_access",
            "path": path,
            "pattern": pattern,
        }).to_string()
    }

}

#[cfg(test)]
mod tests {
    use super::msg;
    #[test]
    fn unborn_and_reentrant_strings() {
        assert!(msg::unborn_op().contains("unborn"));
        assert!(msg::reentrant_birth().contains("birth"));
        let cyc = msg::circular_tail_resolution("A.B/1");
        assert!(cyc.contains("circular"));
    }
    #[test]
    fn arity_message_contains_numbers() {
        let m = msg::no_method_arity("MapBox","get",1,&[0,2]);
        assert!(m.contains("MapBox"));
        assert!(m.contains("get"));
        assert!(m.contains("1"));
        assert!(m.contains("2"));
    }
    #[test]
    fn diagnostics_json_escape() {
        let j = crate::common::diagnostics::plugin_invoke_non_plugin_warn("a\"b");
        let v: serde_json::Value = match serde_json::from_str(&j) { Ok(v)=>v, Err(e)=> panic!("json string invalid: {}\nraw={}", e, j) };
        assert_eq!(v["method"], "a\"b");
    }
}

/// Using prelude/collect errors (JSON diagnostics)
pub mod using_error {
    pub fn duplicate_import(path: &str, filename: &str, line: usize, prev_alias: &str, prev_line: usize) -> String {
        serde_json::json!({
            "kind": "using_error",
            "code": "duplicate_import",
            "path": path,
            "file": filename,
            "line": line,
            "prev_alias": prev_alias,
            "prev_line": prev_line,
        }).to_string()
    }
    pub fn alias_rebound(alias: &str, filename: &str, line: usize, prev_path: &str, prev_line: usize) -> String {
        serde_json::json!({
            "kind": "using_error",
            "code": "alias_rebound",
            "alias": alias,
            "file": filename,
            "line": line,
            "prev_path": prev_path,
            "prev_line": prev_line,
        }).to_string()
    }
}
