use std::collections::HashMap;

use inkwell::values::BasicValueEnum as BVE;
use inkwell::AddressSpace;

use crate::backend::llvm::compiler::codegen::instructions::builder_cursor::BuilderCursor;
use crate::backend::llvm::context::CodegenContext;
use crate::mir::{function::MirFunction, BasicBlockId, ValueId};

pub(super) fn lower_future_spawn_instance<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    resolver: &mut super::super::Resolver<'ctx>,
    cur_bid: BasicBlockId,
    vmap: &mut HashMap<ValueId, BVE<'ctx>>,
    dst: &Option<ValueId>,
    args: &[ValueId],
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
    if args.len() < 2 {
        return Err("env.future.spawn_instance expects at least (recv, method_name)".to_string());
    }
    let i64t = codegen.context.i64_type();
    let i8p = codegen.context.ptr_type(AddressSpace::from(0));
    let recv_h = resolver.resolve_i64(
        codegen,
        cursor,
        cur_bid,
        args[0],
        bb_map,
        preds,
        block_end_values,
        vmap,
    )?;
    let name_p = resolver.resolve_ptr(
        codegen,
        cursor,
        cur_bid,
        args[1],
        bb_map,
        preds,
        block_end_values,
        vmap,
    )?;
    let fnty = i64t.fn_type(&[i64t.into(), i8p.into()], false);
    let callee = codegen
        .module
        .get_function("nyash.future.spawn_instance")
        .unwrap_or_else(|| {
            codegen
                .module
                .add_function("nyash.future.spawn_instance", fnty, None)
        });
    let call = cursor
        .emit_instr(cur_bid, |b| {
            b.build_call(callee, &[recv_h.into(), name_p.into()], "spawn_instance")
        })
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

