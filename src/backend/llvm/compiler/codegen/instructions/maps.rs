use std::collections::HashMap;

use inkwell::{values::BasicValueEnum as BVE, AddressSpace};

use super::builder_cursor::BuilderCursor;
use crate::backend::llvm::context::CodegenContext;
use crate::mir::{function::MirFunction, BasicBlockId, ValueId};

/// Handle MapBox fast-paths (core-first). Returns true if handled.
pub(super) fn try_handle_map_method<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    resolver: &mut super::Resolver<'ctx>,
    cur_bid: BasicBlockId,
    func: &MirFunction,
    vmap: &mut HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    dst: &Option<ValueId>,
    box_val: &ValueId,
    method: &str,
    args: &[ValueId],
    recv_h: inkwell::values::IntValue<'ctx>,
) -> Result<bool, String> {
    // Only when receiver is annotated as MapBox
    let is_map_annot = matches!(func.metadata.value_types.get(box_val), Some(crate::mir::MirType::Box(b)) if b == "MapBox");
    if !is_map_annot {
        return Ok(false);
    }
    let i64t = codegen.context.i64_type();
    match method {
        "size" => {
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!("[LLVM] lower Map.size (core)");
            }
            if !args.is_empty() {
                return Err("MapBox.size expects 0 arg".to_string());
            }
            let fnty = i64t.fn_type(&[i64t.into()], false);
            let callee = codegen
                .module
                .get_function("nyash.map.size_h")
                .unwrap_or_else(|| codegen.module.add_function("nyash.map.size_h", fnty, None));
            let call = cursor
                .emit_instr(cur_bid, |b| b.build_call(callee, &[recv_h.into()], "msize"))
                .map_err(|e| e.to_string())?;
            if let Some(d) = dst {
                let rv = call
                    .try_as_basic_value()
                    .left()
                    .ok_or("map.size_h returned void".to_string())?;
                vmap.insert(*d, rv);
            }
            Ok(true)
        }
        "has" => {
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!("[LLVM] lower Map.has (core)");
            }
            if args.len() != 1 {
                return Err("MapBox.has expects 1 arg".to_string());
            }
            let key_i = match func.metadata.value_types.get(&args[0]) {
                Some(crate::mir::MirType::String) => {
                    // string key: i8* -> handle
                    let pv = resolver.resolve_ptr(
                        codegen,
                        cursor,
                        cur_bid,
                        args[0],
                        &std::collections::HashMap::new(),
                        &std::collections::HashMap::new(),
                        &std::collections::HashMap::new(),
                        vmap,
                    )?;
                    let fnty_conv = i64t.fn_type(
                        &[codegen.context.ptr_type(AddressSpace::from(0)).into()],
                        false,
                    );
                    let conv = codegen
                        .module
                        .get_function("nyash.box.from_i8_string")
                        .unwrap_or_else(|| {
                            codegen
                                .module
                                .add_function("nyash.box.from_i8_string", fnty_conv, None)
                        });
                    let kcall = cursor
                        .emit_instr(cur_bid, |b| {
                            b.build_call(conv, &[pv.into()], "key_i8_to_handle")
                        })
                        .map_err(|e| e.to_string())?;
                    kcall
                        .try_as_basic_value()
                        .left()
                        .ok_or("from_i8_string returned void".to_string())?
                        .into_int_value()
                }
                _ => resolver.resolve_i64(
                    codegen,
                    cursor,
                    cur_bid,
                    args[0],
                    &std::collections::HashMap::new(),
                    &std::collections::HashMap::new(),
                    &std::collections::HashMap::new(),
                    vmap,
                )?,
            };
            let fnty = i64t.fn_type(&[i64t.into(), i64t.into()], false);
            let callee = codegen
                .module
                .get_function("nyash.map.has_h")
                .unwrap_or_else(|| codegen.module.add_function("nyash.map.has_h", fnty, None));
            let call = cursor
                .emit_instr(cur_bid, |b| {
                    b.build_call(callee, &[recv_h.into(), key_i.into()], "mhas")
                })
                .map_err(|e| e.to_string())?;
            if let Some(d) = dst {
                let rv = call
                    .try_as_basic_value()
                    .left()
                    .ok_or("map.has_h returned void".to_string())?;
                vmap.insert(*d, rv);
            }
            Ok(true)
        }
        "get" => {
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!("[LLVM] lower Map.get (core)");
            }
            if args.len() != 1 {
                return Err("MapBox.get expects 1 arg".to_string());
            }
            let call = match func.metadata.value_types.get(&args[0]) {
                Some(crate::mir::MirType::String) => {
                    // key: i8* -> i64 handle via from_i8_string (string key)
                    let pv = resolver.resolve_ptr(
                        codegen,
                        cursor,
                        cur_bid,
                        args[0],
                        &std::collections::HashMap::new(),
                        &std::collections::HashMap::new(),
                        &std::collections::HashMap::new(),
                        vmap,
                    )?;
                    let fnty_conv = i64t.fn_type(
                        &[codegen.context.ptr_type(AddressSpace::from(0)).into()],
                        false,
                    );
                    let conv = codegen
                        .module
                        .get_function("nyash.box.from_i8_string")
                        .unwrap_or_else(|| {
                            codegen
                                .module
                                .add_function("nyash.box.from_i8_string", fnty_conv, None)
                        });
                    let kcall = cursor
                        .emit_instr(cur_bid, |b| {
                            b.build_call(conv, &[pv.into()], "key_i8_to_handle")
                        })
                        .map_err(|e| e.to_string())?;
                    let kh = kcall
                        .try_as_basic_value()
                        .left()
                        .ok_or("from_i8_string returned void".to_string())?
                        .into_int_value();
                    let fnty = i64t.fn_type(&[i64t.into(), i64t.into()], false);
                    let callee = codegen
                        .module
                        .get_function("nyash.map.get_hh")
                        .unwrap_or_else(|| {
                            codegen.module.add_function("nyash.map.get_hh", fnty, None)
                        });
                    cursor
                        .emit_instr(cur_bid, |b| {
                            b.build_call(callee, &[recv_h.into(), kh.into()], "mget_hh")
                        })
                        .map_err(|e| e.to_string())?
                }
                _ => {
                    let iv = resolver.resolve_i64(
                        codegen,
                        cursor,
                        cur_bid,
                        args[0],
                        &std::collections::HashMap::new(),
                        &std::collections::HashMap::new(),
                        &std::collections::HashMap::new(),
                        vmap,
                    )?;
                    let fnty = i64t.fn_type(&[i64t.into(), i64t.into()], false);
                    let callee = codegen
                        .module
                        .get_function("nyash.map.get_h")
                        .unwrap_or_else(|| {
                            codegen.module.add_function("nyash.map.get_h", fnty, None)
                        });
                    cursor
                        .emit_instr(cur_bid, |b| {
                            b.build_call(callee, &[recv_h.into(), iv.into()], "mget")
                        })
                        .map_err(|e| e.to_string())?
                }
            };
            if let Some(d) = dst {
                let rv = call
                    .try_as_basic_value()
                    .left()
                    .ok_or("map.get returned void".to_string())?;
                vmap.insert(*d, rv);
            }
            Ok(true)
        }
        "set" => {
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!("[LLVM] lower Map.set (core)");
            }
            if args.len() != 2 {
                return Err("MapBox.set expects 2 args (key, value)".to_string());
            }
            let key_i = match func.metadata.value_types.get(&args[0]) {
                Some(crate::mir::MirType::String) => {
                    let pv = resolver.resolve_ptr(
                        codegen,
                        cursor,
                        cur_bid,
                        args[0],
                        &std::collections::HashMap::new(),
                        &std::collections::HashMap::new(),
                        &std::collections::HashMap::new(),
                        vmap,
                    )?;
                    let fnty_conv = i64t.fn_type(
                        &[codegen.context.ptr_type(AddressSpace::from(0)).into()],
                        false,
                    );
                    let conv = codegen
                        .module
                        .get_function("nyash.box.from_i8_string")
                        .unwrap_or_else(|| {
                            codegen
                                .module
                                .add_function("nyash.box.from_i8_string", fnty_conv, None)
                        });
                    let kcall = cursor
                        .emit_instr(cur_bid, |b| {
                            b.build_call(conv, &[pv.into()], "key_i8_to_handle")
                        })
                        .map_err(|e| e.to_string())?;
                    kcall
                        .try_as_basic_value()
                        .left()
                        .ok_or("from_i8_string returned void".to_string())?
                        .into_int_value()
                }
                _ => resolver.resolve_i64(
                    codegen,
                    cursor,
                    cur_bid,
                    args[0],
                    &std::collections::HashMap::new(),
                    &std::collections::HashMap::new(),
                    &std::collections::HashMap::new(),
                    vmap,
                )?,
            };
            let val_i = resolver.resolve_i64(
                codegen,
                cursor,
                cur_bid,
                args[1],
                &std::collections::HashMap::new(),
                &std::collections::HashMap::new(),
                &std::collections::HashMap::new(),
                vmap,
            )?;
            let fnty = i64t.fn_type(&[i64t.into(), i64t.into(), i64t.into()], false);
            let callee = codegen
                .module
                .get_function("nyash.map.set_h")
                .unwrap_or_else(|| codegen.module.add_function("nyash.map.set_h", fnty, None));
            let _ = cursor
                .emit_instr(cur_bid, |b| {
                    b.build_call(callee, &[recv_h.into(), key_i.into(), val_i.into()], "mset")
                })
                .map_err(|e| e.to_string())?;
            Ok(true)
        }
        _ => Ok(false),
    }
}
