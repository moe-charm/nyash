mod console;
mod env;

use std::collections::HashMap;

use crate::backend::llvm::compiler::codegen::instructions::builder_cursor::BuilderCursor;
use crate::backend::llvm::context::CodegenContext;
use crate::mir::{function::MirFunction, BasicBlockId, ValueId};
use inkwell::values::BasicValueEnum as BVE;

/// Full ExternCall lowering dispatcher (console/debug/env.*)
pub(in super::super) fn lower_externcall<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    resolver: &mut super::Resolver<'ctx>,
    cur_bid: BasicBlockId,
    func: &MirFunction,
    vmap: &mut HashMap<ValueId, BVE<'ctx>>,
    dst: &Option<ValueId>,
    iface_name: &str,
    method_name: &str,
    args: &[ValueId],
    bb_map: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        inkwell::basic_block::BasicBlock<'ctx>,
    >,
    preds: &std::collections::HashMap<crate::mir::BasicBlockId, Vec<crate::mir::BasicBlockId>>,
    block_end_values: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        std::collections::HashMap<ValueId, BVE<'ctx>>,
    >,
) -> Result<(), String> {
    // console/debug
    if (iface_name == "env.console" && matches!(method_name, "log" | "warn" | "error"))
        || (iface_name == "env.debug" && method_name == "trace")
    {
        return console::lower_log_or_trace(
            codegen,
            cursor,
            resolver,
            cur_bid,
            vmap,
            dst,
            iface_name,
            method_name,
            args,
            bb_map,
            preds,
            block_end_values,
        );
    }
    if iface_name == "env.console" && method_name == "readLine" {
        return console::lower_readline(codegen, cursor, cur_bid, vmap, dst, args);
    }

    // env.*
    if iface_name == "env.future" && method_name == "spawn_instance" {
        return env::lower_future_spawn_instance(
            codegen,
            cursor,
            resolver,
            cur_bid,
            vmap,
            dst,
            args,
            bb_map,
            preds,
            block_end_values,
        );
    }
    if iface_name == "env.local" && method_name == "get" {
        return env::lower_local_get(
            codegen,
            cursor,
            resolver,
            cur_bid,
            func,
            vmap,
            dst,
            args,
            bb_map,
            preds,
            block_end_values,
        );
    }
    if iface_name == "env.box" && method_name == "new" {
        return env::lower_box_new(
            codegen,
            cursor,
            resolver,
            cur_bid,
            func,
            vmap,
            dst,
            args,
            bb_map,
            preds,
            block_end_values,
        );
    }

    Err(format!(
        "ExternCall lowering unsupported: {}.{} (add a NyRT shim for this interface method)",
        iface_name, method_name
    ))
}
