use std::collections::HashMap;

use inkwell::values::BasicValueEnum as BVE;
use inkwell::AddressSpace;

use crate::backend::llvm::context::CodegenContext;
use crate::mir::{function::MirFunction, ValueId};

pub(super) fn lower_future_spawn_instance<'ctx>(
    codegen: &CodegenContext<'ctx>,
    vmap: &mut HashMap<ValueId, BVE<'ctx>>,
    dst: &Option<ValueId>,
    args: &[ValueId],
) -> Result<(), String> {
    if args.len() < 2 {
        return Err("env.future.spawn_instance expects at least (recv, method_name)".to_string());
    }
    let i64t = codegen.context.i64_type();
    let i8p = codegen.context.ptr_type(AddressSpace::from(0));
    let recv_v = *vmap.get(&args[0]).ok_or("recv missing")?;
    let recv_h = match recv_v {
        BVE::IntValue(iv) => iv,
        BVE::PointerValue(pv) => codegen
            .builder
            .build_ptr_to_int(pv, i64t, "recv_p2i")
            .map_err(|e| e.to_string())?,
        _ => return Err("spawn_instance recv must be int or ptr".to_string()),
    };
    let name_v = *vmap.get(&args[1]).ok_or("method name missing")?;
    let name_p = match name_v {
        BVE::PointerValue(pv) => pv,
        _ => return Err("spawn_instance method name must be i8*".to_string()),
    };
    let fnty = i64t.fn_type(&[i64t.into(), i8p.into()], false);
    let callee = codegen
        .module
        .get_function("nyash.future.spawn_instance")
        .unwrap_or_else(|| codegen.module.add_function("nyash.future.spawn_instance", fnty, None));
    let call = codegen
        .builder
        .build_call(callee, &[recv_h.into(), name_p.into()], "spawn_instance")
        .map_err(|e| e.to_string())?;
    if let Some(d) = dst {
        let rv = call
            .try_as_basic_value()
            .left()
            .ok_or("spawn_instance returned void".to_string())?;
        vmap.insert(*d, rv);
    }
    Ok(())
}

pub(super) fn lower_local_get<'ctx>(
    codegen: &CodegenContext<'ctx>,
    func: &MirFunction,
    vmap: &mut HashMap<ValueId, BVE<'ctx>>,
    dst: &Option<ValueId>,
    args: &[ValueId],
) -> Result<(), String> {
    if args.len() != 1 {
        return Err("env.local.get expects 1 arg".to_string());
    }
    let name_v = *vmap.get(&args[0]).ok_or("local.get name missing")?;
    let name_p = if let BVE::PointerValue(pv) = name_v {
        pv
    } else {
        return Err("env.local.get name must be i8*".to_string());
    };
    let i64t = codegen.context.i64_type();
    let i8p = codegen.context.ptr_type(AddressSpace::from(0));
    let fnty = i64t.fn_type(&[i8p.into()], false);
    let callee = codegen
        .module
        .get_function("nyash.env.local.get_h")
        .unwrap_or_else(|| codegen.module.add_function("nyash.env.local.get_h", fnty, None));
    let call = codegen
        .builder
        .build_call(callee, &[name_p.into()], "local_get_h")
        .map_err(|e| e.to_string())?;
    let rv = call
        .try_as_basic_value()
        .left()
        .ok_or("local.get returned void".to_string())?;
    // Cast handle to pointer for Box-like return types
    if let Some(d) = dst {
        if let Some(mt) = func.metadata.value_types.get(d) {
            match mt {
                crate::mir::MirType::Integer | crate::mir::MirType::Bool => {
                    vmap.insert(*d, rv);
                }
                crate::mir::MirType::String => {
                    // keep as handle (i64)
                    vmap.insert(*d, rv);
                }
                crate::mir::MirType::Box(_)
                | crate::mir::MirType::Array(_)
                | crate::mir::MirType::Future(_)
                | crate::mir::MirType::Unknown => {
                    let h = rv.into_int_value();
                    let pty = codegen.context.ptr_type(AddressSpace::from(0));
                    let ptr = codegen
                        .builder
                        .build_int_to_ptr(h, pty, "local_get_handle_to_ptr")
                        .map_err(|e| e.to_string())?;
                    vmap.insert(*d, ptr.into());
                }
                _ => {
                    vmap.insert(*d, rv);
                }
            }
        } else {
            vmap.insert(*d, rv);
        }
    }
    Ok(())
}

