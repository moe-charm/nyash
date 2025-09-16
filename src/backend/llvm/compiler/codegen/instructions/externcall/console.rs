use std::collections::HashMap;

use inkwell::values::BasicValueEnum as BVE;
use inkwell::AddressSpace;

use crate::backend::llvm::compiler::codegen::instructions::builder_cursor::BuilderCursor;
use crate::backend::llvm::context::CodegenContext;
use crate::mir::{BasicBlockId, ValueId};

pub(super) fn lower_log_or_trace<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    resolver: &mut super::super::Resolver<'ctx>,
    cur_bid: BasicBlockId,
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
    if args.len() != 1 {
        return Err(format!("{}.{} expects 1 arg", iface_name, method_name));
    }
    // Localize to i64 (handle path) to avoid vmap shape inspection
    let arg_val = resolver.resolve_i64(
        codegen,
        cursor,
        cur_bid,
        args[0],
        bb_map,
        preds,
        block_end_values,
        vmap,
    )?;
    let i64t = codegen.context.i64_type();
    let fnty = i64t.fn_type(&[i64t.into()], false);
    let fname = if iface_name == "env.console" {
        match method_name {
            "log" => "nyash.console.log_handle",
            "warn" => "nyash.console.warn_handle",
            _ => "nyash.console.error_handle",
        }
    } else {
        "nyash.debug.trace_handle"
    };
    let callee = codegen
        .module
        .get_function(fname)
        .unwrap_or_else(|| codegen.module.add_function(fname, fnty, None));
    let _ = cursor
        .emit_instr(cur_bid, |b| {
            b.build_call(callee, &[arg_val.into()], "console_log_h")
        })
        .map_err(|e| e.to_string())?;
    if let Some(d) = dst {
        vmap.insert(*d, codegen.context.i64_type().const_zero().into());
    }
    Ok(())
}

pub(super) fn lower_readline<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    cur_bid: BasicBlockId,
    vmap: &mut HashMap<ValueId, BVE<'ctx>>,
    dst: &Option<ValueId>,
    args: &[ValueId],
) -> Result<(), String> {
    if !args.is_empty() {
        return Err("console.readLine expects 0 args".to_string());
    }
    let i8p = codegen.context.ptr_type(AddressSpace::from(0));
    let fnty = i8p.fn_type(&[], false);
    let callee = codegen
        .module
        .get_function("nyash.console.readline")
        .unwrap_or_else(|| {
            codegen
                .module
                .add_function("nyash.console.readline", fnty, None)
        });
    let call = cursor
        .emit_instr(cur_bid, |b| b.build_call(callee, &[], "readline"))
        .map_err(|e| e.to_string())?;
    if let Some(d) = dst {
        let rv = call
            .try_as_basic_value()
            .left()
            .ok_or("readline returned void".to_string())?;
        vmap.insert(*d, rv);
    }
    Ok(())
}
