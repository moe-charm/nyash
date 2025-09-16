use std::collections::HashMap;

use inkwell::{
    types::BasicMetadataTypeEnum as BMT,
    values::{BasicMetadataValueEnum, BasicValueEnum as BVE, FunctionValue},
};

use crate::backend::llvm::compiler::codegen::instructions::builder_cursor::BuilderCursor;
use crate::backend::llvm::context::CodegenContext;
use crate::mir::{function::MirFunction, BasicBlockId, ValueId};

/// Lower a direct Call where callee is provided as a const string ValueId in MIR14.
///
/// Requirements:
/// - `const_strs`: mapping from ValueId to the string literal value within the same function.
/// - `llvm_funcs`: predeclared LLVM functions keyed by MIR function name (same string as const).
pub(in super::super) fn lower_call<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    resolver: &mut super::Resolver<'ctx>,
    cur_bid: BasicBlockId,
    _func: &MirFunction,
    vmap: &mut HashMap<ValueId, BVE<'ctx>>,
    dst: &Option<ValueId>,
    callee: &ValueId,
    args: &[ValueId],
    const_strs: &HashMap<ValueId, String>,
    llvm_funcs: &HashMap<String, FunctionValue<'ctx>>,
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
        use inkwell::types::BasicMetadataTypeEnum as BMTy;
        let coerced: BVE<'ctx> = match exp_tys[i] {
            BMTy::IntType(it) => {
                // Localize as i64, then adjust width to callee expectation
                let iv = resolver.resolve_i64(
                    codegen,
                    cursor,
                    cur_bid,
                    *a,
                    bb_map,
                    preds,
                    block_end_values,
                    vmap,
                )?;
                let bw_dst = it.get_bit_width();
                let bw_src = iv.get_type().get_bit_width();
                if bw_src == bw_dst {
                    iv.into()
                } else if bw_src < bw_dst {
                    cursor
                        .emit_instr(cur_bid, |b| b.build_int_z_extend(iv, it, "call_arg_zext"))
                        .map_err(|e| e.to_string())?
                        .into()
                } else if bw_dst == 1 {
                    super::super::types::to_bool(codegen.context, iv.into(), &codegen.builder)?
                        .into()
                } else {
                    cursor
                        .emit_instr(cur_bid, |b| b.build_int_truncate(iv, it, "call_arg_trunc"))
                        .map_err(|e| e.to_string())?
                        .into()
                }
            }
            BMTy::PointerType(pt) => {
                // Localize as i64 handle and convert to expected pointer type
                let iv = resolver.resolve_i64(
                    codegen,
                    cursor,
                    cur_bid,
                    *a,
                    bb_map,
                    preds,
                    block_end_values,
                    vmap,
                )?;
                let p = cursor
                    .emit_instr(cur_bid, |b| b.build_int_to_ptr(iv, pt, "call_arg_i2p"))
                    .map_err(|e| e.to_string())?;
                p.into()
            }
            BMTy::FloatType(ft) => {
                // Localize as f64, then adjust to callee expectation width if needed
                let fv = resolver.resolve_f64(
                    codegen,
                    cursor,
                    cur_bid,
                    *a,
                    bb_map,
                    preds,
                    block_end_values,
                    vmap,
                )?;
                if fv.get_type() == ft {
                    fv.into()
                } else {
                    // Cast f64<->f32 as needed
                    cursor
                        .emit_instr(cur_bid, |b| b.build_float_cast(fv, ft, "call_arg_fcast"))
                        .map_err(|e| e.to_string())?
                        .into()
                }
            }
            _ => {
                return Err("call: unsupported parameter type (expected int/ptr/float)".to_string());
            }
        };
        params.push(coerced.into());
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
