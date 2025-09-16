use std::collections::HashMap;

use super::super::ctx::{BlockCtx, LowerFnCtx};
use crate::backend::llvm::context::CodegenContext;
use crate::mir::ValueId;
use inkwell::{values::BasicValueEnum as BVE, AddressSpace};

/// Handle getField/setField; returns true if handled.
use super::super::builder_cursor::BuilderCursor;

pub(super) fn try_handle_field_method<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut super::super::builder_cursor::BuilderCursor<'ctx, 'b>,
    cur_bid: crate::mir::BasicBlockId,
    vmap: &mut HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    dst: &Option<ValueId>,
    method: &str,
    args: &[ValueId],
    recv_h: inkwell::values::IntValue<'ctx>,
    resolver: &mut super::super::Resolver<'ctx>,
    bb_map: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        inkwell::basic_block::BasicBlock<'ctx>,
    >,
    preds: &std::collections::HashMap<crate::mir::BasicBlockId, Vec<crate::mir::BasicBlockId>>,
    block_end_values: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        std::collections::HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    >,
) -> Result<bool, String> {
    let i64t = codegen.context.i64_type();
    match method {
        "getField" => {
            if args.len() != 1 {
                return Err("getField expects 1 arg (name)".to_string());
            }
            let name_p = resolver.resolve_ptr(
                codegen,
                cursor,
                cur_bid,
                args[0],
                bb_map,
                preds,
                block_end_values,
                vmap,
            )?;
            let i8p = codegen.context.ptr_type(AddressSpace::from(0));
            let fnty = i64t.fn_type(&[i64t.into(), i8p.into()], false);
            let callee = codegen
                .module
                .get_function("nyash.instance.get_field_h")
                .unwrap_or_else(|| {
                    codegen
                        .module
                        .add_function("nyash.instance.get_field_h", fnty, None)
                });
            let call = cursor
                .emit_instr(cur_bid, |b| {
                    b.build_call(callee, &[recv_h.into(), name_p.into()], "getField")
                })
                .map_err(|e| e.to_string())?;
            if let Some(d) = dst {
                let rv = call
                    .try_as_basic_value()
                    .left()
                    .ok_or("get_field returned void".to_string())?;
                let h = if let BVE::IntValue(iv) = rv {
                    iv
                } else {
                    return Err("get_field ret expected i64".to_string());
                };
                let pty = codegen.context.ptr_type(AddressSpace::from(0));
                let ptr = codegen
                    .builder
                    .build_int_to_ptr(h, pty, "gf_handle_to_ptr")
                    .map_err(|e| e.to_string())?;
                vmap.insert(*d, ptr.into());
            }
            Ok(true)
        }
        "setField" => {
            if args.len() != 2 {
                return Err("setField expects 2 args (name, value)".to_string());
            }
            let name_p = resolver.resolve_ptr(
                codegen,
                cursor,
                cur_bid,
                args[0],
                bb_map,
                preds,
                block_end_values,
                vmap,
            )?;
            let val_h = resolver.resolve_i64(
                codegen,
                cursor,
                cur_bid,
                args[1],
                bb_map,
                preds,
                block_end_values,
                vmap,
            )?;
            let i8p = codegen.context.ptr_type(AddressSpace::from(0));
            let fnty = i64t.fn_type(&[i64t.into(), i8p.into(), i64t.into()], false);
            let callee = codegen
                .module
                .get_function("nyash.instance.set_field_h")
                .unwrap_or_else(|| {
                    codegen
                        .module
                        .add_function("nyash.instance.set_field_h", fnty, None)
                });
            let _ = cursor
                .emit_instr(cur_bid, |b| {
                    b.build_call(
                        callee,
                        &[recv_h.into(), name_p.into(), val_h.into()],
                        "setField",
                    )
                })
                .map_err(|e| e.to_string())?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

// Boxed wrapper that delegates to the non-boxed implementation
pub(super) fn try_handle_field_method_boxed<'ctx, 'b>(
    ctx: &mut LowerFnCtx<'ctx, 'b>,
    blk: &BlockCtx<'ctx>,
    dst: &Option<ValueId>,
    method: &str,
    args: &[ValueId],
    recv_h: inkwell::values::IntValue<'ctx>,
) -> Result<bool, String> {
    try_handle_field_method(
        ctx.codegen,
        ctx.cursor,
        blk.cur_bid,
        ctx.vmap,
        dst,
        method,
        args,
        recv_h,
        ctx.resolver,
        ctx.bb_map,
        ctx.preds,
        ctx.block_end_values,
    )
}
