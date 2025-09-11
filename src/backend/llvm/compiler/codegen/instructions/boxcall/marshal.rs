use std::collections::HashMap;

use inkwell::{values::BasicValueEnum as BVE, AddressSpace};
use crate::backend::llvm::compiler::codegen::types;

use crate::backend::llvm::context::CodegenContext;
use crate::mir::ValueId;

/// Convert a value to i64 handle/int for plugin invoke (ptr->i64, f64->box->i64)
pub(super) fn get_i64<'ctx>(
    codegen: &CodegenContext<'ctx>,
    vmap: &HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    vid: ValueId,
) -> Result<inkwell::values::IntValue<'ctx>, String> {
    let i64t = codegen.context.i64_type();
    let bv = *vmap.get(&vid).ok_or("arg missing")?;
    match bv {
        BVE::IntValue(iv) => Ok(iv),
        BVE::PointerValue(pv) => codegen
            .builder
            .build_ptr_to_int(pv, i64t, "p2i")
            .map_err(|e| e.to_string()),
        BVE::FloatValue(fv) => {
            let fnty = i64t.fn_type(&[codegen.context.f64_type().into()], false);
            let callee = codegen
                .module
                .get_function("nyash.box.from_f64")
                .unwrap_or_else(|| codegen.module.add_function("nyash.box.from_f64", fnty, None));
            let call = codegen
                .builder
                .build_call(callee, &[fv.into()], "arg_f64_to_box")
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
        _ => Err("unsupported arg value".to_string()),
    }
}

/// Classify a value into tag constant i64 (uses types::classify_tag)
pub(super) fn get_tag_const<'ctx>(
    codegen: &CodegenContext<'ctx>,
    vmap: &HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    vid: ValueId,
) -> inkwell::values::IntValue<'ctx> {
    let i64t = codegen.context.i64_type();
    let tag = vmap
        .get(&vid)
        .map(|v| types::classify_tag(*v))
        .unwrap_or(3);
    i64t.const_int(tag as u64, false)
}
