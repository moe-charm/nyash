//! Legacy box call handling
//!
//! This module provides backward compatibility and runtime fallback logic
//! for BoxCall instructions.
//!
//! ## Module Structure
//! - `plugin_bridge`: Plugin box invocation + fallbacks
//! - `mod` (this file): Main BoxCall dispatcher

use super::super::super::*;
use crate::backend::mir_interpreter::handlers::{
    boxes_fields, boxes_instance,
};

mod plugin_bridge;


impl MirInterpreter {
    #[inline]
    fn env_truthy_default(key: &str, default_on: bool) -> bool {
        match std::env::var(key).ok().as_deref() {
            Some("1" | "true" | "on" | "TRUE" | "ON") => true,
            Some("0" | "false" | "off" | "FALSE" | "OFF") => false,
            _ => default_on,
        }
    }
    pub(crate) fn handle_box_call(
        &mut self,
        dst: Option<ValueId>,
        box_val: ValueId,
        method: &str,
        args: &[ValueId],
    ) -> Result<(), VMError> {

        // Unborn guard for plugin instance methods (except birth)
        if method != "birth" { self.check_unborn_guard(box_val)?; }

        // Early lifecycle: if this is birth(), mark as born before any dispatch.
        // This allows birth() implementation to call back into instance methods
        // without tripping unborn guards in the same step.
        if method == "birth" {
            self.lifecycle_contracts_birth(box_val, args.len());
        }
        // Fail-Fast: forbid operations on unborn instances (user InstanceBox) until birth()
        if method != "birth" { self.check_unborn_guard(box_val)?; }
        // Dev-only call trace for BoxCall (parity aid)
        let label = format!("BoxCall:{}", method);
        self.emit_call_trace_label(&label, args.len(), None);

        // ArrayBox.slice early intercept removed. Parity handled downstream.

        // Handle-trace: birth/fini observation (centralized)
        self.lifecycle_observe_method(box_val, method);

        // PluginInvoke retired: routing to a separate PluginInvoke path is removed.
        // BoxCall proceeds via builtin handlers, user InstanceBox dispatch,
        // and finally the plugin bridge when receiver is a plugin-backed box.
        // Dev-safe: stringify(Void) → "null" (最小安全弁)
        if method == "stringify" {
            if let VMValue::Void = self.reg_load(box_val)? {
                if let Some(d) = dst { self.regs.insert(d, VMValue::String("null".to_string())); }
                return Ok(());
            }
            if let VMValue::BoxRef(b) = self.reg_load(box_val)? {
                if b.as_any().downcast_ref::<crate::box_trait::VoidBox>().is_some() {
                    if let Some(d) = dst { self.regs.insert(d, VMValue::String("null".to_string())); }
                    return Ok(());
                }
            }
        }
        // Birth no-op for builtin (non-plugin, non-user-instance) boxes: if receiver is a BoxRef that is
        // neither PluginBoxV2 nor user InstanceBox, and no specific handler claimed it yet, treat birth() as
        // successful no-op. Early lifecycle marking at entry already recorded born.
        if method == "birth" {
            if let VMValue::BoxRef(bx) = self.reg_load(box_val)? {
                let is_plugin = bx.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>().is_some();
                let is_instance = bx.as_any().downcast_ref::<crate::instance_v2::InstanceBox>().is_some();
                if !is_plugin && !is_instance {
                    if let Some(d) = dst { self.regs.insert(d, VMValue::Void); }
                    return Ok(());
                }
            }
        }
        // Trace: method call (class inferred from receiver)
        if Self::box_trace_enabled() {
            let cls = match self.reg_load(box_val).unwrap_or(VMValue::Void) {
                VMValue::BoxRef(b) => {
                    if let Some(inst) = b.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                        inst.class_name.clone()
                    } else {
                        b.type_name().to_string()
                    }
                }
                VMValue::String(_) => "StringBox".to_string(),
                VMValue::Integer(_) => "IntegerBox".to_string(),
                VMValue::Float(_) => "FloatBox".to_string(),
                VMValue::Bool(_) => "BoolBox".to_string(),
                VMValue::Void => "<Void>".to_string(),
                VMValue::Future(_) => "<Future>".to_string(),
            };
            self.box_trace_emit_call(&cls, method, args.len());
        }
        // Debug: trace length dispatch receiver type before any handler resolution
        if method == "length" && super::super::VmConfig::global().general_trace {
            let recv = self.reg_load(box_val).unwrap_or(VMValue::Void);
            let type_name = match recv.clone() {
                VMValue::BoxRef(b) => b.type_name().to_string(),
                VMValue::Integer(_) => "Integer".to_string(),
                VMValue::Float(_) => "Float".to_string(),
                VMValue::Bool(_) => "Bool".to_string(),
                VMValue::String(_) => "String".to_string(),
                VMValue::Void => "Void".to_string(),
                VMValue::Future(_) => "Future".to_string(),
            };
            eprintln!("[vm-trace] length dispatch recv_type={}", type_name);
        }
        // Graceful void guard for common short-circuit patterns in user code
        if let Some(res) = self.boxcall_void_guard_defaults(dst, &self.reg_load(box_val)?, method) { return res; }
        if boxes_fields::try_handle_object_fields(self, dst, box_val, method, args)? {
            if method == "length" && super::super::VmConfig::global().general_trace {
                eprintln!("[vm-trace] length dispatch handler=object_fields");
            }
            return Ok(());
        }
        // Policy gate: user InstanceBox BoxCall runtime fallback
        // - Prod: disallowed (builder must have rewritten obj.m(...) to a
        //   function call). Error here indicates a builder/using materialize
        //   miss.
        // - Dev/CI: allowed with WARN to aid diagnosis.
        let mut user_instance_class: Option<String> = None;
        if let VMValue::BoxRef(ref b) = self.reg_load(box_val)? {
            if let Some(inst) = b.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                user_instance_class = Some(inst.class_name.clone());
            }
        }
        if user_instance_class.is_some() && method != "birth" && !crate::config::env::vm_allow_user_instance_boxcall() {
            let cls = user_instance_class.unwrap();
            return Err(VMError::InvalidInstruction(format!(
                "User Instance BoxCall disallowed in prod: {}.{} (enable builder rewrite)",
                cls, method
            )));
        }
        if user_instance_class.is_some() && method != "birth" && crate::config::env::vm_allow_user_instance_boxcall() {
            if crate::config::env::cli_verbose() && !crate::config::env::cli_quiet() {
                let cls = user_instance_class.as_ref().unwrap();
                // Human-readable warn (legacy)
                eprintln!(
                    "[warn] dev fallback: user instance BoxCall {}.{} routed via VM instance-dispatch",
                    cls,
                    method
                );
                // Optional JSON warn for tooling when enabled
                if std::env::var("NYASH_WARN_JSON").ok().as_deref() == Some("1") {
                    eprintln!("{}", crate::common::diagnostics::dev_fallback_instance_boxcall(cls, method));
                }
            }
        }
        if boxes_instance::try_handle_instance_box(self, dst, box_val, method, args)? {
            if method == "length" && super::super::VmConfig::global().general_trace {
                eprintln!("[vm-trace] length dispatch handler=instance_box");
            }
            return Ok(());
        }
        // String VM convenience handlers removed (Phase 15.7). Prefer plugin/User paths.
        // Array VM convenience handlers removed (Phase 15.7). Prefer plugin/User paths.
        // Map VM convenience handlers removed (Phase 15.7). Prefer plugin/User paths.
        // Birth no-op for user InstanceBox when no class-defined birth() is available.
        // Builder injects birth() after NewBox; absence of a user birth implementation
        // should not be fatal. Birth is already recorded at entry; returning Void here
        // mirrors plugin no-op behavior.
        if method == "birth" {
            if let VMValue::BoxRef(b) = self.reg_load(box_val)? {
                if b.as_any().downcast_ref::<crate::instance_v2::InstanceBox>().is_some() {
                    if let Some(d) = dst { self.regs.insert(d, VMValue::Void); }
                    return Ok(());
                }
            }
        }
        // Narrow safety valve: if 'length' wasn't handled by any box-specific path,
        // treat it as 0 (avoids Lt on Void in common loops). This is a dev-time
        // robustness measure; precise behavior should be provided by concrete boxes.
        // Gate by env: set NYASH_VM_LENGTH_FALLBACK=0 to disable and fail-fast upstream.
        if method == "length" && Self::env_truthy_default("NYASH_VM_LENGTH_FALLBACK", true) {
            let recv_any = self.reg_load(box_val)?;
            let is_core = match &recv_any {
                VMValue::String(_) => true,
                VMValue::BoxRef(bx) => crate::runtime::type_registry::is_core_box(bx.type_name()),
                _ => false,
            };
            if !is_core {
                if super::super::VmConfig::global().general_trace {
                    eprintln!("[vm-trace] length dispatch handler=fallback(length=0)");
                }
                if let Some(d) = dst { self.regs.insert(d, VMValue::Integer(0)); }
                return Ok(());
            }
        }
        // Fallback: unique-tail dynamic resolution for user-defined methods
        // Narrowing: restrict to receiver's class when available to avoid
        // accidentally binding methods from unrelated boxes that happen to
        // share the same method name/arity (e.g., JsonScanner.is_eof vs JsonToken.is_eof).
        if let Some(func) = {
            let tail = format!(".{}{}", method, format!("/{}", args.len()));
            // Determine receiver class name when possible
            let recv_cls: Option<String> = match self.reg_load(box_val).ok() {
                Some(VMValue::BoxRef(b)) => {
                    if let Some(inst) = b.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                        Some(inst.class_name.clone())
                    } else { None }
                }
                _ => None,
            };
            // Safety: only attempt tail-based fallback when we know the receiver class.
            // This prevents accidental cross-module resolution (e.g., JsonCursorBox → JsonFragBox)
            // which can create recursion cycles.
            if let Some(ref want) = recv_cls {
                let mut cands: Vec<String> = self
                    .functions
                    .keys()
                    .filter(|k| k.ends_with(&tail))
                    .cloned()
                    .collect();
                let prefix = format!("{}.", want);
                cands.retain(|k| k.starts_with(&prefix));
                if cands.len() == 1 { self.functions.get(&cands[0]).cloned() } else { None }
            } else {
                None
            }
        } {
            // Build argv: pass receiver as first arg (me)
            let recv_vm = self.reg_load(box_val)?;
            let mut argv: Vec<VMValue> = Vec::with_capacity(1 + args.len());
            argv.push(recv_vm);
            for a in args { argv.push(self.reg_load(*a)?); }
            let ret = self.exec_function_inner(&func, Some(&argv))?;
            if let Some(d) = dst { self.regs.insert(d, ret); }
            return Ok(());
        }

        // Birth contracts are recorded at entry for birth(); avoid double-recording here.
        // Route via Router for plugin boxes; builtin/instance → builtin executor
        {
            let recv_any = self.reg_load(box_val)?;
            if let VMValue::BoxRef(ref bx) = recv_any {
                if bx.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>().is_some() {
                    let mut argv_vals: Vec<VMValue> = Vec::with_capacity(args.len());
                    for a in args { argv_vals.push(self.reg_load(*a)?); }
                    let result = crate::runtime::method_router_box::route(self, &recv_any, method, &argv_vals)?;
                    if let Some(d) = dst { self.regs.insert(d, result.clone()); }
                    self.maybe_register_scope_value(&result);
                    return Ok(());
                }
            }
            let result = self.execute_method_call(&recv_any, method, args)?;
            if let Some(d) = dst { self.regs.insert(d, result.clone()); }
            self.maybe_register_scope_value(&result);
            return Ok(());
        }
    }
}