pub(super) fn lower_box_new<'ctx>(
    codegen: &CodegenContext<'ctx>,
    vmap: &mut HashMap<ValueId, BVE<'ctx>>,
    dst: &Option<ValueId>,
    args: &[ValueId],
) -> Result<(), String> {
    // Two variants: (name) and (argc, arg1, arg2, arg3, arg4) with optional ptr conversion
    // Prefer the i64 birth when possible; else call env.box.new(name)
    let i64t = codegen.context.i64_type();
    let i8p = codegen.context.ptr_type(AddressSpace::from(0));
    if args.len() == 1 {
        let name_v = *vmap.get(&args[0]).ok_or("env.box.new name missing")?;
        let name_p = if let BVE::PointerValue(pv) = name_v {
            pv
        } else {
            return Err("env.box.new name must be i8*".to_string());
        };
        let fnty = i64t.fn_type(&[i8p.into()], false);
        let callee = codegen
            .module
            .get_function("nyash.env.box.new")
            .unwrap_or_else(|| codegen.module.add_function("nyash.env.box.new", fnty, None));
        let call = codegen
            .builder
            .build_call(callee, &[name_p.into()], "env_box_new")
            .map_err(|e| e.to_string())?;
        let h = call
            .try_as_basic_value()
            .left()
            .ok_or("env.box.new returned void".to_string())?
            .into_int_value();
        let out_ptr = codegen
            .builder
            .build_int_to_ptr(h, i8p, "box_handle_to_ptr")
            .map_err(|e| e.to_string())?;
        if let Some(d) = dst {
            vmap.insert(*d, out_ptr.into());
        }
        return Ok(());
    }
    if !args.is_empty() {
        // argc + up to 4 i64 payloads: build i64 via conversions
        let argc_val = i64t.const_int(args.len() as u64, false);
        let fnty = i64t.fn_type(
            &[
                i8p.into(),
                i64t.into(),
                i64t.into(),
                i64t.into(),
                i64t.into(),
                i64t.into(),
            ],
            false,
        );
        let callee = codegen
            .module
            .get_function("nyash.env.box.new_i64")
            .unwrap_or_else(|| codegen.module.add_function("nyash.env.box.new_i64", fnty, None));
        // arg0: type name string pointer
        if args.is_empty() {
            return Err("env.box.new_i64 requires at least type name".to_string());
        }
        let ty_ptr = match *vmap.get(&args[0]).ok_or("type name missing")? {
            BVE::PointerValue(pv) => pv,
            _ => return Err("env.box.new_i64 arg0 must be i8* type name".to_string()),
        };
        let mut a1 = i64t.const_zero();
        if args.len() >= 2 {
            let bv = *vmap.get(&args[1]).ok_or("arg missing")?;
            a1 = match bv {
                BVE::IntValue(iv) => iv,
                BVE::FloatValue(fv) => {
                    let fnty = i64t.fn_type(&[codegen.context.f64_type().into()], false);
                    let callee = codegen
                        .module
                        .get_function("nyash.box.from_f64")
                        .unwrap_or_else(|| codegen.module.add_function("nyash.box.from_f64", fnty, None));
                    let call = codegen
                        .builder
                        .build_call(callee, &[fv.into()], "arg1_f64_to_box")
                        .map_err(|e| e.to_string())?;
                    let rv = call
                        .try_as_basic_value()
                        .left()
                        .ok_or("from_f64 returned void".to_string())?;
                    if let BVE::IntValue(h) = rv { h } else { return Err("from_f64 ret expected i64".to_string()); }
                }
                BVE::PointerValue(pv) => {
                    let fnty = i64t.fn_type(&[i8p.into()], false);
                    let callee = codegen
                        .module
                        .get_function("nyash.box.from_i8_string")
                        .unwrap_or_else(|| codegen.module.add_function("nyash.box.from_i8_string", fnty, None));
                    let call = codegen
                        .builder
                        .build_call(callee, &[pv.into()], "arg1_i8_to_box")
                        .map_err(|e| e.to_string())?;
                    let rv = call.try_as_basic_value().left().ok_or("from_i8_string returned void".to_string())?;
                    if let BVE::IntValue(h) = rv { h } else { return Err("from_i8_string ret expected i64".to_string()); }
                }
                _ => return Err("unsupported arg value for env.box.new".to_string()),
            };
        }
        let mut a2 = i64t.const_zero();
        if args.len() >= 3 {
            let bv = *vmap.get(&args[2]).ok_or("arg missing")?;
            a2 = match bv {
                BVE::IntValue(iv) => iv,
                BVE::FloatValue(fv) => {
                    let fnty = i64t.fn_type(&[codegen.context.f64_type().into()], false);
                    let callee = codegen
                        .module
                        .get_function("nyash.box.from_f64")
                        .unwrap_or_else(|| codegen.module.add_function("nyash.box.from_f64", fnty, None));
                    let call = codegen
                        .builder
                        .build_call(callee, &[fv.into()], "arg2_f64_to_box")
                        .map_err(|e| e.to_string())?;
                    let rv = call
                        .try_as_basic_value()
                        .left()
                        .ok_or("from_f64 returned void".to_string())?;
                    if let BVE::IntValue(h) = rv { h } else { return Err("from_f64 ret expected i64".to_string()); }
                }
                BVE::PointerValue(pv) => {
                    let fnty = i64t.fn_type(&[i8p.into()], false);
                    let callee = codegen
                        .module
                        .get_function("nyash.box.from_i8_string")
                        .unwrap_or_else(|| codegen.module.add_function("nyash.box.from_i8_string", fnty, None));
                    let call = codegen
                        .builder
                        .build_call(callee, &[pv.into()], "arg2_i8_to_box")
                        .map_err(|e| e.to_string())?;
                    let rv = call.try_as_basic_value().left().ok_or("from_i8_string returned void".to_string())?;
                    if let BVE::IntValue(h) = rv { h } else { return Err("from_i8_string ret expected i64".to_string()); }
                }
                _ => return Err("unsupported arg value for env.box.new".to_string()),
            };
        }
        let mut a3 = i64t.const_zero();
        if args.len() >= 4 {
            let bv = *vmap.get(&args[3]).ok_or("arg missing")?;
            a3 = match bv {
                BVE::IntValue(iv) => iv,
                BVE::FloatValue(fv) => {
                    let fnty = i64t.fn_type(&[codegen.context.f64_type().into()], false);
                    let callee = codegen
                        .module
                        .get_function("nyash.box.from_f64")
                        .unwrap_or_else(|| codegen.module.add_function("nyash.box.from_f64", fnty, None));
                    let call = codegen
                        .builder
                        .build_call(callee, &[fv.into()], "arg3_f64_to_box")
                        .map_err(|e| e.to_string())?;
                    let rv = call
                        .try_as_basic_value()
                        .left()
                        .ok_or("from_f64 returned void".to_string())?;
                    if let BVE::IntValue(h) = rv { h } else { return Err("from_f64 ret expected i64".to_string()); }
                }
                BVE::PointerValue(pv) => {
                    let fnty = i64t.fn_type(&[i8p.into()], false);
                    let callee = codegen
                        .module
                        .get_function("nyash.box.from_i8_string")
                        .unwrap_or_else(|| codegen.module.add_function("nyash.box.from_i8_string", fnty, None));
                    let call = codegen
                        .builder
                        .build_call(callee, &[pv.into()], "arg3_i8_to_box")
                        .map_err(|e| e.to_string())?;
                    let rv = call.try_as_basic_value().left().ok_or("from_i8_string returned void".to_string())?;
                    if let BVE::IntValue(h) = rv { h } else { return Err("from_i8_string ret expected i64".to_string()); }
                }
                _ => return Err("unsupported arg value for env.box.new".to_string()),
            };
        }
        let mut a4 = i64t.const_zero();
        if args.len() >= 5 {
            let bv = *vmap.get(&args[4]).ok_or("arg missing")?;
            a4 = match bv {
                BVE::IntValue(iv) => iv,
                BVE::FloatValue(fv) => {
                    let fnty = i64t.fn_type(&[codegen.context.f64_type().into()], false);
                    let callee = codegen
                        .module
                        .get_function("nyash.box.from_f64")
                        .unwrap_or_else(|| codegen.module.add_function("nyash.box.from_f64", fnty, None));
                    let call = codegen
                        .builder
                        .build_call(callee, &[fv.into()], "arg4_f64_to_box")
                        .map_err(|e| e.to_string())?;
                    let rv = call
                        .try_as_basic_value()
                        .left()
                        .ok_or("from_f64 returned void".to_string())?;
                    if let BVE::IntValue(h) = rv { h } else { return Err("from_f64 ret expected i64".to_string()); }
                }
                BVE::PointerValue(pv) => {
                    let fnty = i64t.fn_type(&[i8p.into()], false);
                    let callee = codegen
                        .module
                        .get_function("nyash.box.from_i8_string")
                        .unwrap_or_else(|| codegen.module.add_function("nyash.box.from_i8_string", fnty, None));
                    let call = codegen
                        .builder
                        .build_call(callee, &[pv.into()], "arg4_i8_to_box")
                        .map_err(|e| e.to_string())?;
                    let rv = call.try_as_basic_value().left().ok_or("from_i8_string returned void".to_string())?;
                    if let BVE::IntValue(h) = rv { h } else { return Err("from_i8_string ret expected i64".to_string()); }
                }
                _ => return Err("unsupported arg value for env.box.new".to_string()),
            };
        }
        let call = codegen
            .builder
            .build_call(
                callee,
                &[ty_ptr.into(), argc_val.into(), a1.into(), a2.into(), a3.into(), a4.into()],
                "env_box_new_i64x",
            )
            .map_err(|e| e.to_string())?;
        let rv = call
            .try_as_basic_value()
            .left()
            .ok_or("env.box.new_i64 returned void".to_string())?;
        let i64v = if let BVE::IntValue(iv) = rv { iv } else { return Err("env.box.new_i64 ret expected i64".to_string()); };
        let out_ptr = codegen
            .builder
            .build_int_to_ptr(i64v, i8p, "box_handle_to_ptr")
            .map_err(|e| e.to_string())?;
        if let Some(d) = dst {
            vmap.insert(*d, out_ptr.into());
        }
        return Ok(());
    }
    Err("env.box.new requires at least 1 arg".to_string())
}

