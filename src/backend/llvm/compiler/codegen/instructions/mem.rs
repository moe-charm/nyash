use std::collections::HashMap;

use inkwell::values::BasicValueEnum;

use super::builder_cursor::BuilderCursor;
use crate::backend::llvm::context::CodegenContext;
use crate::mir::{BasicBlockId, ValueId};

// Lower Store: handle allocas with element type tracking and integer width adjust
pub(in super::super) fn lower_store<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    resolver: &mut super::Resolver<'ctx>,
    cur_bid: BasicBlockId,
    vmap: &HashMap<ValueId, BasicValueEnum<'ctx>>,
    allocas: &mut HashMap<ValueId, inkwell::values::PointerValue<'ctx>>,
    alloca_elem_types: &mut HashMap<ValueId, inkwell::types::BasicTypeEnum<'ctx>>,
    value: &ValueId,
    ptr: &ValueId,
    bb_map: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        inkwell::basic_block::BasicBlock<'ctx>,
    >,
    preds: &std::collections::HashMap<crate::mir::BasicBlockId, Vec<crate::mir::BasicBlockId>>,
    block_end_values: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        std::collections::HashMap<ValueId, BasicValueEnum<'ctx>>,
    >,
) -> Result<(), String> {
    use inkwell::types::BasicTypeEnum;
    // Resolve value preferring native kind; try i64, then f64, else pointer
    let i64t = codegen.context.i64_type();
    let val: BasicValueEnum = if let Ok(iv) = resolver.resolve_i64(
        codegen,
        cursor,
        cur_bid,
        *value,
        bb_map,
        preds,
        block_end_values,
        vmap,
    ) {
        iv.into()
    } else if let Ok(fv) = resolver.resolve_f64(
        codegen,
        cursor,
        cur_bid,
        *value,
        bb_map,
        preds,
        block_end_values,
        vmap,
    ) {
        fv.into()
    } else if let Ok(pv) = resolver.resolve_ptr(
        codegen,
        cursor,
        cur_bid,
        *value,
        bb_map,
        preds,
        block_end_values,
        vmap,
    ) {
        pv.into()
    } else {
        // Fallback: zero i64
        i64t.const_zero().into()
    };
    let elem_ty = match val {
        BasicValueEnum::IntValue(iv) => BasicTypeEnum::IntType(iv.get_type()),
        BasicValueEnum::FloatValue(fv) => BasicTypeEnum::FloatType(fv.get_type()),
        BasicValueEnum::PointerValue(pv) => BasicTypeEnum::PointerType(pv.get_type()),
        _ => return Err("unsupported store value type".to_string()),
    };
    if let Some(existing) = allocas.get(ptr).copied() {
        let existing_elem = *alloca_elem_types
            .get(ptr)
            .ok_or("alloca elem type missing")?;
        if existing_elem != elem_ty {
            match (val, existing_elem) {
                (BasicValueEnum::IntValue(iv), BasicTypeEnum::IntType(t)) => {
                    let bw_src = iv.get_type().get_bit_width();
                    let bw_dst = t.get_bit_width();
                    if bw_src < bw_dst {
                        let adj = cursor
                            .emit_instr(cur_bid, |b| b.build_int_z_extend(iv, t, "zext"))
                            .map_err(|e| e.to_string())?;
                        cursor
                            .emit_instr(cur_bid, |b| b.build_store(existing, adj))
                            .map_err(|e| e.to_string())?;
                    } else if bw_src > bw_dst {
                        let adj = cursor
                            .emit_instr(cur_bid, |b| b.build_int_truncate(iv, t, "trunc"))
                            .map_err(|e| e.to_string())?;
                        cursor
                            .emit_instr(cur_bid, |b| b.build_store(existing, adj))
                            .map_err(|e| e.to_string())?;
                    } else {
                        cursor
                            .emit_instr(cur_bid, |b| b.build_store(existing, iv))
                            .map_err(|e| e.to_string())?;
                    }
                }
                (BasicValueEnum::PointerValue(pv), BasicTypeEnum::PointerType(pt)) => {
                    let adj = cursor
                        .emit_instr(cur_bid, |b| b.build_pointer_cast(pv, pt, "pcast"))
                        .map_err(|e| e.to_string())?;
                    cursor
                        .emit_instr(cur_bid, |b| b.build_store(existing, adj))
                        .map_err(|e| e.to_string())?;
                }
                (BasicValueEnum::FloatValue(fv), BasicTypeEnum::FloatType(ft)) => {
                    // Only f64 currently expected
                    if fv.get_type() != ft {
                        return Err("float width mismatch in store".to_string());
                    }
                    cursor
                        .emit_instr(cur_bid, |b| b.build_store(existing, fv))
                        .map_err(|e| e.to_string())?;
                }
                _ => return Err("store type mismatch".to_string()),
            }
        } else {
            cursor
                .emit_instr(cur_bid, |b| b.build_store(existing, val))
                .map_err(|e| e.to_string())?;
        }
    } else {
        let slot = cursor
            .emit_instr(cur_bid, |b| {
                b.build_alloca(elem_ty, &format!("slot_{}", ptr.as_u32()))
            })
            .map_err(|e| e.to_string())?;
        cursor
            .emit_instr(cur_bid, |b| b.build_store(slot, val))
            .map_err(|e| e.to_string())?;
        allocas.insert(*ptr, slot);
        alloca_elem_types.insert(*ptr, elem_ty);
    }
    Ok(())
}

