use std::collections::HashMap;

use inkwell::values::BasicValueEnum as BVE;
use inkwell::AddressSpace;

use super::builder_cursor::BuilderCursor;
use crate::backend::llvm::context::CodegenContext;
use crate::mir::{BasicBlockId, ValueId};

// NewBox lowering (subset consistent with existing code)
pub(in super::super) fn lower_newbox<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    resolver: &mut super::Resolver<'ctx>,
    cur_bid: BasicBlockId,
    vmap: &mut HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    dst: ValueId,
    box_type: &str,
    args: &[ValueId],
    box_type_ids: &HashMap<String, i64>,
    bb_map: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        inkwell::basic_block::BasicBlock<'ctx>,
    >,
    preds: &std::collections::HashMap<crate::mir::BasicBlockId, Vec<crate::mir::BasicBlockId>>,
    block_end_values: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        std::collections::HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    >,
) -> Result<(), String> {
    match (box_type, args.len()) {
        ("StringBox", 1) => {
            // Resolve as i8* string pointer (AOT string fast-path)
            let p = resolver.resolve_ptr(
                codegen,
                cursor,
                cur_bid,
                args[0],
                bb_map,
                preds,
                block_end_values,
                vmap,
            )?;
            vmap.insert(dst, p.into());
            Ok(())
        }
        (_, n) if n == 1 || n == 2 => {
            let type_id = *box_type_ids.get(box_type).unwrap_or(&0);
            let i64t = codegen.context.i64_type();
            let fnty = i64t.fn_type(&[i64t.into(), i64t.into(), i64t.into(), i64t.into()], false);
            let callee = codegen
                .module
                .get_function("nyash.box.birth_i64")
                .unwrap_or_else(|| {
                    codegen
                        .module
                        .add_function("nyash.box.birth_i64", fnty, None)
                });
            let argc = i64t.const_int(args.len() as u64, false);
            let mut a1 = i64t.const_zero();
            let mut a2 = i64t.const_zero();
            if args.len() >= 1 {
                a1 = resolver.resolve_i64(
                    codegen,
                    cursor,
                    cur_bid,
                    args[0],
                    bb_map,
                    preds,
                    block_end_values,
                    vmap,
                )?;
            }
            if args.len() >= 2 {
                a2 = resolver.resolve_i64(
                    codegen,
                    cursor,
                    cur_bid,
                    args[1],
                    bb_map,
                    preds,
                    block_end_values,
                    vmap,
                )?;
            }
            let tid = i64t.const_int(type_id as u64, true);
            let call = cursor
                .emit_instr(cur_bid, |b| {
                    b.build_call(
                        callee,
                        &[tid.into(), argc.into(), a1.into(), a2.into()],
                        "birth_i64",
                    )
                })
                .map_err(|e| e.to_string())?;
            let h = call
                .try_as_basic_value()
                .left()
                .ok_or("birth_i64 returned void".to_string())?
                .into_int_value();
            let pty = codegen.context.ptr_type(AddressSpace::from(0));
            let ptr = cursor
                .emit_instr(cur_bid, |b| b.build_int_to_ptr(h, pty, "handle_to_ptr"))
                .map_err(|e| e.to_string())?;
            vmap.insert(dst, ptr.into());
            Ok(())
        }
        _ => {
            // No-arg birth via central type registry (preferred),
            // fallback to env.box.new(name) when type_id is unavailable.
            if !args.is_empty() {
                return Err("NewBox with >2 args not yet supported in LLVM lowering".to_string());
            }
            let type_id = *box_type_ids.get(box_type).unwrap_or(&0);
            // Temporary gate: allow forcing MapBox to plugin path explicitly
            let force_plugin_map =
                std::env::var("NYASH_LLVM_FORCE_PLUGIN_MAP").ok().as_deref() == Some("1");
            let i64t = codegen.context.i64_type();
            // Core-first: avoid birth_h for built-ins we provide directly (MapBox/ArrayBox)
            let is_core_builtin = box_type == "MapBox" || box_type == "ArrayBox";
            if type_id != 0 && !(is_core_builtin && !force_plugin_map) {
                // declare i64 @nyash.box.birth_h(i64)
                let fn_ty = i64t.fn_type(&[i64t.into()], false);
                let callee = codegen
                    .module
                    .get_function("nyash.box.birth_h")
                    .unwrap_or_else(|| {
                        codegen
                            .module
                            .add_function("nyash.box.birth_h", fn_ty, None)
                    });
                let tid = i64t.const_int(type_id as u64, true);
                let call = cursor
                    .emit_instr(cur_bid, |b| b.build_call(callee, &[tid.into()], "birth"))
                    .map_err(|e| e.to_string())?;
                let h_i64 = call
                    .try_as_basic_value()
                    .left()
                    .ok_or("birth_h returned void".to_string())?
                    .into_int_value();
                let pty = codegen.context.ptr_type(AddressSpace::from(0));
                let ptr = cursor
                    .emit_instr(cur_bid, |b| b.build_int_to_ptr(h_i64, pty, "handle_to_ptr"))
                    .map_err(|e| e.to_string())?;
                vmap.insert(dst, ptr.into());
                Ok(())
            } else {
                // Fallback: call i64 @nyash.env.box.new(i8*) with type name
                let i8p = codegen.context.ptr_type(AddressSpace::from(0));
                let fn_ty = i64t.fn_type(&[i8p.into()], false);
                let callee = codegen
                    .module
                    .get_function("nyash.env.box.new")
                    .unwrap_or_else(|| {
                        codegen
                            .module
                            .add_function("nyash.env.box.new", fn_ty, None)
                    });
                let tn = cursor
                    .emit_instr(cur_bid, |b| {
                        b.build_global_string_ptr(box_type, "box_type_name")
                    })
                    .map_err(|e| e.to_string())?;
                let call = cursor
                    .emit_instr(cur_bid, |b| {
                        b.build_call(callee, &[tn.as_pointer_value().into()], "env_box_new")
                    })
                    .map_err(|e| e.to_string())?;
                let h_i64 = call
                    .try_as_basic_value()
                    .left()
                    .ok_or("env.box.new returned void".to_string())?
                    .into_int_value();
                let pty = codegen.context.ptr_type(AddressSpace::from(0));
                let ptr = cursor
                    .emit_instr(cur_bid, |b| b.build_int_to_ptr(h_i64, pty, "handle_to_ptr"))
                    .map_err(|e| e.to_string())?;
                vmap.insert(dst, ptr.into());
                Ok(())
            }
        }
    }
}
