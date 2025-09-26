use std::collections::HashMap;

use inkwell::values::BasicValueEnum;

use crate::backend::llvm::context::CodegenContext;
use crate::mir::{instruction::ConstValue, ValueId};

pub(in super::super) fn lower_const<'ctx>(
    codegen: &CodegenContext<'ctx>,
    vmap: &mut HashMap<ValueId, BasicValueEnum<'ctx>>,
    dst: ValueId,
    value: &ConstValue,
) -> Result<(), String> {
    let bval = match value {
        ConstValue::Integer(i) => codegen.context.i64_type().const_int(*i as u64, true).into(),
        ConstValue::Float(f) => codegen.context.f64_type().const_float(*f).into(),
        ConstValue::Bool(b) => codegen
            .context
            .bool_type()
            .const_int(*b as u64, false)
            .into(),
        ConstValue::String(s) => {
            let gv = codegen
                .builder
                .build_global_string_ptr(s, "str")
                .map_err(|e| e.to_string())?;
            let len = codegen.context.i32_type().const_int(s.len() as u64, false);
            // declare i8* @nyash_string_new(i8*, i32)
            let rt = codegen.context.ptr_type(inkwell::AddressSpace::from(0));
            let fn_ty = rt.fn_type(
                &[
                    codegen
                        .context
                        .ptr_type(inkwell::AddressSpace::from(0))
                        .into(),
                    codegen.context.i32_type().into(),
                ],
                false,
            );
            let callee = codegen
                .module
                .get_function("nyash_string_new")
                .unwrap_or_else(|| codegen.module.add_function("nyash_string_new", fn_ty, None));
            let call = codegen
                .builder
                .build_call(
                    callee,
                    &[gv.as_pointer_value().into(), len.into()],
                    "strnew",
                )
                .map_err(|e| e.to_string())?;
            call.try_as_basic_value()
                .left()
                .ok_or("nyash_string_new returned void".to_string())?
        }
        ConstValue::Null => codegen
            .context
            .ptr_type(inkwell::AddressSpace::from(0))
            .const_null()
            .into(),
        ConstValue::Void => return Err("Const Void unsupported".to_string()),
    };
    vmap.insert(dst, bval);
    Ok(())
}
