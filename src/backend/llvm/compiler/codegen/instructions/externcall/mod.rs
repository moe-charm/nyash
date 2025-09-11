mod console;
mod env;

use std::collections::HashMap;

use crate::backend::llvm::context::CodegenContext;
use crate::mir::{function::MirFunction, ValueId};
use inkwell::values::BasicValueEnum as BVE;

/// Full ExternCall lowering dispatcher (console/debug/env.*)
pub(in super::super) fn lower_externcall<'ctx>(
    codegen: &CodegenContext<'ctx>,
    func: &MirFunction,
    vmap: &mut HashMap<ValueId, BVE<'ctx>>,
    dst: &Option<ValueId>,
    iface_name: &str,
    method_name: &str,
    args: &[ValueId],
) -> Result<(), String> {
    // console/debug
    if (iface_name == "env.console"
        && matches!(method_name, "log" | "warn" | "error"))
        || (iface_name == "env.debug" && method_name == "trace")
    {
        return console::lower_log_or_trace(codegen, vmap, dst, iface_name, method_name, args);
    }
    if iface_name == "env.console" && method_name == "readLine" {
        return console::lower_readline(codegen, vmap, dst, args);
    }

    // env.*
    if iface_name == "env.future" && method_name == "spawn_instance" {
        return env::lower_future_spawn_instance(codegen, vmap, dst, args);
    }
    if iface_name == "env.local" && method_name == "get" {
        return env::lower_local_get(codegen, func, vmap, dst, args);
    }
    if iface_name == "env.box" && method_name == "new" {
        return env::lower_box_new(codegen, vmap, dst, args);
    }

    Err(format!(
        "ExternCall lowering unsupported: {}.{} (add a NyRT shim for this interface method)",
        iface_name, method_name
    ))
}

