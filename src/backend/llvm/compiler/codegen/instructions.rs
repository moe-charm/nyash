use inkwell::basic_block::BasicBlock;
use inkwell::values::{BasicValueEnum, FunctionValue, PhiValue};
use std::collections::HashMap;

use crate::mir::{function::MirFunction, BasicBlockId};

use crate::backend::llvm::context::CodegenContext;
use crate::mir::instruction::{ConstValue, MirInstruction};
use crate::mir::{CompareOp, ValueId};

use super::types::{to_bool, to_i64_any};
use inkwell::AddressSpace;

// Small, safe extraction: create LLVM basic blocks for a MIR function and
// return the block map together with the entry block.
pub(super) fn create_basic_blocks<'ctx>(
    codegen: &CodegenContext<'ctx>,
    llvm_func: FunctionValue<'ctx>,
    func: &MirFunction,
) -> (HashMap<BasicBlockId, BasicBlock<'ctx>>, BasicBlock<'ctx>) {
    let mut bb_map: HashMap<BasicBlockId, BasicBlock> = HashMap::new();
    let entry_first = func.entry_block;
    let entry_bb = codegen
        .context
        .append_basic_block(llvm_func, &format!("bb{}", entry_first.as_u32()));
    bb_map.insert(entry_first, entry_bb);
    for bid in func.block_ids() {
        if bid == entry_first {
            continue;
        }
        let name = format!("bb{}", bid.as_u32());
        let bb = codegen.context.append_basic_block(llvm_func, &name);
        bb_map.insert(bid, bb);
    }
    (bb_map, entry_bb)
}