pub(super) fn lower_local_get<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    _resolver: &mut super::super::Resolver<'ctx>,
    cur_bid: BasicBlockId,
    func: &MirFunction,
    vmap: &mut HashMap<ValueId, BVE<'ctx>>,
    dst: &Option<ValueId>,
    args: &[ValueId],
    _bb_map: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        inkwell::basic_block::BasicBlock<'ctx>,
    >,
    _preds: &std::collections::HashMap<crate::mir::BasicBlockId, Vec<crate::mir::BasicBlockId>>,
    _block_end_values: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        std::collections::HashMap<ValueId, BVE<'ctx>>,
    >,
) -> Result<(), String> {
    if args.len() != 1 {
        return Err("env.local.get expects 1 arg".to_string());
    }
    let name_p = _resolver.resolve_ptr(
        codegen,
        cursor,
        cur_bid,
        args[0],
        _bb_map,
        _preds,
        _block_end_values,
        vmap,
    )?;
    let i64t = codegen.context.i64_type();
    let i8p = codegen.context.ptr_type(AddressSpace::from(0));
    let fnty = i64t.fn_type(&[i8p.into()], false);
    let callee = codegen
        .module
        .get_function("nyash.env.local.get_h")
        .unwrap_or_else(|| {
            codegen
                .module
                .add_function("nyash.env.local.get_h", fnty, None)
        });
    let call = cursor
        .emit_instr(cur_bid, |b| {
            b.build_call(callee, &[name_p.into()], "local_get_h")
        })
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
                    let ptr = cursor
                        .emit_instr(cur_bid, |b| {
                            b.build_int_to_ptr(h, pty, "local_get_handle_to_ptr")
                        })
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

pub(super) fn lower_box_new<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    resolver: &mut super::super::Resolver<'ctx>,
    cur_bid: BasicBlockId,
    func: &MirFunction,
    vmap: &mut HashMap<ValueId, BVE<'ctx>>,
    dst: &Option<ValueId>,
    args: &[ValueId],
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
    // Two variants: (name) and (argc, arg1, arg2, arg3, arg4) with optional ptr conversion
    // Prefer the i64 birth when possible; else call env.box.new(name)
    let i64t = codegen.context.i64_type();
    let i8p = codegen.context.ptr_type(AddressSpace::from(0));
    if args.len() == 1 {
        let name_p = resolver.resolve_ptr(
            codegen,
            cursor,
            cur_bid,
            args[0],
            bb_map,
            preds,
            block_end_values,
            vmap,
        )?;
        let fnty = i64t.fn_type(&[i8p.into()], false);
        let callee = codegen
            .module
            .get_function("nyash.env.box.new")
            .unwrap_or_else(|| codegen.module.add_function("nyash.env.box.new", fnty, None));
        let call = cursor
            .emit_instr(cur_bid, |b| {
                b.build_call(callee, &[name_p.into()], "env_box_new")
            })
            .map_err(|e| e.to_string())?;
        let h = call
            .try_as_basic_value()
            .left()
            .ok_or("env.box.new returned void".to_string())?
            .into_int_value();
        let out_ptr = cursor
            .emit_instr(cur_bid, |b| b.build_int_to_ptr(h, i8p, "box_handle_to_ptr"))
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
            .unwrap_or_else(|| {
                codegen
                    .module
                    .add_function("nyash.env.box.new_i64", fnty, None)
            });
        // arg0: type name string pointer
        if args.is_empty() {
            return Err("env.box.new_i64 requires at least type name".to_string());
        }
        let ty_ptr = resolver.resolve_ptr(
            codegen,
            cursor,
            cur_bid,
            args[0],
            bb_map,
            preds,
            block_end_values,
            vmap,
        )?;
        let mut a1 = i64t.const_zero();
        if args.len() >= 2 {
            a1 = match func.metadata.value_types.get(&args[1]) {
                Some(crate::mir::MirType::Float) => {
                    let fv = resolver.resolve_f64(
                        codegen,
                        cursor,
                        cur_bid,
                        args[1],
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
                            b.build_call(callee, &[fv.into()], "arg1_f64_to_box")
                        })
                        .map_err(|e| e.to_string())?;
                    call.try_as_basic_value()
                        .left()
                        .ok_or("from_f64 returned void".to_string())?
                        .into_int_value()
                }
                Some(crate::mir::MirType::String) => {
                    let pv = resolver.resolve_ptr(
                        codegen,
                        cursor,
                        cur_bid,
                        args[1],
                        bb_map,
                        preds,
                        block_end_values,
                        vmap,
                    )?;
                    let fnty = i64t.fn_type(&[i8p.into()], false);
                    let callee = codegen
                        .module
                        .get_function("nyash.box.from_i8_string")
                        .unwrap_or_else(|| {
                            codegen
                                .module
                                .add_function("nyash.box.from_i8_string", fnty, None)
                        });
                    let call = cursor
                        .emit_instr(cur_bid, |b| {
                            b.build_call(callee, &[pv.into()], "arg1_i8_to_box")
                        })
                        .map_err(|e| e.to_string())?;
                    call.try_as_basic_value()
                        .left()
                        .ok_or("from_i8_string returned void".to_string())?
                        .into_int_value()
                }
                _ => resolver.resolve_i64(
                    codegen,
                    cursor,
                    cur_bid,
                    args[1],
                    bb_map,
                    preds,
                    block_end_values,
                    vmap,
                )?,
            };
        }
        let mut a2 = i64t.const_zero();
        if args.len() >= 3 {
            a2 = match func.metadata.value_types.get(&args[2]) {
                Some(crate::mir::MirType::Float) => {
                    let fv = resolver.resolve_f64(
                        codegen,
                        cursor,
                        cur_bid,
                        args[2],
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
                            b.build_call(callee, &[fv.into()], "arg2_f64_to_box")
                        })
                        .map_err(|e| e.to_string())?;
                    call.try_as_basic_value()
                        .left()
                        .ok_or("from_f64 returned void".to_string())?
                        .into_int_value()
                }
                Some(crate::mir::MirType::String) => {
                    let pv = resolver.resolve_ptr(
                        codegen,
                        cursor,
                        cur_bid,
                        args[2],
                        bb_map,
                        preds,
                        block_end_values,
                        vmap,
                    )?;
                    let fnty = i64t.fn_type(&[i8p.into()], false);
                    let callee = codegen
                        .module
                        .get_function("nyash.box.from_i8_string")
                        .unwrap_or_else(|| {
                            codegen
                                .module
                                .add_function("nyash.box.from_i8_string", fnty, None)
                        });
                    let call = cursor
                        .emit_instr(cur_bid, |b| {
                            b.build_call(callee, &[pv.into()], "arg2_i8_to_box")
                        })
                        .map_err(|e| e.to_string())?;
                    call.try_as_basic_value()
                        .left()
                        .ok_or("from_i8_string returned void".to_string())?
                        .into_int_value()
                }
                _ => resolver.resolve_i64(
                    codegen,
                    cursor,
                    cur_bid,
                    args[2],
                    bb_map,
                    preds,
                    block_end_values,
                    vmap,
                )?,
            };
        }
        let mut a3 = i64t.const_zero();
        if args.len() >= 4 {
            a3 = match func.metadata.value_types.get(&args[3]) {
                Some(crate::mir::MirType::Integer) | Some(crate::mir::MirType::Bool) => resolver
                    .resolve_i64(
                        codegen,
                        cursor,
                        cur_bid,
                        args[3],
                        bb_map,
                        preds,
                        block_end_values,
                        vmap,
                    )?,
                Some(crate::mir::MirType::Float) => {
                    let fv = resolver.resolve_f64(
                        codegen,
                        cursor,
                        cur_bid,
                        args[3],
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
                            b.build_call(callee, &[fv.into()], "arg3_f64_to_box")
                        })
                        .map_err(|e| e.to_string())?;
                    call.try_as_basic_value()
                        .left()
                        .ok_or("from_f64 returned void".to_string())?
                        .into_int_value()
                }
                Some(crate::mir::MirType::String) => {
                    let pv = resolver.resolve_ptr(
                        codegen,
                        cursor,
                        cur_bid,
                        args[3],
                        bb_map,
                        preds,
                        block_end_values,
                        vmap,
                    )?;
                    let fnty = i64t.fn_type(&[i8p.into()], false);
                    let callee = codegen
                        .module
                        .get_function("nyash.box.from_i8_string")
                        .unwrap_or_else(|| {
                            codegen
                                .module
                                .add_function("nyash.box.from_i8_string", fnty, None)
                        });
                    let call = cursor
                        .emit_instr(cur_bid, |b| {
                            b.build_call(callee, &[pv.into()], "arg3_i8_to_box")
                        })
                        .map_err(|e| e.to_string())?;
                    call.try_as_basic_value()
                        .left()
                        .ok_or("from_i8_string returned void".to_string())?
                        .into_int_value()
                }
                _ => return Err("unsupported arg value for env.box.new".to_string()),
            };
        }
        let mut a4 = i64t.const_zero();
        if args.len() >= 5 {
            a4 = match func.metadata.value_types.get(&args[4]) {
                Some(crate::mir::MirType::Integer) | Some(crate::mir::MirType::Bool) => resolver
                    .resolve_i64(
                        codegen,
                        cursor,
                        cur_bid,
                        args[4],
                        bb_map,
                        preds,
                        block_end_values,
                        vmap,
                    )?,
                Some(crate::mir::MirType::Float) => {
                    let fv = resolver.resolve_f64(
                        codegen,
                        cursor,
                        cur_bid,
                        args[4],
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
                            b.build_call(callee, &[fv.into()], "arg4_f64_to_box")
                        })
                        .map_err(|e| e.to_string())?;
                    call.try_as_basic_value()
                        .left()
                        .ok_or("from_f64 returned void".to_string())?
                        .into_int_value()
                }
                Some(crate::mir::MirType::String) => {
                    let pv = resolver.resolve_ptr(
                        codegen,
                        cursor,
                        cur_bid,
                        args[4],
                        bb_map,
                        preds,
                        block_end_values,
                        vmap,
                    )?;
                    let fnty = i64t.fn_type(&[i8p.into()], false);
                    let callee = codegen
                        .module
                        .get_function("nyash.box.from_i8_string")
                        .unwrap_or_else(|| {
                            codegen
                                .module
                                .add_function("nyash.box.from_i8_string", fnty, None)
                        });
                    let call = cursor
                        .emit_instr(cur_bid, |b| {
                            b.build_call(callee, &[pv.into()], "arg4_i8_to_box")
                        })
                        .map_err(|e| e.to_string())?;
                    call.try_as_basic_value()
                        .left()
                        .ok_or("from_i8_string returned void".to_string())?
                        .into_int_value()
                }
                _ => return Err("unsupported arg value for env.box.new".to_string()),
            };
        }
        let call = cursor
            .emit_instr(cur_bid, |b| {
                b.build_call(
                    callee,
                    &[
                        ty_ptr.into(),
                        argc_val.into(),
                        a1.into(),
                        a2.into(),
                        a3.into(),
                        a4.into(),
                    ],
                    "env_box_new_i64x",
                )
            })
            .map_err(|e| e.to_string())?;
        let rv = call
            .try_as_basic_value()
            .left()
            .ok_or("env.box.new_i64 returned void".to_string())?;
        let i64v = if let BVE::IntValue(iv) = rv {
            iv
        } else {
            return Err("env.box.new_i64 ret expected i64".to_string());
        };
        let out_ptr = cursor
            .emit_instr(cur_bid, |b| {
                b.build_int_to_ptr(i64v, i8p, "box_handle_to_ptr")
            })
            .map_err(|e| e.to_string())?;
        if let Some(d) = dst {
            vmap.insert(*d, out_ptr.into());
        }
        return Ok(());
    }
    Err("env.box.new requires at least 1 arg".to_string())
}
