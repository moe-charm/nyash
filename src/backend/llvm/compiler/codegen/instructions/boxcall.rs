use std::collections::HashMap;

use inkwell::AddressSpace;
use inkwell::values::BasicValueEnum as BVE;

use crate::backend::llvm::context::CodegenContext;
mod fields;
mod invoke;
mod marshal;
use crate::mir::{function::MirFunction, ValueId};

// BoxCall lowering (large): mirrors existing logic; kept in one function for now
pub(in super::super) fn lower_boxcall<'ctx>(
    codegen: &CodegenContext<'ctx>,
    func: &MirFunction,
    vmap: &mut HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    dst: &Option<ValueId>,
    box_val: &ValueId,
    method: &str,
    method_id: &Option<u16>,
    args: &[ValueId],
    box_type_ids: &HashMap<String, i64>,
    entry_builder: &inkwell::builder::Builder<'ctx>,
) -> Result<(), String> {
    use crate::backend::llvm::compiler::helpers::{as_float, as_int};
    use super::super::types::classify_tag;
    let i64t = codegen.context.i64_type();
    let recv_v = *vmap.get(box_val).ok_or("box receiver missing")?;
    let recv_p = match recv_v {
        BVE::PointerValue(pv) => pv,
        BVE::IntValue(iv) => {
            let pty = codegen.context.ptr_type(AddressSpace::from(0));
            codegen
                .builder
                .build_int_to_ptr(iv, pty, "recv_i2p")
                .map_err(|e| e.to_string())?
        }
        _ => return Err("box receiver must be pointer or i64 handle".to_string()),
    };
    let recv_h = codegen
        .builder
        .build_ptr_to_int(recv_p, i64t, "recv_p2i")
        .map_err(|e| e.to_string())?;

    // Resolve type_id
    let type_id: i64 = if let Some(crate::mir::MirType::Box(bname)) = func.metadata.value_types.get(box_val) {
        *box_type_ids.get(bname).unwrap_or(&0)
    } else if let Some(crate::mir::MirType::String) = func.metadata.value_types.get(box_val) {
        *box_type_ids.get("StringBox").unwrap_or(&0)
    } else {
        0
    };

    // Delegate String methods
    if super::strings::try_handle_string_method(codegen, func, vmap, dst, box_val, method, args, recv_v)? {
        return Ok(());
    }

    // Delegate Array methods
    if super::arrays::try_handle_array_method(codegen, func, vmap, dst, box_val, method, args, recv_h)? {
        return Ok(());
    }

    // Delegate Map methods
    if super::maps::try_handle_map_method(codegen, func, vmap, dst, box_val, method, args, recv_h)? {
        return Ok(());
    }

    // getField/setField
    if fields::try_handle_field_method(codegen, vmap, dst, method, args, recv_h)? {
        return Ok(());
    }

    if let Some(mid) = method_id {
        invoke::try_handle_tagged_invoke(
            codegen,
            func,
            vmap,
            dst,
            *mid,
            type_id,
            recv_h,
            args,
            entry_builder,
        )?;
        return Ok(());
    } else {
        Err(format!("BoxCall requires method_id for method '{}'. The method_id should be automatically injected during MIR compilation.", method))
    }
}
