use std::collections::HashMap;

use inkwell::AddressSpace;
use inkwell::values::BasicValueEnum as BVE;

use crate::backend::llvm::context::CodegenContext;
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

    // String concat fast-path (avoid plugin path for builtin StringBox)
    if method == "concat" {
        // Recognize receiver typed as String or StringBox
        let is_string_recv = match func.metadata.value_types.get(box_val) {
            Some(crate::mir::MirType::String) => true,
            Some(crate::mir::MirType::Box(b)) if b == "StringBox" => true,
            _ => false,
        };
        if is_string_recv {
            if args.len() != 1 { return Err("String.concat expects 1 arg".to_string()); }
            // Prefer pointer-based concat to keep AOT string fast-path
            let i8p = codegen.context.ptr_type(AddressSpace::from(0));
            let rhs_v = *vmap.get(&args[0]).ok_or("concat arg missing")?;
            match (recv_v, rhs_v) {
                (BVE::PointerValue(lp), BVE::PointerValue(rp)) => {
                    let fnty = i8p.fn_type(&[i8p.into(), i8p.into()], false);
                    let callee = codegen.module.get_function("nyash.string.concat_ss").
                        unwrap_or_else(|| codegen.module.add_function("nyash.string.concat_ss", fnty, None));
                    let call = codegen.builder.build_call(callee, &[lp.into(), rp.into()], "concat_ss_call").map_err(|e| e.to_string())?;
                    if let Some(d) = dst { let rv = call.try_as_basic_value().left().ok_or("concat_ss returned void".to_string())?; vmap.insert(*d, rv); }
                    return Ok(());
                }
                (BVE::PointerValue(lp), BVE::IntValue(ri)) => {
                    let i64t = codegen.context.i64_type();
                    let fnty = i8p.fn_type(&[i8p.into(), i64t.into()], false);
                    let callee = codegen.module.get_function("nyash.string.concat_si").
                        unwrap_or_else(|| codegen.module.add_function("nyash.string.concat_si", fnty, None));
                    let call = codegen.builder.build_call(callee, &[lp.into(), ri.into()], "concat_si_call").map_err(|e| e.to_string())?;
                    if let Some(d) = dst { let rv = call.try_as_basic_value().left().ok_or("concat_si returned void".to_string())?; vmap.insert(*d, rv); }
                    return Ok(());
                }
                (BVE::IntValue(li), BVE::PointerValue(rp)) => {
                    let i64t = codegen.context.i64_type();
                    let fnty = i8p.fn_type(&[i64t.into(), i8p.into()], false);
                    let callee = codegen.module.get_function("nyash.string.concat_is").
                        unwrap_or_else(|| codegen.module.add_function("nyash.string.concat_is", fnty, None));
                    let call = codegen.builder.build_call(callee, &[li.into(), rp.into()], "concat_is_call").map_err(|e| e.to_string())?;
                    if let Some(d) = dst { let rv = call.try_as_basic_value().left().ok_or("concat_is returned void".to_string())?; vmap.insert(*d, rv); }
                    return Ok(());
                }
                _ => { /* fall through to generic path below */ }
            }
        }
    }

    // String length fast-path: length/len
    if method == "length" || method == "len" {
        // Only when receiver is String/StringBox by annotation
        let is_string_recv = match func.metadata.value_types.get(box_val) {
            Some(crate::mir::MirType::String) => true,
            Some(crate::mir::MirType::Box(b)) if b == "StringBox" => true,
            _ => false,
        };
        if is_string_recv {
            let i64t = codegen.context.i64_type();
            // Ensure we have a handle: convert i8* receiver to handle when needed
            let recv_h = match recv_v {
                BVE::IntValue(h) => h,
                BVE::PointerValue(p) => {
                    let fnty = i64t.fn_type(&[codegen.context.ptr_type(AddressSpace::from(0)).into()], false);
                    let callee = codegen
                        .module
                        .get_function("nyash.box.from_i8_string")
                        .unwrap_or_else(|| codegen.module.add_function("nyash.box.from_i8_string", fnty, None));
                    let call = codegen
                        .builder
                        .build_call(callee, &[p.into()], "str_ptr_to_handle")
                        .map_err(|e| e.to_string())?;
                    let rv = call
                        .try_as_basic_value()
                        .left()
                        .ok_or("from_i8_string returned void".to_string())?;
                    if let BVE::IntValue(iv) = rv { iv } else { return Err("from_i8_string ret expected i64".to_string()); }
                }
                _ => return Err("String.length receiver type unsupported".to_string()),
            };
            // call i64 @nyash.string.len_h(i64)
            let fnty = i64t.fn_type(&[i64t.into()], false);
            let callee = codegen
                .module
                .get_function("nyash.string.len_h")
                .unwrap_or_else(|| codegen.module.add_function("nyash.string.len_h", fnty, None));
            let call = codegen
                .builder
                .build_call(callee, &[recv_h.into()], "strlen_h")
                .map_err(|e| e.to_string())?;
            if let Some(d) = dst {
                let rv = call
                    .try_as_basic_value()
                    .left()
                    .ok_or("len_h returned void".to_string())?;
                vmap.insert(*d, rv);
            }
            return Ok(());
        }
    }

    // Array fast-paths
    if let Some(crate::mir::MirType::Box(bname)) = func.metadata.value_types.get(box_val) {
        if bname == "ArrayBox" && (method == "get" || method == "set" || method == "push" || method == "length") {
            match method {
                "get" => {
                    if args.len() != 1 { return Err("ArrayBox.get expects 1 arg".to_string()); }
                    let idx_v = *vmap.get(&args[0]).ok_or("array.get index missing")?;
                    let idx_i = if let BVE::IntValue(iv) = idx_v { iv } else { return Err("array.get index must be int".to_string()); };
                    let fnty = i64t.fn_type(&[i64t.into(), i64t.into()], false);
                    let callee = codegen.module.get_function("nyash_array_get_h").unwrap_or_else(|| codegen.module.add_function("nyash_array_get_h", fnty, None));
                    let call = codegen.builder.build_call(callee, &[recv_h.into(), idx_i.into()], "aget").map_err(|e| e.to_string())?;
                    if let Some(d) = dst {
                        let rv = call.try_as_basic_value().left().ok_or("array_get_h returned void".to_string())?;
                        vmap.insert(*d, rv);
                    }
                    return Ok(());
                }
                "set" => {
                    if args.len() != 2 { return Err("ArrayBox.set expects 2 arg".to_string()); }
                    let idx_v = *vmap.get(&args[0]).ok_or("array.set index missing")?;
                    let val_v = *vmap.get(&args[1]).ok_or("array.set value missing")?;
                    let idx_i = if let BVE::IntValue(iv) = idx_v { iv } else { return Err("array.set index must be int".to_string()); };
                    let val_i = if let BVE::IntValue(iv) = val_v { iv } else { return Err("array.set value must be int".to_string()); };
                    let fnty = i64t.fn_type(&[i64t.into(), i64t.into(), i64t.into()], false);
                    let callee = codegen.module.get_function("nyash_array_set_h").unwrap_or_else(|| codegen.module.add_function("nyash_array_set_h", fnty, None));
                    let _ = codegen.builder.build_call(callee, &[recv_h.into(), idx_i.into(), val_i.into()], "aset").map_err(|e| e.to_string())?;
                    return Ok(());
                }
                "push" => {
                    if args.len() != 1 { return Err("ArrayBox.push expects 1 arg".to_string()); }
                    let val_v = *vmap.get(&args[0]).ok_or("array.push value missing")?;
                    let val_i = match val_v {
                        BVE::IntValue(iv) => iv,
                        BVE::PointerValue(pv) => codegen.builder.build_ptr_to_int(pv, i64t, "val_p2i").map_err(|e| e.to_string())?,
                        _ => return Err("array.push value must be int or handle ptr".to_string()),
                    };
                    let fnty = i64t.fn_type(&[i64t.into(), i64t.into()], false);
                    let callee = codegen.module.get_function("nyash_array_push_h").unwrap_or_else(|| codegen.module.add_function("nyash_array_push_h", fnty, None));
                    let _ = codegen.builder.build_call(callee, &[recv_h.into(), val_i.into()], "apush").map_err(|e| e.to_string())?;
                    return Ok(());
                }
                "length" => {
                    if !args.is_empty() { return Err("ArrayBox.length expects 0 arg".to_string()); }
                    let fnty = i64t.fn_type(&[i64t.into()], false);
                    let callee = codegen.module.get_function("nyash_array_length_h").unwrap_or_else(|| codegen.module.add_function("nyash_array_length_h", fnty, None));
                    let call = codegen.builder.build_call(callee, &[recv_h.into()], "alen").map_err(|e| e.to_string())?;
                    if let Some(d) = dst {
                        let rv = call.try_as_basic_value().left().ok_or("array_length_h returned void".to_string())?;
                        vmap.insert(*d, rv);
                    }
                    return Ok(());
                }
                _ => {}
            }
        }
    }

    // Map fast-paths (core-first): get/set/has/size using NyRT shims with handle receiver
    if let Some(crate::mir::MirType::Box(bname)) = func.metadata.value_types.get(box_val) {
        if bname == "MapBox" && (method == "get" || method == "set" || method == "has" || method == "size") {
            let i64t = codegen.context.i64_type();
            match method {
                "size" => {
                    if !args.is_empty() { return Err("MapBox.size expects 0 arg".to_string()); }
                    let fnty = i64t.fn_type(&[i64t.into()], false);
                    let callee = codegen.module.get_function("nyash.map.size_h").unwrap_or_else(|| codegen.module.add_function("nyash.map.size_h", fnty, None));
                    let call = codegen.builder.build_call(callee, &[recv_h.into()], "msize").map_err(|e| e.to_string())?;
                    if let Some(d) = dst {
                        let rv = call.try_as_basic_value().left().ok_or("map.size_h returned void".to_string())?;
                        vmap.insert(*d, rv);
                    }
                    return Ok(());
                }
                "has" => {
                    if args.len() != 1 { return Err("MapBox.has expects 1 arg".to_string()); }
                    let key_v = *vmap.get(&args[0]).ok_or("map.has key missing")?;
                    let key_i = match key_v {
                        BVE::IntValue(iv) => iv,
                        BVE::PointerValue(pv) => codegen.builder.build_ptr_to_int(pv, i64t, "key_p2i").map_err(|e| e.to_string())?,
                        _ => return Err("map.has key must be int or handle ptr".to_string()),
                    };
                    let fnty = i64t.fn_type(&[i64t.into(), i64t.into()], false);
                    let callee = codegen.module.get_function("nyash.map.has_h").unwrap_or_else(|| codegen.module.add_function("nyash.map.has_h", fnty, None));
                    let call = codegen.builder.build_call(callee, &[recv_h.into(), key_i.into()], "mhas").map_err(|e| e.to_string())?;
                    if let Some(d) = dst {
                        let rv = call.try_as_basic_value().left().ok_or("map.has_h returned void".to_string())?;
                        vmap.insert(*d, rv);
                    }
                    return Ok(());
                }
                "get" => {
                    if args.len() != 1 { return Err("MapBox.get expects 1 arg".to_string()); }
                    let key_v = *vmap.get(&args[0]).ok_or("map.get key missing")?;
                    // prefer integer key; if pointer, convert to handle and call get_hh
                    let call = match key_v {
                        BVE::IntValue(iv) => {
                            let fnty = i64t.fn_type(&[i64t.into(), i64t.into()], false);
                            let callee = codegen.module.get_function("nyash.map.get_h").unwrap_or_else(|| codegen.module.add_function("nyash.map.get_h", fnty, None));
                            codegen.builder.build_call(callee, &[recv_h.into(), iv.into()], "mget").map_err(|e| e.to_string())?
                        }
                        BVE::PointerValue(pv) => {
                            // key: i8* -> i64 handle via from_i8_string (string key)
                            let fnty_conv = i64t.fn_type(&[codegen.context.ptr_type(AddressSpace::from(0)).into()], false);
                            let conv = codegen.module.get_function("nyash.box.from_i8_string").unwrap_or_else(|| codegen.module.add_function("nyash.box.from_i8_string", fnty_conv, None));
                            let kcall = codegen.builder.build_call(conv, &[pv.into()], "key_i8_to_handle").map_err(|e| e.to_string())?;
                            let kh = kcall.try_as_basic_value().left().ok_or("from_i8_string returned void".to_string())?.into_int_value();
                            let fnty = i64t.fn_type(&[i64t.into(), i64t.into()], false);
                            let callee = codegen.module.get_function("nyash.map.get_hh").unwrap_or_else(|| codegen.module.add_function("nyash.map.get_hh", fnty, None));
                            codegen.builder.build_call(callee, &[recv_h.into(), kh.into()], "mget_hh").map_err(|e| e.to_string())?
                        }
                        _ => return Err("map.get key must be int or pointer".to_string()),
                    };
                    if let Some(d) = dst {
                        let rv = call.try_as_basic_value().left().ok_or("map.get returned void".to_string())?;
                        vmap.insert(*d, rv);
                    }
                    return Ok(());
                }
                "set" => {
                    if args.len() != 2 { return Err("MapBox.set expects 2 args (key, value)".to_string()); }
                    let key_v = *vmap.get(&args[0]).ok_or("map.set key missing")?;
                    let val_v = *vmap.get(&args[1]).ok_or("map.set value missing")?;
                    let key_i = match key_v {
                        BVE::IntValue(iv) => iv,
                        BVE::PointerValue(pv) => codegen.builder.build_ptr_to_int(pv, i64t, "key_p2i").map_err(|e| e.to_string())?,
                        _ => return Err("map.set key must be int or handle ptr".to_string()),
                    };
                    let val_i = match val_v {
                        BVE::IntValue(iv) => iv,
                        BVE::PointerValue(pv) => codegen.builder.build_ptr_to_int(pv, i64t, "val_p2i").map_err(|e| e.to_string())?,
                        _ => return Err("map.set value must be int or handle ptr".to_string()),
                    };
                    let fnty = i64t.fn_type(&[i64t.into(), i64t.into(), i64t.into()], false);
                    let callee = codegen.module.get_function("nyash.map.set_h").unwrap_or_else(|| codegen.module.add_function("nyash.map.set_h", fnty, None));
                    let _ = codegen.builder.build_call(callee, &[recv_h.into(), key_i.into(), val_i.into()], "mset").map_err(|e| e.to_string())?;
                    return Ok(());
                }
                _ => {}
            }
        }
    }

    // getField
    if method == "getField" {
        if args.len() != 1 { return Err("getField expects 1 arg (name)".to_string()); }
        let name_v = *vmap.get(&args[0]).ok_or("getField name missing")?;
        let name_p = if let BVE::PointerValue(pv) = name_v { pv } else { return Err("getField name must be pointer".to_string()); };
        let i8p = codegen.context.ptr_type(AddressSpace::from(0));
        let fnty = i64t.fn_type(&[i64t.into(), i8p.into()], false);
        let callee = codegen.module.get_function("nyash.instance.get_field_h").unwrap_or_else(|| codegen.module.add_function("nyash.instance.get_field_h", fnty, None));
        let call = codegen.builder.build_call(callee, &[recv_h.into(), name_p.into()], "getField").map_err(|e| e.to_string())?;
        if let Some(d) = dst {
            let rv = call.try_as_basic_value().left().ok_or("get_field returned void".to_string())?;
            let h = if let BVE::IntValue(iv) = rv { iv } else { return Err("get_field ret expected i64".to_string()); };
            let pty = codegen.context.ptr_type(AddressSpace::from(0));
            let ptr = codegen.builder.build_int_to_ptr(h, pty, "gf_handle_to_ptr").map_err(|e| e.to_string())?;
            vmap.insert(*d, ptr.into());
        }
        // continue for potential further lowering
    }
    if method == "setField" {
        if args.len() != 2 { return Err("setField expects 2 args (name, value)".to_string()); }
        let name_v = *vmap.get(&args[0]).ok_or("setField name missing")?;
        let val_v = *vmap.get(&args[1]).ok_or("setField value missing")?;
        let name_p = if let BVE::PointerValue(pv) = name_v { pv } else { return Err("setField name must be pointer".to_string()); };
        let val_h = match val_v {
            BVE::PointerValue(pv) => codegen.builder.build_ptr_to_int(pv, i64t, "valp2i").map_err(|e| e.to_string())?,
            BVE::IntValue(iv) => iv,
            BVE::FloatValue(_) => return Err("setField value must be int/handle".to_string()),
            _ => return Err("setField value must be int/handle".to_string()),
        };
        let i8p = codegen.context.ptr_type(AddressSpace::from(0));
        let fnty = i64t.fn_type(&[i64t.into(), i8p.into(), i64t.into()], false);
        let callee = codegen.module.get_function("nyash.instance.set_field_h").unwrap_or_else(|| codegen.module.add_function("nyash.instance.set_field_h", fnty, None));
        let _ = codegen.builder.build_call(callee, &[recv_h.into(), name_p.into(), val_h.into()], "setField").map_err(|e| e.to_string())?;
        // continue for method_id path if any
    }

    if let Some(mid) = method_id {
        // Build argc and a1..a4
        let argc_val = i64t.const_int(args.len() as u64, false);
        let mut a1 = i64t.const_zero();
        let mut a2 = i64t.const_zero();
        let mut a3 = i64t.const_zero();
        let mut a4 = i64t.const_zero();
        let get_i64 = |vid: ValueId| -> Result<inkwell::values::IntValue<'ctx>, String> {
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
                    let rv = call.try_as_basic_value().left().ok_or("from_f64 returned void".to_string())?;
                    if let BVE::IntValue(h) = rv { Ok(h) } else { Err("from_f64 ret expected i64".to_string()) }
                }
                _ => Err("unsupported arg value".to_string()),
            }
        };
        if args.len() >= 1 { a1 = get_i64(args[0])?; }
        if args.len() >= 2 { a2 = get_i64(args[1])?; }
        if args.len() >= 3 { a3 = get_i64(args[2])?; }
        if args.len() >= 4 { a4 = get_i64(args[3])?; }

        // Note: String.concat should return a handle (i64) and be processed by the tagged path below.
        // tagged fixed-arity <=4
        let mut tag1 = i64t.const_int(3, false);
        let mut tag2 = i64t.const_int(3, false);
        let mut tag3 = i64t.const_int(3, false);
        let mut tag4 = i64t.const_int(3, false);
        let classify = |vid: ValueId| -> Option<i64> { vmap.get(&vid).map(|v| classify_tag(*v)) };
        if args.len() >= 1 { if let Some(t) = classify(args[0]) { tag1 = i64t.const_int(t as u64, false); } }
        if args.len() >= 2 { if let Some(t) = classify(args[1]) { tag2 = i64t.const_int(t as u64, false); } }
        if args.len() >= 3 { if let Some(t) = classify(args[2]) { tag3 = i64t.const_int(t as u64, false); } }
        if args.len() >= 4 { if let Some(t) = classify(args[3]) { tag4 = i64t.const_int(t as u64, false); } }
        if args.len() <= 4 {
            let fnty = i64t.fn_type(&[i64t.into(), i64t.into(), i64t.into(), i64t.into(), i64t.into(), i64t.into(), i64t.into(), i64t.into(), i64t.into(), i64t.into(), i64t.into(), i64t.into()], false);
            let callee = codegen.module.get_function("nyash_plugin_invoke3_tagged_i64").unwrap_or_else(|| codegen.module.add_function("nyash_plugin_invoke3_tagged_i64", fnty, None));
            let tid = i64t.const_int(type_id as u64, true);
            let midv = i64t.const_int((*mid) as u64, false);
            let call = codegen.builder.build_call(callee, &[tid.into(), midv.into(), argc_val.into(), recv_h.into(), a1.into(), tag1.into(), a2.into(), tag2.into(), a3.into(), tag3.into(), a4.into(), tag4.into()], "pinvoke_tagged").map_err(|e| e.to_string())?;
            if let Some(d) = dst {
                let rv = call.try_as_basic_value().left().ok_or("invoke3_i64 returned void".to_string())?;
                if let Some(mt) = func.metadata.value_types.get(d) {
                    match mt {
                        crate::mir::MirType::Integer | crate::mir::MirType::Bool => { vmap.insert(*d, rv); }
                        // String: keep as i64 handle (do not cast to i8*)
                        crate::mir::MirType::String => { vmap.insert(*d, rv); }
                        // Box/Array/Future/Unknown: cast handle to opaque pointer
                        crate::mir::MirType::Box(_) | crate::mir::MirType::Array(_) | crate::mir::MirType::Future(_) | crate::mir::MirType::Unknown => {
                            let h = if let BVE::IntValue(iv) = rv { iv } else { return Err("invoke ret expected i64".to_string()); };
                            let pty = codegen.context.ptr_type(AddressSpace::from(0));
                            let ptr = codegen.builder.build_int_to_ptr(h, pty, "ret_handle_to_ptr").map_err(|e| e.to_string())?;
                            vmap.insert(*d, ptr.into());
                        }
                        _ => { vmap.insert(*d, rv); }
                    }
                } else {
                    vmap.insert(*d, rv);
                }
            }
            return Ok(());
        }
        // variable length path
        let n = args.len() as u32;
        let arr_ty = i64t.array_type(n);
        let vals_arr = entry_builder.build_alloca(arr_ty, "vals_arr").map_err(|e| e.to_string())?;
        let tags_arr = entry_builder.build_alloca(arr_ty, "tags_arr").map_err(|e| e.to_string())?;
        for (i, vid) in args.iter().enumerate() {
            let idx = [codegen.context.i32_type().const_zero(), codegen.context.i32_type().const_int(i as u64, false)];
            let gep_v = unsafe { codegen.builder.build_in_bounds_gep(arr_ty, vals_arr, &idx, &format!("v_gep_{}", i)).map_err(|e| e.to_string())? };
            let gep_t = unsafe { codegen.builder.build_in_bounds_gep(arr_ty, tags_arr, &idx, &format!("t_gep_{}", i)).map_err(|e| e.to_string())? };
            let vi = get_i64(*vid)?;
            let ti = classify_tag(*vmap.get(vid).ok_or("missing arg")?);
            codegen.builder.build_store(gep_v, vi).map_err(|e| e.to_string())?;
            codegen.builder.build_store(gep_t, i64t.const_int(ti as u64, false)).map_err(|e| e.to_string())?;
        }
        let vals_ptr = codegen.builder.build_pointer_cast(vals_arr, codegen.context.ptr_type(AddressSpace::from(0)), "vals_arr_i8p").map_err(|e| e.to_string())?;
        let tags_ptr = codegen.builder.build_pointer_cast(tags_arr, codegen.context.ptr_type(AddressSpace::from(0)), "tags_arr_i8p").map_err(|e| e.to_string())?;
        let fnty = i64t.fn_type(&[i64t.into(), i64t.into(), i64t.into(), i64t.into(), codegen.context.ptr_type(AddressSpace::from(0)).into(), codegen.context.ptr_type(AddressSpace::from(0)).into()], false);
        let callee = codegen.module.get_function("nyash.plugin.invoke_tagged_v_i64").unwrap_or_else(|| codegen.module.add_function("nyash.plugin.invoke_tagged_v_i64", fnty, None));
        let tid = i64t.const_int(type_id as u64, true);
        let midv = i64t.const_int((*mid) as u64, false);
        let call = codegen.builder.build_call(callee, &[tid.into(), midv.into(), argc_val.into(), recv_h.into(), vals_ptr.into(), tags_ptr.into()], "pinvoke_tagged_v").map_err(|e| e.to_string())?;
        if let Some(d) = dst {
            let rv = call.try_as_basic_value().left().ok_or("invoke_v returned void".to_string())?;
            if let Some(mt) = func.metadata.value_types.get(d) {
                match mt {
                    crate::mir::MirType::Integer | crate::mir::MirType::Bool => { vmap.insert(*d, rv); }
                    crate::mir::MirType::String => { vmap.insert(*d, rv); }
                    crate::mir::MirType::Box(_) | crate::mir::MirType::Array(_) | crate::mir::MirType::Future(_) | crate::mir::MirType::Unknown => {
                        let h = if let BVE::IntValue(iv) = rv { iv } else { return Err("invoke ret expected i64".to_string()); };
                        let pty = codegen.context.ptr_type(AddressSpace::from(0));
                        let ptr = codegen.builder.build_int_to_ptr(h, pty, "ret_handle_to_ptr").map_err(|e| e.to_string())?;
                        vmap.insert(*d, ptr.into());
                    }
                    _ => { vmap.insert(*d, rv); }
                }
            } else {
                vmap.insert(*d, rv);
            }
        }
        return Ok(());
    } else {
        Err(format!("BoxCall requires method_id for method '{}'. The method_id should be automatically injected during MIR compilation.", method))
    }
}
