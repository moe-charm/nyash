use std::collections::HashMap;

use inkwell::{values::BasicValueEnum as BVE, AddressSpace};

use crate::backend::llvm::context::CodegenContext;
use crate::mir::ValueId;

/// Handle getField/setField; returns true if handled.
use super::super::builder_cursor::BuilderCursor;

pub(super) fn try_handle_field_method<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    _cursor: &mut BuilderCursor<'ctx, 'b>,
    _cur_bid: crate::mir::BasicBlockId,
    vmap: &mut HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    dst: &Option<ValueId>,
    method: &str,
    args: &[ValueId],
    recv_h: inkwell::values::IntValue<'ctx>,
) -> Result<bool, String> {
    let i64t = codegen.context.i64_type();
    match method {
        "getField" => {
            if args.len() != 1 {
                return Err("getField expects 1 arg (name)".to_string());
            }
            let name_p = match vmap.get(&args[0]).copied() {
                Some(BVE::PointerValue(pv)) => pv,
                _ => return Err("getField name must be pointer".to_string()),
            };
            let i8p = codegen.context.ptr_type(AddressSpace::from(0));
            let fnty = i64t.fn_type(&[i64t.into(), i8p.into()], false);
            let callee = codegen
                .module
                .get_function("nyash.instance.get_field_h")
                .unwrap_or_else(|| codegen.module.add_function("nyash.instance.get_field_h", fnty, None));
            let call = codegen
                .builder
                .build_call(callee, &[recv_h.into(), name_p.into()], "getField")
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
            let name_p = match vmap.get(&args[0]).copied() {
                Some(BVE::PointerValue(pv)) => pv,
                _ => return Err("setField name must be pointer".to_string()),
            };
            let val_h = match vmap.get(&args[1]).copied() {
                Some(BVE::PointerValue(pv)) => codegen.builder.build_ptr_to_int(pv, i64t, "valp2i").map_err(|e| e.to_string())?,
                Some(BVE::IntValue(iv)) => iv,
                _ => return Err("setField value must be int/handle".to_string()),
            };
            let i8p = codegen.context.ptr_type(AddressSpace::from(0));
            let fnty = i64t.fn_type(&[i64t.into(), i8p.into(), i64t.into()], false);
            let callee = codegen
                .module
                .get_function("nyash.instance.set_field_h")
                .unwrap_or_else(|| codegen.module.add_function("nyash.instance.set_field_h", fnty, None));
            let _ = codegen
                .builder
                .build_call(callee, &[recv_h.into(), name_p.into(), val_h.into()], "setField")
                .map_err(|e| e.to_string())?;
            Ok(true)
        }
        _ => Ok(false),
    }
}
