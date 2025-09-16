use std::collections::HashMap;

use crate::backend::llvm::compiler::codegen::types;
use inkwell::{values::BasicValueEnum as BVE, AddressSpace};

use crate::backend::llvm::context::CodegenContext;
use crate::mir::{function::MirFunction, ValueId};

/// Convert a value to i64 handle/int for plugin invoke (ptr->i64, f64->box->i64)
pub(super) fn get_i64<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut crate::backend::llvm::compiler::codegen::instructions::builder_cursor::BuilderCursor<'ctx, 'b>,
    resolver: &mut super::super::Resolver<'ctx>,
    cur_bid: crate::mir::BasicBlockId,
    func: &MirFunction,
    vmap: &HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    vid: ValueId,
    bb_map: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        inkwell::basic_block::BasicBlock<'ctx>,
    >,
    preds: &std::collections::HashMap<crate::mir::BasicBlockId, Vec<crate::mir::BasicBlockId>>,
    block_end_values: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        std::collections::HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    >,
) -> Result<inkwell::values::IntValue<'ctx>, String> {
    let i64t = codegen.context.i64_type();
    match func.metadata.value_types.get(&vid) {
        Some(crate::mir::MirType::Float) => {
            // Box f64 then use its handle
            let fv = resolver.resolve_f64(
                codegen,
                cursor,
                cur_bid,
                vid,
                bb_map,
                preds,
                block_end_values,
                vmap,
            )?;
            let fnty = i64t.fn_type(&[codegen.context.f64_type().into()], false);
            let callee = codegen
                .module
                .get_function("nyash.box.from_f64")
                .unwrap_or_else(|| {
                    codegen
                        .module
                        .add_function("nyash.box.from_f64", fnty, None)
                });
            let call = cursor
                .emit_instr(cur_bid, |b| {
                    b.build_call(callee, &[fv.into()], "arg_f64_to_box")
                })
                .map_err(|e| e.to_string())?;
            let rv = call
                .try_as_basic_value()
                .left()
                .ok_or("from_f64 returned void".to_string())?;
            if let BVE::IntValue(h) = rv {
                Ok(h)
            } else {
                Err("from_f64 ret expected i64".to_string())
            }
        }
        _ => resolver.resolve_i64(
            codegen,
            cursor,
            cur_bid,
            vid,
            bb_map,
            preds,
            block_end_values,
            vmap,
        ),
    }
}

/// Classify a value into tag constant i64 (uses types::classify_tag)
pub(super) fn get_tag_const<'ctx>(
    codegen: &CodegenContext<'ctx>,
    func: &MirFunction,
    vid: ValueId,
) -> inkwell::values::IntValue<'ctx> {
    let i64t = codegen.context.i64_type();
    let tag = match func.metadata.value_types.get(&vid) {
        Some(crate::mir::MirType::Float) => 5,
        Some(crate::mir::MirType::String)
        | Some(crate::mir::MirType::Box(_))
        | Some(crate::mir::MirType::Array(_))
        | Some(crate::mir::MirType::Future(_))
        | Some(crate::mir::MirType::Unknown) => 8,
        _ => 3,
    };
    i64t.const_int(tag as u64, false)
}