// Pre-create PHI nodes for all blocks; also inserts placeholder values into vmap.
pub(super) fn precreate_phis<'ctx>(
    codegen: &CodegenContext<'ctx>,
    func: &MirFunction,
    bb_map: &HashMap<BasicBlockId, BasicBlock<'ctx>>,
    vmap: &mut HashMap<ValueId, BasicValueEnum<'ctx>>,
) -> Result<
    HashMap<
        BasicBlockId,
        Vec<(ValueId, PhiValue<'ctx>, Vec<(BasicBlockId, ValueId)>)>,
    >,
    String,
> {
    use crate::mir::instruction::MirInstruction;
    use super::types::map_mirtype_to_basic;
    let mut phis_by_block: HashMap<
        BasicBlockId,
        Vec<(ValueId, PhiValue<'ctx>, Vec<(BasicBlockId, ValueId)>)>,
    > = HashMap::new();
    for bid in func.block_ids() {
        let bb = *bb_map.get(&bid).ok_or("missing bb in map")?;
        codegen.builder.position_at_end(bb);
        let block = func.blocks.get(&bid).unwrap();
        for inst in block
            .instructions
            .iter()
            .take_while(|i| matches!(i, MirInstruction::Phi { .. }))
        {
            if let MirInstruction::Phi { dst, inputs } = inst {
                let mut phi_ty: Option<inkwell::types::BasicTypeEnum> = None;
                if let Some(mt) = func.metadata.value_types.get(dst) {
                    phi_ty = Some(map_mirtype_to_basic(codegen.context, mt));
                } else if let Some((_, iv)) = inputs.first() {
                    if let Some(mt) = func.metadata.value_types.get(iv) {
                        phi_ty = Some(map_mirtype_to_basic(codegen.context, mt));
                    }
                }
                let phi_ty = phi_ty.unwrap_or_else(|| codegen.context.i64_type().into());
                let phi = codegen
                    .builder
                    .build_phi(phi_ty, &format!("phi_{}", dst.as_u32()))
                    .map_err(|e| e.to_string())?;
                vmap.insert(*dst, phi.as_basic_value());
                phis_by_block
                    .entry(bid)
                    .or_default()
                    .push((*dst, phi, inputs.clone()));
            }
        }
    }
    Ok(phis_by_block)
}

// Lower Store: handle allocas with element type tracking and integer width adjust
pub(super) fn lower_store<'ctx>(
    codegen: &CodegenContext<'ctx>,
    vmap: &HashMap<ValueId, BasicValueEnum<'ctx>>,
    allocas: &mut HashMap<ValueId, inkwell::values::PointerValue<'ctx>>,
    alloca_elem_types: &mut HashMap<ValueId, inkwell::types::BasicTypeEnum<'ctx>>,
    value: &ValueId,
    ptr: &ValueId,
) -> Result<(), String> {
    use inkwell::types::BasicTypeEnum;
    let val = *vmap.get(value).ok_or("store value missing")?;
    let elem_ty = match val {
        BasicValueEnum::IntValue(iv) => BasicTypeEnum::IntType(iv.get_type()),
        BasicValueEnum::FloatValue(fv) => BasicTypeEnum::FloatType(fv.get_type()),
        BasicValueEnum::PointerValue(pv) => BasicTypeEnum::PointerType(pv.get_type()),
        _ => return Err("unsupported store value type".to_string()),
    };
    if let Some(existing) = allocas.get(ptr).copied() {
        let existing_elem = *alloca_elem_types.get(ptr).ok_or("alloca elem type missing")?;
        if existing_elem != elem_ty {
            match (val, existing_elem) {
                (BasicValueEnum::IntValue(iv), BasicTypeEnum::IntType(t)) => {
                    let bw_src = iv.get_type().get_bit_width();
                    let bw_dst = t.get_bit_width();
                    if bw_src < bw_dst {
                        let adj = codegen.builder.build_int_z_extend(iv, t, "zext").map_err(|e| e.to_string())?;
                        codegen.builder.build_store(existing, adj).map_err(|e| e.to_string())?;
                    } else if bw_src > bw_dst {
                        let adj = codegen.builder.build_int_truncate(iv, t, "trunc").map_err(|e| e.to_string())?;
                        codegen.builder.build_store(existing, adj).map_err(|e| e.to_string())?;
                    } else {
                        codegen.builder.build_store(existing, iv).map_err(|e| e.to_string())?;
                    }
                }
                (BasicValueEnum::PointerValue(pv), BasicTypeEnum::PointerType(pt)) => {
                    let adj = codegen.builder.build_pointer_cast(pv, pt, "pcast").map_err(|e| e.to_string())?;
                    codegen.builder.build_store(existing, adj).map_err(|e| e.to_string())?;
                }
                (BasicValueEnum::FloatValue(fv), BasicTypeEnum::FloatType(ft)) => {
                    // Only f64 currently expected
                    if fv.get_type() != ft { return Err("float width mismatch in store".to_string()); }
                    codegen.builder.build_store(existing, fv).map_err(|e| e.to_string())?;
                }
                _ => return Err("store type mismatch".to_string()),
            }
        } else {
            codegen.builder.build_store(existing, val).map_err(|e| e.to_string())?;
        }
    } else {
        let slot = codegen
            .builder
            .build_alloca(elem_ty, &format!("slot_{}", ptr.as_u32()))
            .map_err(|e| e.to_string())?;
        codegen.builder.build_store(slot, val).map_err(|e| e.to_string())?;
        allocas.insert(*ptr, slot);
        alloca_elem_types.insert(*ptr, elem_ty);
    }
    Ok(())
}

pub(super) fn lower_load<'ctx>(
    codegen: &CodegenContext<'ctx>,
    vmap: &mut HashMap<ValueId, BasicValueEnum<'ctx>>,
    allocas: &mut HashMap<ValueId, inkwell::values::PointerValue<'ctx>>,
    alloca_elem_types: &mut HashMap<ValueId, inkwell::types::BasicTypeEnum<'ctx>>,
    dst: &ValueId,
    ptr: &ValueId,
) -> Result<(), String> {
    use inkwell::types::BasicTypeEnum;
    let (slot, elem_ty) = if let Some(s) = allocas.get(ptr).copied() {
        let et = *alloca_elem_types.get(ptr).ok_or("alloca elem type missing")?;
        (s, et)
    } else {
        // Default new slot as i64 for uninitialized loads
        let i64t = codegen.context.i64_type();
        let slot = codegen
            .builder
            .build_alloca(i64t, &format!("slot_{}", ptr.as_u32()))
            .map_err(|e| e.to_string())?;
        allocas.insert(*ptr, slot);
        alloca_elem_types.insert(*ptr, i64t.into());
        (slot, i64t.into())
    };
    let lv = codegen
        .builder
        .build_load(elem_ty, slot, &format!("load_{}", dst.as_u32()))
        .map_err(|e| e.to_string())?;
    vmap.insert(*dst, lv);
    Ok(())
}

