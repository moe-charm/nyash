use std::collections::HashMap;

use inkwell::{types::BasicMetadataTypeEnum as BMT, values::{BasicMetadataValueEnum, BasicValueEnum as BVE, FunctionValue}};

use crate::backend::llvm::context::CodegenContext;
use crate::mir::{function::MirFunction, BasicBlockId, ValueId};
use crate::backend::llvm::compiler::codegen::instructions::builder_cursor::BuilderCursor;

/// Lower a direct Call where callee is provided as a const string ValueId in MIR14.
///
/// Requirements:
/// - `const_strs`: mapping from ValueId to the string literal value within the same function.
/// - `llvm_funcs`: predeclared LLVM functions keyed by MIR function name (same string as const).
pub(in super::super) fn lower_call<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    cur_bid: BasicBlockId,
    _func: &MirFunction,
    vmap: &mut HashMap<ValueId, BVE<'ctx>>,
    dst: &Option<ValueId>,
    callee: &ValueId,
    args: &[ValueId],
    const_strs: &HashMap<ValueId, String>,
    llvm_funcs: &HashMap<String, FunctionValue<'ctx>>,
) -> Result<(), String> {
    let name_s = const_strs
        .get(callee)
        .ok_or_else(|| format!("call: callee value {} not a const string", callee.as_u32()))?;
    let target = llvm_funcs
        .get(name_s)
        .ok_or_else(|| format!("call: function not predeclared: {}", name_s))?;

    // Collect and coerce args to the callee's expected parameter types
    let fn_ty = target.get_type();
    let exp_tys: Vec<BMT<'ctx>> = fn_ty.get_param_types();
    if exp_tys.len() != args.len() {
        return Err(format!(
            "call: arg count mismatch for {} (expected {}, got {})",
            name_s,
            exp_tys.len(),
            args.len()
        ));
    }
    let mut params: Vec<BasicMetadataValueEnum> = Vec::with_capacity(args.len());
    for (i, a) in args.iter().enumerate() {
        let v = *vmap
            .get(a)
            .ok_or_else(|| format!("call arg missing: {}", a.as_u32()))?;
        let tv = coerce_to_type_cursor(codegen, cursor, cur_bid, v, exp_tys[i])?;
        params.push(tv.into());
    }
    let call = cursor
        .emit_instr(cur_bid, |b| b.build_call(*target, &params, "call"))
        .map_err(|e| e.to_string())?;
    if let Some(d) = dst {
        if let Some(rv) = call.try_as_basic_value().left() {
            vmap.insert(*d, rv);
        }
    }
    Ok(())
}

fn coerce_to_type_cursor<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    cur_bid: BasicBlockId,
    val: BVE<'ctx>,
    target: BMT<'ctx>,
) -> Result<BVE<'ctx>, String> {
    use inkwell::types::BasicMetadataTypeEnum as BMTy;
    match (val, target) {
        (BVE::IntValue(iv), BMTy::IntType(it)) => {
            let bw_src = iv.get_type().get_bit_width();
            let bw_dst = it.get_bit_width();
            if bw_src == bw_dst {
                Ok(iv.into())
            } else if bw_src < bw_dst {
                Ok(cursor
                    .emit_instr(cur_bid, |b| b.build_int_z_extend(iv, it, "call_zext"))
                    .map_err(|e| e.to_string())?
                    .into())
            } else if bw_dst == 1 {
                Ok(super::super::types::to_bool(codegen.context, iv.into(), &codegen.builder)?.into())
            } else {
                Ok(cursor
                    .emit_instr(cur_bid, |b| b.build_int_truncate(iv, it, "call_trunc"))
                    .map_err(|e| e.to_string())?
                    .into())
            }
        }
        (BVE::PointerValue(pv), BMTy::IntType(it)) => Ok(cursor
            .emit_instr(cur_bid, |b| b.build_ptr_to_int(pv, it, "call_p2i"))
            .map_err(|e| e.to_string())?
            .into()),
        (BVE::FloatValue(fv), BMTy::IntType(it)) => Ok(cursor
            .emit_instr(cur_bid, |b| b.build_float_to_signed_int(fv, it, "call_f2i"))
            .map_err(|e| e.to_string())?
            .into()),
        (BVE::IntValue(iv), BMTy::PointerType(pt)) => Ok(cursor
            .emit_instr(cur_bid, |b| b.build_int_to_ptr(iv, pt, "call_i2p"))
            .map_err(|e| e.to_string())?
            .into()),
        (BVE::PointerValue(pv), BMTy::PointerType(_)) => Ok(pv.into()),
        (BVE::IntValue(iv), BMTy::FloatType(ft)) => Ok(cursor
            .emit_instr(cur_bid, |b| b.build_signed_int_to_float(iv, ft, "call_i2f"))
            .map_err(|e| e.to_string())?
            .into()),
        (BVE::FloatValue(fv), BMTy::FloatType(_)) => Ok(fv.into()),
        (v, _) => Ok(v),
    }
}