pub(in super::super) fn lower_load<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    cur_bid: BasicBlockId,
    vmap: &mut HashMap<ValueId, BasicValueEnum<'ctx>>,
    allocas: &mut HashMap<ValueId, inkwell::values::PointerValue<'ctx>>,
    alloca_elem_types: &mut HashMap<ValueId, inkwell::types::BasicTypeEnum<'ctx>>,
    dst: &ValueId,
    ptr: &ValueId,
) -> Result<(), String> {
    use inkwell::types::BasicTypeEnum;
    let (slot, elem_ty) = if let Some(s) = allocas.get(ptr).copied() {
        let et = *alloca_elem_types
            .get(ptr)
            .ok_or("alloca elem type missing")?;
        (s, et)
    } else {
        // Default new slot as i64 for uninitialized loads
        let i64t = codegen.context.i64_type();
        let slot = cursor
            .emit_instr(cur_bid, |b| {
                b.build_alloca(i64t, &format!("slot_{}", ptr.as_u32()))
            })
            .map_err(|e| e.to_string())?;
        allocas.insert(*ptr, slot);
        alloca_elem_types.insert(*ptr, i64t.into());
        (slot, i64t.into())
    };
    let lv = cursor
        .emit_instr(cur_bid, |b| {
            b.build_load(elem_ty, slot, &format!("load_{}", dst.as_u32()))
        })
        .map_err(|e| e.to_string())?;
    vmap.insert(*dst, lv);
    Ok(())
}

// Lower Copy: define dst in the current block by localizing src via Resolver
pub(in super::super) fn lower_copy<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    resolver: &mut super::Resolver<'ctx>,
    cur_bid: BasicBlockId,
    func: &crate::mir::function::MirFunction,
    vmap: &mut HashMap<ValueId, BasicValueEnum<'ctx>>,
    dst: &ValueId,
    src: &ValueId,
    bb_map: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        inkwell::basic_block::BasicBlock<'ctx>,
    >,
    preds: &std::collections::HashMap<crate::mir::BasicBlockId, Vec<crate::mir::BasicBlockId>>,
    block_end_values: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        std::collections::HashMap<ValueId, BasicValueEnum<'ctx>>,
    >,
) -> Result<(), String> {
    // Choose resolution kind based on metadata type preference
    use inkwell::types::BasicTypeEnum as BT;
    let expected_bt: Option<BT> = func
        .metadata
        .value_types
        .get(dst)
        .or_else(|| func.metadata.value_types.get(src))
        .map(|mt| super::super::types::map_mirtype_to_basic(codegen.context, mt));
    let out: BasicValueEnum<'ctx> = match expected_bt {
        Some(BT::IntType(_)) | None => {
            // Prefer i64 for unknown
            let iv = resolver.resolve_i64(
                codegen,
                cursor,
                cur_bid,
                *src,
                bb_map,
                preds,
                block_end_values,
                vmap,
            )?;
            iv.into()
        }
        Some(BT::PointerType(_)) => {
            let pv = resolver.resolve_ptr(
                codegen,
                cursor,
                cur_bid,
                *src,
                bb_map,
                preds,
                block_end_values,
                vmap,
            )?;
            pv.into()
        }
        Some(BT::FloatType(_)) => {
            let fv = resolver.resolve_f64(
                codegen,
                cursor,
                cur_bid,
                *src,
                bb_map,
                preds,
                block_end_values,
                vmap,
            )?;
            fv.into()
        }
        _ => {
            // Fallback i64
            let iv = resolver.resolve_i64(
                codegen,
                cursor,
                cur_bid,
                *src,
                bb_map,
                preds,
                block_end_values,
                vmap,
            )?;
            iv.into()
        }
    };
    vmap.insert(*dst, out);
    Ok(())
}