// Const lowering: produce a BasicValue and store into vmap
pub(super) fn lower_const<'ctx>(
    codegen: &CodegenContext<'ctx>,
    vmap: &mut HashMap<ValueId, BasicValueEnum<'ctx>>,
    dst: ValueId,
    value: &ConstValue,
) -> Result<(), String> {
    let bval = match value {
        ConstValue::Integer(i) => codegen
            .context
            .i64_type()
            .const_int(*i as u64, true)
            .into(),
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
            let len = codegen
                .context
                .i32_type()
                .const_int(s.len() as u64, false);
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
                .build_call(callee, &[gv.as_pointer_value().into(), len.into()], "strnew")
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

// Compare lowering: return the resulting BasicValueEnum (i1)
pub(super) fn lower_compare<'ctx>(
    codegen: &CodegenContext<'ctx>,
    vmap: &HashMap<ValueId, BasicValueEnum<'ctx>>,
    op: &CompareOp,
    lhs: &ValueId,
    rhs: &ValueId,
) -> Result<BasicValueEnum<'ctx>, String> {
    use crate::backend::llvm::compiler::helpers::{as_float, as_int};
    let lv = *vmap.get(lhs).ok_or("lhs missing")?;
    let rv = *vmap.get(rhs).ok_or("rhs missing")?;
    let out = if let (Some(li), Some(ri)) = (as_int(lv), as_int(rv)) {
        use CompareOp as C;
        let pred = match op {
            C::Eq => inkwell::IntPredicate::EQ,
            C::Ne => inkwell::IntPredicate::NE,
            C::Lt => inkwell::IntPredicate::SLT,
            C::Le => inkwell::IntPredicate::SLE,
            C::Gt => inkwell::IntPredicate::SGT,
            C::Ge => inkwell::IntPredicate::SGE,
        };
        codegen
            .builder
            .build_int_compare(pred, li, ri, "icmp")
            .map_err(|e| e.to_string())?
            .into()
    } else if let (Some(lf), Some(rf)) = (as_float(lv), as_float(rv)) {
        use CompareOp as C;
        let pred = match op {
            C::Eq => inkwell::FloatPredicate::OEQ,
            C::Ne => inkwell::FloatPredicate::ONE,
            C::Lt => inkwell::FloatPredicate::OLT,
            C::Le => inkwell::FloatPredicate::OLE,
            C::Gt => inkwell::FloatPredicate::OGT,
            C::Ge => inkwell::FloatPredicate::OGE,
        };
        codegen
            .builder
            .build_float_compare(pred, lf, rf, "fcmp")
            .map_err(|e| e.to_string())?
            .into()
    } else if let (BasicValueEnum::PointerValue(lp), BasicValueEnum::PointerValue(rp)) = (lv, rv) {
        // Support pointer equality/inequality comparisons
        use CompareOp as C;
        match op {
            C::Eq | C::Ne => {
                let i64t = codegen.context.i64_type();
                let li = codegen
                    .builder
                    .build_ptr_to_int(lp, i64t, "pi_l")
                    .map_err(|e| e.to_string())?;
                let ri = codegen
                    .builder
                    .build_ptr_to_int(rp, i64t, "pi_r")
                    .map_err(|e| e.to_string())?;
                let pred = if matches!(op, C::Eq) {
                    inkwell::IntPredicate::EQ
                } else {
                    inkwell::IntPredicate::NE
                };
                codegen
                    .builder
                    .build_int_compare(pred, li, ri, "pcmp")
                    .map_err(|e| e.to_string())?
                    .into()
            }
            _ => return Err("unsupported pointer comparison (only Eq/Ne)".to_string()),
        }
    } else if let (BasicValueEnum::PointerValue(lp), BasicValueEnum::IntValue(ri)) = (lv, rv) {
        use CompareOp as C;
        match op {
            C::Eq | C::Ne => {
                let i64t = codegen.context.i64_type();
                let li = codegen
                    .builder
                    .build_ptr_to_int(lp, i64t, "pi_l")
                    .map_err(|e| e.to_string())?;
                let pred = if matches!(op, C::Eq) {
                    inkwell::IntPredicate::EQ
                } else {
                    inkwell::IntPredicate::NE
                };
                codegen
                    .builder
                    .build_int_compare(pred, li, ri, "pcmpi")
                    .map_err(|e| e.to_string())?
                    .into()
            }
            _ => return Err("unsupported pointer-int comparison (only Eq/Ne)".to_string()),
        }
    } else if let (BasicValueEnum::IntValue(li), BasicValueEnum::PointerValue(rp)) = (lv, rv) {
        use CompareOp as C;
        match op {
            C::Eq | C::Ne => {
                let i64t = codegen.context.i64_type();
                let ri = codegen
                    .builder
                    .build_ptr_to_int(rp, i64t, "pi_r")
                    .map_err(|e| e.to_string())?;
                let pred = if matches!(op, C::Eq) {
                    inkwell::IntPredicate::EQ
                } else {
                    inkwell::IntPredicate::NE
                };
                codegen
                    .builder
                    .build_int_compare(pred, li, ri, "ipcmi")
                    .map_err(|e| e.to_string())?
                    .into()
            }
            _ => return Err("unsupported int-pointer comparison (only Eq/Ne)".to_string()),
        }
    } else {
        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
            let lk = match lv {
                BasicValueEnum::IntValue(_) => "int",
                BasicValueEnum::FloatValue(_) => "float",
                BasicValueEnum::PointerValue(_) => "ptr",
                _ => "other",
            };
            let rk = match rv {
                BasicValueEnum::IntValue(_) => "int",
                BasicValueEnum::FloatValue(_) => "float",
                BasicValueEnum::PointerValue(_) => "ptr",
                _ => "other",
            };
            eprintln!("[LLVM] compare type mismatch: lhs={}, rhs={} (op={:?})", lk, rk, op);
        }
        return Err("compare type mismatch".to_string());
    };
    Ok(out)
}

pub(super) fn emit_return<'ctx>(
    codegen: &CodegenContext<'ctx>,
    func: &MirFunction,
    vmap: &HashMap<ValueId, BasicValueEnum<'ctx>>,
    value: &Option<ValueId>,
) -> Result<(), String> {
    match (&func.signature.return_type, value) {
        (crate::mir::MirType::Void, _) => {
            codegen.builder.build_return(None).unwrap();
            Ok(())
        }
        (_t, Some(vid)) => {
            let v = *vmap.get(vid).ok_or("ret value missing")?;
            codegen
                .builder
                .build_return(Some(&v))
                .map_err(|e| e.to_string())?;
            Ok(())
        }
        (_t, None) => Err("non-void function missing return value".to_string()),
    }
}

pub(super) fn emit_jump<'ctx>(
    codegen: &CodegenContext<'ctx>,
    bid: BasicBlockId,
    target: &BasicBlockId,
    bb_map: &HashMap<BasicBlockId, BasicBlock<'ctx>>,
    phis_by_block: &HashMap<
        BasicBlockId,
        Vec<(ValueId, PhiValue<'ctx>, Vec<(BasicBlockId, ValueId)>)>,
    >,
    vmap: &HashMap<ValueId, BasicValueEnum<'ctx>>,
) -> Result<(), String> {
    if let Some(list) = phis_by_block.get(target) {
        for (_dst, phi, inputs) in list {
            if let Some((_, in_vid)) = inputs.iter().find(|(pred, _)| pred == &bid) {
                let val = *vmap.get(in_vid).ok_or("phi incoming value missing")?;
                let pred_bb = *bb_map.get(&bid).ok_or("pred bb missing")?;
                match val {
                    BasicValueEnum::IntValue(iv) => phi.add_incoming(&[(&iv, pred_bb)]),
                    BasicValueEnum::FloatValue(fv) => phi.add_incoming(&[(&fv, pred_bb)]),
                    BasicValueEnum::PointerValue(pv) => phi.add_incoming(&[(&pv, pred_bb)]),
                    _ => return Err("unsupported phi incoming value".to_string()),
                }
            }
        }
    }
    let tbb = *bb_map.get(target).ok_or("target bb missing")?;
    codegen
        .builder
        .build_unconditional_branch(tbb)
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub(super) fn emit_branch<'ctx>(
    codegen: &CodegenContext<'ctx>,
    bid: BasicBlockId,
    condition: &ValueId,
    then_bb: &BasicBlockId,
    else_bb: &BasicBlockId,
    bb_map: &HashMap<BasicBlockId, BasicBlock<'ctx>>,
    phis_by_block: &HashMap<
        BasicBlockId,
        Vec<(ValueId, PhiValue<'ctx>, Vec<(BasicBlockId, ValueId)>)>,
    >,
    vmap: &HashMap<ValueId, BasicValueEnum<'ctx>>,
) -> Result<(), String> {
    let cond_v = *vmap.get(condition).ok_or("cond missing")?;
    let b = to_bool(codegen.context, cond_v, &codegen.builder)?;
    // then
    if let Some(list) = phis_by_block.get(then_bb) {
        for (_dst, phi, inputs) in list {
            if let Some((_, in_vid)) = inputs.iter().find(|(pred, _)| pred == &bid) {
                let val = *vmap
                    .get(in_vid)
                    .ok_or("phi incoming (then) value missing")?;
                let pred_bb = *bb_map.get(&bid).ok_or("pred bb missing")?;
                match val {
                    BasicValueEnum::IntValue(iv) => phi.add_incoming(&[(&iv, pred_bb)]),
                    BasicValueEnum::FloatValue(fv) => phi.add_incoming(&[(&fv, pred_bb)]),
                    BasicValueEnum::PointerValue(pv) => phi.add_incoming(&[(&pv, pred_bb)]),
                    _ => return Err("unsupported phi incoming value (then)".to_string()),
                }
            }
        }
    }
    // else
    if let Some(list) = phis_by_block.get(else_bb) {
        for (_dst, phi, inputs) in list {
            if let Some((_, in_vid)) = inputs.iter().find(|(pred, _)| pred == &bid) {
                let val = *vmap
                    .get(in_vid)
                    .ok_or("phi incoming (else) value missing")?;
                let pred_bb = *bb_map.get(&bid).ok_or("pred bb missing")?;
                match val {
                    BasicValueEnum::IntValue(iv) => phi.add_incoming(&[(&iv, pred_bb)]),
                    BasicValueEnum::FloatValue(fv) => phi.add_incoming(&[(&fv, pred_bb)]),
                    BasicValueEnum::PointerValue(pv) => phi.add_incoming(&[(&pv, pred_bb)]),
                    _ => return Err("unsupported phi incoming value (else)".to_string()),
                }
            }
        }
    }
    let tbb = *bb_map.get(then_bb).ok_or("then bb missing")?;
    let ebb = *bb_map.get(else_bb).ok_or("else bb missing")?;
    codegen
        .builder
        .build_conditional_branch(b, tbb, ebb)
        .map_err(|e| e.to_string())?;
    Ok(())
}

// Full ExternCall lowering (console/debug, future.spawn_instance, env.local, env.box.new)
pub(super) fn lower_externcall<'ctx>(
    codegen: &CodegenContext<'ctx>,
    func: &MirFunction,
    vmap: &mut HashMap<ValueId, BasicValueEnum<'ctx>>,
    dst: &Option<ValueId>,
    iface_name: &str,
    method_name: &str,
    args: &[ValueId],
) -> Result<(), String> {
    use inkwell::values::PointerValue;
    use super::super::helpers::{as_float, as_int};
    use inkwell::values::BasicValueEnum as BVE;

    if (iface_name == "env.console"
        && (method_name == "log" || method_name == "warn" || method_name == "error"))
        || (iface_name == "env.debug" && method_name == "trace")
    {
        if args.len() != 1 {
            return Err(format!("{}.{} expects 1 arg (handle)", iface_name, method_name));
        }
        let av = *vmap.get(&args[0]).ok_or("extern arg missing")?;
        let arg_val = match av {
            BVE::IntValue(iv) => {
                if iv.get_type() == codegen.context.bool_type() {
                    codegen
                        .builder
                        .build_int_z_extend(iv, codegen.context.i64_type(), "bool2i64")
                        .map_err(|e| e.to_string())?
                } else if iv.get_type() == codegen.context.i64_type() {
                    iv
                } else {
                    codegen
                        .builder
                        .build_int_s_extend(iv, codegen.context.i64_type(), "int2i64")
                        .map_err(|e| e.to_string())?
                }
            }
            BVE::PointerValue(pv) => codegen
                .builder
                .build_ptr_to_int(pv, codegen.context.i64_type(), "p2i")
                .map_err(|e| e.to_string())?,
            _ => return Err("console.log arg conversion failed".to_string()),
        };
        let fnty = codegen
            .context
            .i64_type()
            .fn_type(&[codegen.context.i64_type().into()], false);
        let fname = if iface_name == "env.console" {
            match method_name {
                "log" => "nyash.console.log_handle",
                "warn" => "nyash.console.warn_handle",
                _ => "nyash.console.error_handle",
            }
        } else {
            "nyash.debug.trace_handle"
        };
        let callee = codegen
            .module
            .get_function(fname)
            .unwrap_or_else(|| codegen.module.add_function(fname, fnty, None));
        let _ = codegen
            .builder
            .build_call(callee, &[arg_val.into()], "console_log_h")
            .map_err(|e| e.to_string())?;
        if let Some(d) = dst {
            vmap.insert(*d, codegen.context.i64_type().const_zero().into());
        }
        return Ok(());
    }

    if iface_name == "env.console" && method_name == "readLine" {
        if !args.is_empty() {
            return Err("console.readLine expects 0 args".to_string());
        }
        let i8p = codegen.context.ptr_type(AddressSpace::from(0));
        let fnty = i8p.fn_type(&[], false);
        let callee = codegen
            .module
            .get_function("nyash.console.readline")
            .unwrap_or_else(|| codegen.module.add_function("nyash.console.readline", fnty, None));
        let call = codegen
            .builder
            .build_call(callee, &[], "readline")
            .map_err(|e| e.to_string())?;
        if let Some(d) = dst {
            let rv = call
                .try_as_basic_value()
                .left()
                .ok_or("readline returned void".to_string())?;
            vmap.insert(*d, rv);
        }
        return Ok(());
    }

    if iface_name == "env.future" && method_name == "spawn_instance" {
        if args.len() < 2 {
            return Err("env.future.spawn_instance expects at least (recv, method_name)".to_string());
        }
        let i64t = codegen.context.i64_type();
        let i8p = codegen.context.ptr_type(AddressSpace::from(0));
        let a0_v = *vmap.get(&args[0]).ok_or("recv missing")?;
        let a0 = to_i64_any(codegen.context, &codegen.builder, a0_v)?;
        let a1_v = *vmap.get(&args[1]).ok_or("method_name missing")?;
        let a1 = match a1_v {
            BVE::PointerValue(pv) => codegen
                .builder
                .build_ptr_to_int(pv, i64t, "mname_p2i")
                .map_err(|e| e.to_string())?,
            _ => to_i64_any(codegen.context, &codegen.builder, a1_v)?,
        };
        let a2 = if args.len() >= 3 {
            let v = *vmap.get(&args[2]).ok_or("arg2 missing")?;
            to_i64_any(codegen.context, &codegen.builder, v)?
        } else {
            i64t.const_zero()
        };
        let argc_total = i64t.const_int(args.len().saturating_sub(1) as u64, false);
        let fnty = i64t.fn_type(&[i64t.into(), i64t.into(), i64t.into(), i64t.into()], false);
        let callee = codegen
            .module
            .get_function("nyash.future.spawn_instance3_i64")
            .unwrap_or_else(|| {
                codegen
                    .module
                    .add_function("nyash.future.spawn_instance3_i64", fnty, None)
            });
        let call = codegen
            .builder
            .build_call(callee, &[a0.into(), a1.into(), a2.into(), argc_total.into()], "spawn_i3")
            .map_err(|e| e.to_string())?;
        if let Some(d) = dst {
            let rv = call
                .try_as_basic_value()
                .left()
                .ok_or("spawn_instance3 returned void".to_string())?;
            if let Some(mt) = func.metadata.value_types.get(d) {
                match mt {
                    crate::mir::MirType::Integer | crate::mir::MirType::Bool => {
                        vmap.insert(*d, rv);
                    }
                    crate::mir::MirType::Box(_)
                    | crate::mir::MirType::String
                    | crate::mir::MirType::Array(_)
                    | crate::mir::MirType::Future(_)
                    | crate::mir::MirType::Unknown => {
                        let iv = if let BVE::IntValue(iv) = rv {
                            iv
                        } else {
                            return Err("spawn ret expected i64".to_string());
                        };
                        let pty = codegen.context.ptr_type(AddressSpace::from(0));
                        let ptr = codegen
                            .builder
                            .build_int_to_ptr(iv, pty, "ret_handle_to_ptr")
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
        return Ok(());
    }

    if iface_name == "env.local" && method_name == "get" {
        if let Some(d) = dst {
            let av = *vmap.get(&args[0]).ok_or("extern arg missing")?;
            vmap.insert(*d, av);
        }
        return Ok(());
    }

    if iface_name == "env.local" && method_name == "set" {
        // no-op in AOT
        return Ok(());
    }

    if iface_name == "env.box" && method_name == "new" {
        if args.len() < 1 {
            return Err("env.box.new expects at least 1 arg (type name)".to_string());
        }
        let i8p = codegen.context.ptr_type(AddressSpace::from(0));
        let tyv = *vmap.get(&args[0]).ok_or("type name arg missing")?;
        let ty_ptr = match tyv { BVE::PointerValue(p) => p, _ => return Err("env.box.new type must be i8* string".to_string()) };
        let i64t = codegen.context.i64_type();
        let out_ptr: PointerValue = if args.len() == 1 {
            let fnty = i64t.fn_type(&[i8p.into()], false);
            let callee = codegen
                .module
                .get_function("nyash.env.box.new")
                .unwrap_or_else(|| codegen.module.add_function("nyash.env.box.new", fnty, None));
            let call = codegen
                .builder
                .build_call(callee, &[ty_ptr.into()], "env_box_new")
                .map_err(|e| e.to_string())?;
            let rv = call
                .try_as_basic_value()
                .left()
                .ok_or("env.box.new returned void".to_string())?;
            let i64v = if let BVE::IntValue(iv) = rv { iv } else { return Err("env.box.new ret expected i64".to_string()); };
            codegen
                .builder
                .build_int_to_ptr(i64v, i8p, "box_handle_to_ptr")
                .map_err(|e| e.to_string())?
        } else {
            if args.len() - 1 > 4 {
                return Err("env.box.new supports up to 4 args in AOT shim".to_string());
            }
            let fnty = i64t.fn_type(&[i8p.into(), i64t.into(), i64t.into(), i64t.into(), i64t.into(), i64t.into()], false);
            let callee = codegen
                .module
                .get_function("nyash.env.box.new_i64x")
                .unwrap_or_else(|| codegen.module.add_function("nyash.env.box.new_i64x", fnty, None));
            let argc_val = i64t.const_int((args.len() - 1) as u64, false);
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
            codegen
                .builder
                .build_int_to_ptr(i64v, i8p, "box_handle_to_ptr")
                .map_err(|e| e.to_string())?
        };
        if let Some(d) = dst {
            vmap.insert(*d, out_ptr.into());
        }
        return Ok(());
    }

    Err(format!(
        "ExternCall lowering unsupported: {}.{} (add a NyRT shim for this interface method)",
        iface_name, method_name
    ))
}

// NewBox lowering (subset consistent with existing code)
pub(super) fn lower_newbox<'ctx>(
    codegen: &CodegenContext<'ctx>,
    vmap: &mut HashMap<ValueId, BasicValueEnum<'ctx>>,
    dst: ValueId,
    box_type: &str,
    args: &[ValueId],
    box_type_ids: &HashMap<String, i64>,
) -> Result<(), String> {
    use inkwell::values::BasicValueEnum as BVE;
    match (box_type, args.len()) {
        ("StringBox", 1) => {
            let av = *vmap.get(&args[0]).ok_or("StringBox arg missing")?;
            vmap.insert(dst, av);
            Ok(())
        }
        ("IntegerBox", 1) => {
            let av = *vmap.get(&args[0]).ok_or("IntegerBox arg missing")?;
            vmap.insert(dst, av);
            Ok(())
        }
        (_, n) if n == 1 || n == 2 => {
            let type_id = *box_type_ids.get(box_type).unwrap_or(&0);
            let i64t = codegen.context.i64_type();
            let fnty = i64t.fn_type(&[i64t.into(), i64t.into(), i64t.into(), i64t.into()], false);
            let callee = codegen
                .module
                .get_function("nyash.box.birth_i64")
                .unwrap_or_else(|| codegen.module.add_function("nyash.box.birth_i64", fnty, None));
            let argc = i64t.const_int(args.len() as u64, false);
            let mut a1 = i64t.const_zero();
            let mut a2 = i64t.const_zero();
            if args.len() >= 1 {
                let v = *vmap.get(&args[0]).ok_or("newbox arg[0] missing")?;
                a1 = match v {
                    BVE::IntValue(iv) => iv,
                    BVE::PointerValue(pv) => codegen
                        .builder
                        .build_ptr_to_int(pv, i64t, "arg0_p2i")
                        .map_err(|e| e.to_string())?,
                    _ => {
                        return Err(
                            "newbox arg[0]: unsupported type (expect int or handle ptr)"
                                .to_string(),
                        )
                    }
                };
            }
            if args.len() >= 2 {
                let v = *vmap.get(&args[1]).ok_or("newbox arg[1] missing")?;
                a2 = match v {
                    BVE::IntValue(iv) => iv,
                    BVE::PointerValue(pv) => codegen
                        .builder
                        .build_ptr_to_int(pv, i64t, "arg1_p2i")
                        .map_err(|e| e.to_string())?,
                    _ => {
                        return Err(
                            "newbox arg[1]: unsupported type (expect int or handle ptr)"
                                .to_string(),
                        )
                    }
                };
            }
            let tid = i64t.const_int(type_id as u64, true);
            let call = codegen
                .builder
                .build_call(callee, &[tid.into(), argc.into(), a1.into(), a2.into()], "birth_i64")
                .map_err(|e| e.to_string())?;
            let h = call
                .try_as_basic_value()
                .left()
                .ok_or("birth_i64 returned void".to_string())?
                .into_int_value();
            let pty = codegen.context.ptr_type(AddressSpace::from(0));
            let ptr = codegen
                .builder
                .build_int_to_ptr(h, pty, "handle_to_ptr")
                .map_err(|e| e.to_string())?;
            vmap.insert(dst, ptr.into());
            Ok(())
        }
        _ => {
            if !args.is_empty() {
                return Err("NewBox with >2 args not yet supported in LLVM lowering".to_string());
            }
            let type_id = *box_type_ids.get(box_type).unwrap_or(&0);
            let i64t = codegen.context.i64_type();
            let fn_ty = i64t.fn_type(&[i64t.into()], false);
            let callee = codegen
                .module
                .get_function("nyash.box.birth_h")
                .unwrap_or_else(|| codegen.module.add_function("nyash.box.birth_h", fn_ty, None));
            let tid = i64t.const_int(type_id as u64, true);
            let call = codegen
                .builder
                .build_call(callee, &[tid.into()], "birth")
                .map_err(|e| e.to_string())?;
            let h_i64 = call
                .try_as_basic_value()
                .left()
                .ok_or("birth_h returned void".to_string())?
                .into_int_value();
            let pty = codegen.context.ptr_type(AddressSpace::from(0));
            let ptr = codegen
                .builder
                .build_int_to_ptr(h_i64, pty, "handle_to_ptr")
                .map_err(|e| e.to_string())?;
            vmap.insert(dst, ptr.into());
            Ok(())
        }
    }
}

// BoxCall lowering (large): mirrors existing logic; kept in one function for now
pub(super) fn lower_boxcall<'ctx>(
    codegen: &CodegenContext<'ctx>,
    func: &MirFunction,
    vmap: &mut HashMap<ValueId, BasicValueEnum<'ctx>>,
    dst: &Option<ValueId>,
    box_val: &ValueId,
    method: &str,
    method_id: &Option<u16>,
    args: &[ValueId],
    box_type_ids: &HashMap<String, i64>,
    entry_builder: &inkwell::builder::Builder<'ctx>,
) -> Result<(), String> {
    use super::super::helpers::{as_float, as_int};
    use super::types::classify_tag;
    use inkwell::values::BasicValueEnum as BVE;
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

        // f64 fast-path for concat
        if method == "concat" && args.len() == 1 {
            let fnty = codegen.context.f64_type().fn_type(&[i64t.into(), i64t.into(), i64t.into()], false);
            let callee = codegen.module.get_function("nyash_plugin_invoke3_f64").unwrap_or_else(|| codegen.module.add_function("nyash_plugin_invoke3_f64", fnty, None));
            let tid = i64t.const_int(type_id as u64, true);
            let midv = i64t.const_int((*mid) as u64, false);
            let call = codegen.builder.build_call(callee, &[tid.into(), midv.into(), argc_val.into(), recv_h.into(), a1.into(), a2.into()], "pinvoke_f64").map_err(|e| e.to_string())?;
            if let Some(d) = dst {
                let rv = call.try_as_basic_value().left().ok_or("invoke3_f64 returned void".to_string())?;
                vmap.insert(*d, rv);
            }
            return Ok(());
        }
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
                        crate::mir::MirType::Box(_) | crate::mir::MirType::String | crate::mir::MirType::Array(_) | crate::mir::MirType::Future(_) | crate::mir::MirType::Unknown => {
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
                    crate::mir::MirType::Box(_) | crate::mir::MirType::String | crate::mir::MirType::Array(_) | crate::mir::MirType::Future(_) | crate::mir::MirType::Unknown => {
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
        Ok(())
    } else {
        Err(format!("BoxCall requires method_id for method '{}'. The method_id should be automatically injected during MIR compilation.", method))
    }
}
