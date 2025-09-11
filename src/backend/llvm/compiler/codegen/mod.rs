use super::helpers::{as_float, as_int, map_type};
use super::LLVMCompiler;
use crate::backend::llvm::context::CodegenContext;
use crate::mir::function::MirModule;
use crate::mir::instruction::{ConstValue, MirInstruction, UnaryOp};
use crate::mir::ValueId;
use inkwell::context::Context;
use inkwell::{
    types::{BasicTypeEnum, FloatType, IntType, PointerType},
    values::{BasicValueEnum, FloatValue, FunctionValue, IntValue, PhiValue, PointerValue},
    AddressSpace,
};
use std::collections::HashMap;

// Submodules: helpers for type conversion/classification used by lowering
mod types;
use self::types::{
    classify_tag, cmp_eq_ne_any, i64_to_ptr, map_mirtype_to_basic, to_bool, to_i64_any,
};
mod instructions;

impl LLVMCompiler {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            values: HashMap::new(),
        })
    }

    pub fn compile_module(&self, mir_module: &MirModule, output_path: &str) -> Result<(), String> {
        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
            eprintln!(
                "[LLVM] compile_module start: functions={}, out={}",
                mir_module.functions.len(),
                output_path
            );
        }
        let context = Context::create();
        let codegen = CodegenContext::new(&context, "nyash_module")?;
        // Load box type-id mapping from nyash_box.toml (central plugin registry)
        let box_type_ids = crate::backend::llvm::box_types::load_box_type_ids();

        // Utility: sanitize MIR function name to a valid C symbol
        let sanitize = |name: &str| -> String {
            name.chars()
                .map(|c| match c {
                    '.' | '/' | '-' => '_',
                    other => other,
                })
                .collect()
        };

        // Find entry function
        let (entry_name, _entry_func_ref) = if let Some((n, f)) = mir_module
            .functions
            .iter()
            .find(|(_n, f)| f.metadata.is_entry_point)
        {
            (n.clone(), f)
        } else if let Some(f) = mir_module.functions.get("Main.main") {
            ("Main.main".to_string(), f)
        } else if let Some(f) = mir_module.functions.get("main") {
            ("main".to_string(), f)
        } else if let Some((n, f)) = mir_module.functions.iter().next() {
            (n.clone(), f)
        } else {
            return Err("Main.main function not found in module".to_string());
        };

        // Predeclare all MIR functions as LLVM functions
        let mut llvm_funcs: HashMap<String, FunctionValue> = HashMap::new();
        for (name, f) in &mir_module.functions {
            let ret_bt = match f.signature.return_type {
                crate::mir::MirType::Void => codegen.context.i64_type().into(),
                ref t => map_type(codegen.context, t)?,
            };
            let mut params_bt: Vec<BasicTypeEnum> = Vec::new();
            for pt in &f.signature.params {
                params_bt.push(map_type(codegen.context, pt)?);
            }
            let ll_fn_ty = match ret_bt {
                BasicTypeEnum::IntType(t) => t.fn_type(&params_bt.iter().map(|t| (*t).into()).collect::<Vec<_>>(), false),
                BasicTypeEnum::FloatType(t) => t.fn_type(&params_bt.iter().map(|t| (*t).into()).collect::<Vec<_>>(), false),
                BasicTypeEnum::PointerType(t) => t.fn_type(&params_bt.iter().map(|t| (*t).into()).collect::<Vec<_>>(), false),
                _ => return Err("Unsupported return basic type".to_string()),
            };
            let sym = format!("ny_f_{}", sanitize(name));
            let lf = codegen.module.add_function(&sym, ll_fn_ty, None);
            llvm_funcs.insert(name.clone(), lf);
        }

        // Helper to build a map of ValueId -> const string for each function (to resolve call targets)
        let build_const_str_map = |f: &crate::mir::function::MirFunction| -> HashMap<ValueId, String> {
            let mut m = HashMap::new();
            for bid in f.block_ids() {
                if let Some(b) = f.blocks.get(&bid) {
                    for inst in &b.instructions {
                        if let MirInstruction::Const { dst, value: ConstValue::String(s) } = inst {
                            m.insert(*dst, s.clone());
                        }
                    }
                    if let Some(MirInstruction::Const { dst, value: ConstValue::String(s) }) = &b.terminator {
                        m.insert(*dst, s.clone());
                    }
                }
            }
            m
        };

        // Lower all functions
        for (name, func) in &mir_module.functions {
            let llvm_func = *llvm_funcs.get(name).ok_or("predecl not found")?;
            // Create basic blocks
            let (mut bb_map, entry_bb) = instructions::create_basic_blocks(&codegen, llvm_func, func);
            codegen.builder.position_at_end(entry_bb);
            let mut vmap: HashMap<ValueId, BasicValueEnum> = HashMap::new();
            let mut allocas: HashMap<ValueId, PointerValue> = HashMap::new();
            let entry_builder = codegen.context.create_builder();
            entry_builder.position_at_end(entry_bb);
            let mut alloca_elem_types: HashMap<ValueId, BasicTypeEnum> = HashMap::new();
            let mut phis_by_block: HashMap<
                crate::mir::BasicBlockId,
                Vec<(ValueId, PhiValue, Vec<(crate::mir::BasicBlockId, ValueId)>)>,
            > = HashMap::new();
            // Bind parameters
            for (i, pid) in func.params.iter().enumerate() {
                if let Some(av) = llvm_func.get_nth_param(i as u32) {
                    vmap.insert(*pid, av);
                }
            }
            // Precreate phis
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
                        let mut phi_ty: Option<BasicTypeEnum> = None;
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

            // Map of const strings for Call resolution
            let const_strs = build_const_str_map(func);

            // Lower body
            for bid in func.block_ids() {
                let bb = *bb_map.get(&bid).unwrap();
                if codegen
                    .builder
                    .get_insert_block()
                    .map(|b| b != bb)
                    .unwrap_or(true)
                {
                    codegen.builder.position_at_end(bb);
                }
                let block = func.blocks.get(&bid).unwrap();
                for inst in &block.instructions {
                    match inst {
                        MirInstruction::NewBox { dst, box_type, args } => {
                            instructions::lower_newbox(&codegen, &mut vmap, *dst, box_type, args, &box_type_ids)?;
                        }
                        MirInstruction::Const { dst, value } => {
                            let bval = match value {
                            ConstValue::Integer(i) => {
                                codegen.context.i64_type().const_int(*i as u64, true).into()
                            }
                            ConstValue::Float(f) => {
                                codegen.context.f64_type().const_float(*f).into()
                            }
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
                                let len =
                                    codegen.context.i32_type().const_int(s.len() as u64, false);
                                // declare i8* @nyash_string_new(i8*, i32)
                                let rt = codegen
                                    .context
                                    .ptr_type(inkwell::AddressSpace::from(0));
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
                                    .unwrap_or_else(|| {
                                        codegen.module.add_function("nyash_string_new", fn_ty, None)
                                    });
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
                                .const_zero()
                                .into(),
                            ConstValue::Void => return Err("Const Void unsupported".to_string()),
                        };
                            vmap.insert(*dst, bval);
                        }
                        MirInstruction::Call { dst, func: callee, args, .. } => {
                            // Resolve callee name from const string -> lookup predeclared function
                            let name_s = const_strs.get(callee).ok_or_else(|| format!("call: callee value {} not a const string", callee.as_u32()))?;
                            let sym = format!("ny_f_{}", sanitize(name_s));
                            let target = codegen
                                .module
                                .get_function(&sym)
                                .ok_or_else(|| format!("call: function symbol not found: {}", sym))?;
                            // Collect args
                            let mut avs: Vec<BasicValueEnum> = Vec::new();
                            for a in args {
                                let v = *vmap
                                    .get(a)
                                    .ok_or_else(|| format!("call arg missing: {}", a.as_u32()))?;
                                avs.push(v);
                            }
                            let call = codegen
                                .builder
                                .build_call(target, &avs.iter().map(|v| (*v).into()).collect::<Vec<_>>(), "call")
                                .map_err(|e| e.to_string())?;
                            if let Some(d) = dst {
                                if let Some(rv) = call.try_as_basic_value().left() {
                                    vmap.insert(*d, rv);
                                }
                            }
                        }
                        MirInstruction::BoxCall {
                            dst,
                            box_val,
                            method,
                            method_id,
                            args,
                            effects: _,
                        } => {
                        // Delegate to refactored lowering and skip legacy body
                        instructions::lower_boxcall(
                            &codegen,
                            func,
                            &mut vmap,
                            dst,
                            box_val,
                            method,
                            method_id,
                            args,
                            &box_type_ids,
                            &entry_builder,
                        )?;
                        continue;
                    }
                    MirInstruction::ExternCall { dst, iface_name, method_name, args, effects: _ } => {
                        instructions::lower_externcall(&codegen, func, &mut vmap, dst, iface_name, method_name, args)?;
                    }
                    MirInstruction::UnaryOp { dst, op, operand } => {
                        instructions::lower_unary(&codegen, &mut vmap, *dst, op, operand)?;
                    }
                    MirInstruction::BinOp { dst, op, lhs, rhs } => {
                        // Delegated to refactored lowering; keep legacy body for 0-diff but unreachable.
                        instructions::lower_binop(&codegen, func, &mut vmap, *dst, op, lhs, rhs)?;
                        continue;
                        let lv = *vmap.get(lhs).ok_or("lhs missing")?;
                        let rv = *vmap.get(rhs).ok_or("rhs missing")?;
                        let mut handled_concat = false;
                        // String-like concat handling: if either side is a pointer (i8*),
                        // and op is Add, route to NyRT concat helpers
                        if let crate::mir::BinaryOp::Add = op {
                            let i8p = codegen.context.ptr_type(AddressSpace::from(0));
                            let is_stringish = |vid: &ValueId| -> bool {
                                match func.metadata.value_types.get(vid) {
                                    Some(crate::mir::MirType::String) => true,
                                    Some(crate::mir::MirType::Box(_)) => true,
                                    _ => false,
                                }
                            };
                            match (lv, rv) {
                                (
                                    BasicValueEnum::PointerValue(lp),
                                    BasicValueEnum::PointerValue(rp),
                                ) => {
                                    let fnty = i8p.fn_type(&[i8p.into(), i8p.into()], false);
                                    let callee = codegen
                                        .module
                                        .get_function("nyash.string.concat_ss")
                                        .unwrap_or_else(|| {
                                            codegen.module.add_function(
                                                "nyash.string.concat_ss",
                                                fnty,
                                                None,
                                            )
                                        });
                                    let call = codegen
                                        .builder
                                        .build_call(callee, &[lp.into(), rp.into()], "concat_ss")
                                        .map_err(|e| e.to_string())?;
                                    let rv = call
                                        .try_as_basic_value()
                                        .left()
                                        .ok_or("concat_ss returned void".to_string())?;
                                    vmap.insert(*dst, rv);
                                    handled_concat = true;
                                }
                                (
                                    BasicValueEnum::PointerValue(lp),
                                    BasicValueEnum::IntValue(ri),
                                ) => {
                                    // Minimal fallback: if both sides are annotated String/Box, convert ptr->handle and use concat_hh
                                    if is_stringish(lhs) && is_stringish(rhs) {
                                        let i64t = codegen.context.i64_type();
                                        // from_i8_string: i64(i8*)
                                        let fnty_conv = i64t.fn_type(&[i8p.into()], false);
                                        let conv = codegen
                                            .module
                                            .get_function("nyash.box.from_i8_string")
                                            .unwrap_or_else(|| codegen.module.add_function("nyash.box.from_i8_string", fnty_conv, None));
                                        let call_c = codegen
                                            .builder
                                            .build_call(conv, &[lp.into()], "lhs_i8_to_handle")
                                            .map_err(|e| e.to_string())?;
                                        let lh = call_c
                                            .try_as_basic_value()
                                            .left()
                                            .ok_or("from_i8_string returned void".to_string())?
                                            .into_int_value();
                                        // concat_hh: i64(i64,i64)
                                        let fnty_hh = i64t.fn_type(&[i64t.into(), i64t.into()], false);
                                        let callee = codegen
                                            .module
                                            .get_function("nyash.string.concat_hh")
                                            .unwrap_or_else(|| codegen.module.add_function("nyash.string.concat_hh", fnty_hh, None));
                                        let call = codegen
                                            .builder
                                            .build_call(callee, &[lh.into(), ri.into()], "concat_hh")
                                            .map_err(|e| e.to_string())?;
                                        let rv = call
                                            .try_as_basic_value()
                                            .left()
                                            .ok_or("concat_hh returned void".to_string())?;
                                        vmap.insert(*dst, rv);
                                        handled_concat = true;
                                    } else {
                                        let i64t = codegen.context.i64_type();
                                        let fnty = i8p.fn_type(&[i8p.into(), i64t.into()], false);
                                        let callee = codegen
                                            .module
                                            .get_function("nyash.string.concat_si")
                                            .unwrap_or_else(|| {
                                                codegen.module.add_function(
                                                    "nyash.string.concat_si",
                                                    fnty,
                                                    None,
                                                )
                                            });
                                        let call = codegen
                                            .builder
                                            .build_call(callee, &[lp.into(), ri.into()], "concat_si")
                                            .map_err(|e| e.to_string())?;
                                        let rv = call
                                            .try_as_basic_value()
                                            .left()
                                            .ok_or("concat_si returned void".to_string())?;
                                        vmap.insert(*dst, rv);
                                        handled_concat = true;
                                    }
                                }
                                (
                                    BasicValueEnum::IntValue(li),
                                    BasicValueEnum::PointerValue(rp),
                                ) => {
                                    // Minimal fallback: if both sides are annotated String/Box, convert ptr->handle and use concat_hh
                                    if is_stringish(lhs) && is_stringish(rhs) {
                                        let i64t = codegen.context.i64_type();
                                        let fnty_conv = i64t.fn_type(&[i8p.into()], false);
                                        let conv = codegen
                                            .module
                                            .get_function("nyash.box.from_i8_string")
                                            .unwrap_or_else(|| codegen.module.add_function("nyash.box.from_i8_string", fnty_conv, None));
                                        let call_c = codegen
                                            .builder
                                            .build_call(conv, &[rp.into()], "rhs_i8_to_handle")
                                            .map_err(|e| e.to_string())?;
                                        let rh = call_c
                                            .try_as_basic_value()
                                            .left()
                                            .ok_or("from_i8_string returned void".to_string())?
                                            .into_int_value();
                                        let fnty_hh = i64t.fn_type(&[i64t.into(), i64t.into()], false);
                                        let callee = codegen
                                            .module
                                            .get_function("nyash.string.concat_hh")
                                            .unwrap_or_else(|| codegen.module.add_function("nyash.string.concat_hh", fnty_hh, None));
                                        let call = codegen
                                            .builder
                                            .build_call(callee, &[li.into(), rh.into()], "concat_hh")
                                            .map_err(|e| e.to_string())?;
                                        let rv = call
                                            .try_as_basic_value()
                                            .left()
                                            .ok_or("concat_hh returned void".to_string())?;
                                        vmap.insert(*dst, rv);
                                        handled_concat = true;
                                    } else {
                                        let i64t = codegen.context.i64_type();
                                        let fnty = i8p.fn_type(&[i64t.into(), i8p.into()], false);
                                        let callee = codegen
                                            .module
                                            .get_function("nyash.string.concat_is")
                                            .unwrap_or_else(|| {
                                                codegen.module.add_function(
                                                    "nyash.string.concat_is",
                                                    fnty,
                                                    None,
                                                )
                                            });
                                        let call = codegen
                                            .builder
                                            .build_call(callee, &[li.into(), rp.into()], "concat_is")
                                            .map_err(|e| e.to_string())?;
                                        let rv = call
                                            .try_as_basic_value()
                                            .left()
                                            .ok_or("concat_is returned void".to_string())?;
                                        vmap.insert(*dst, rv);
                                        handled_concat = true;
                                    }
                                }
                                _ => {}
                            }
                        }
                        if handled_concat {
                            // Concat already lowered and dst set
                        } else {
                            let out = if let (Some(li), Some(ri)) = (as_int(lv), as_int(rv)) {
                                use crate::mir::BinaryOp as B;
                                match op {
                                    B::Add => codegen
                                        .builder
                                        .build_int_add(li, ri, "iadd")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::Sub => codegen
                                        .builder
                                        .build_int_sub(li, ri, "isub")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::Mul => codegen
                                        .builder
                                        .build_int_mul(li, ri, "imul")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::Div => codegen
                                        .builder
                                        .build_int_signed_div(li, ri, "idiv")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::Mod => codegen
                                        .builder
                                        .build_int_signed_rem(li, ri, "imod")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::BitAnd => codegen
                                        .builder
                                        .build_and(li, ri, "iand")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::BitOr => codegen
                                        .builder
                                        .build_or(li, ri, "ior")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::BitXor => codegen
                                        .builder
                                        .build_xor(li, ri, "ixor")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::Shl => codegen
                                        .builder
                                        .build_left_shift(li, ri, "ishl")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::Shr => codegen
                                        .builder
                                        .build_right_shift(li, ri, false, "ishr")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::And | B::Or => {
                                        // Treat as logical on integers: convert to i1 and and/or
                                        let lb =
                                            to_bool(codegen.context, li.into(), &codegen.builder)?;
                                        let rb =
                                            to_bool(codegen.context, ri.into(), &codegen.builder)?;
                                        match op {
                                            B::And => codegen
                                                .builder
                                                .build_and(lb, rb, "land")
                                                .map_err(|e| e.to_string())?
                                                .into(),
                                            _ => codegen
                                                .builder
                                                .build_or(lb, rb, "lor")
                                                .map_err(|e| e.to_string())?
                                                .into(),
                                        }
                                    }
                                }
                            } else if let (Some(lf), Some(rf)) = (as_float(lv), as_float(rv)) {
                                use crate::mir::BinaryOp as B;
                                match op {
                                    B::Add => codegen
                                        .builder
                                        .build_float_add(lf, rf, "fadd")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::Sub => codegen
                                        .builder
                                        .build_float_sub(lf, rf, "fsub")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::Mul => codegen
                                        .builder
                                        .build_float_mul(lf, rf, "fmul")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::Div => codegen
                                        .builder
                                        .build_float_div(lf, rf, "fdiv")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::Mod => return Err("fmod not supported yet".to_string()),
                                    _ => return Err("bit/logic ops on float".to_string()),
                                }
                            } else {
                                return Err("binop type mismatch".to_string());
                            };
                            vmap.insert(*dst, out);
                        }
                    }
                    MirInstruction::Compare { dst, op, lhs, rhs } => {
                        let out = instructions::lower_compare(&codegen, func, &vmap, op, lhs, rhs)?;
                        vmap.insert(*dst, out);
                    }
                    MirInstruction::Store { value, ptr } => {
                        instructions::lower_store(&codegen, &vmap, &mut allocas, &mut alloca_elem_types, value, ptr)?;
                    }
                    MirInstruction::Load { dst, ptr } => {
                        instructions::lower_load(&codegen, &mut vmap, &mut allocas, &mut alloca_elem_types, dst, ptr)?;
                    }
                    MirInstruction::Phi { .. } => {
                        // Already created in pre-pass; nothing to do here.
                    }
                    _ => { /* ignore other ops for 11.1 */ }
                }
            }
            if let Some(term) = &block.terminator {
                match term {
                    MirInstruction::Return { value } => {
                        instructions::emit_return(&codegen, func, &vmap, value)?;
                    }
                    MirInstruction::Jump { target } => {
                        instructions::emit_jump(&codegen, bid, target, &bb_map, &phis_by_block, &vmap)?;
                    }
                    MirInstruction::Branch { condition, then_bb, else_bb } => {
                        instructions::emit_branch(&codegen, bid, condition, then_bb, else_bb, &bb_map, &phis_by_block, &vmap)?;
                    }
                    _ => {}
                }
            }
            // Verify per-function
            if !llvm_func.verify(true) {
                return Err(format!("Function verification failed: {}", name));
            }
        }

        // Build entry wrapper ny_main -> call entry function
        let i64t = codegen.context.i64_type();
        let ny_main_ty = i64t.fn_type(&[], false);
        let ny_main = codegen.module.add_function("ny_main", ny_main_ty, None);
        let entry_bb = codegen.context.append_basic_block(ny_main, "entry");
        codegen.builder.position_at_end(entry_bb);
        let entry_sym = format!("ny_f_{}", sanitize(&entry_name));
        let entry_fn = codegen
            .module
            .get_function(&entry_sym)
            .ok_or_else(|| format!("entry function symbol not found: {}", entry_sym))?;
        let call = codegen
            .builder
            .build_call(entry_fn, &[], "call_main")
            .map_err(|e| e.to_string())?;
        let rv = call.try_as_basic_value().left();
        // Normalize to i64 return
        let ret_v = if let Some(v) = rv {
            match v {
                BasicValueEnum::IntValue(iv) => {
                    if iv.get_type().get_bit_width() == 64 {
                        iv
                    } else {
                        codegen
                            .builder
                            .build_int_z_extend(iv, i64t, "ret_zext")
                            .map_err(|e| e.to_string())?
                    }
                }
                BasicValueEnum::PointerValue(pv) => codegen
                    .builder
                    .build_ptr_to_int(pv, i64t, "ret_p2i")
                    .map_err(|e| e.to_string())?,
                BasicValueEnum::FloatValue(fv) => codegen
                    .builder
                    .build_float_to_signed_int(fv, i64t, "ret_f2i")
                    .map_err(|e| e.to_string())?,
                _ => i64t.const_zero(),
            }
        } else {
            i64t.const_zero()
        };
        codegen.builder.build_return(Some(&ret_v)).map_err(|e| e.to_string())?;

        // Verify and emit final object
        if !ny_main.verify(true) {
            return Err("ny_main verification failed".to_string());
        }
        // Try writing via file API first; if it succeeds but file is missing due to env/FS quirks,
        // also write via memory buffer as a fallback to ensure presence.
        let verbose = std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1");
        if verbose {
            eprintln!("[LLVM] emitting object to {} (begin)", output_path);
        }
        match codegen.target_machine.write_to_file(
            &codegen.module,
            inkwell::targets::FileType::Object,
            std::path::Path::new(output_path),
        ) {
            Ok(_) => {
                // Verify; if missing, fallback to memory buffer write
                if std::fs::metadata(output_path).is_err() {
                    let buf = codegen
                        .target_machine
                        .write_to_memory_buffer(&codegen.module, inkwell::targets::FileType::Object)
                        .map_err(|e| format!("Failed to get object buffer: {}", e))?;
                    std::fs::write(output_path, buf.as_slice()).map_err(|e| {
                        format!("Failed to write object to '{}': {}", output_path, e)
                    })?;
                    if verbose {
                        eprintln!(
                            "[LLVM] wrote object via memory buffer fallback: {} ({} bytes)",
                            output_path,
                            buf.get_size()
                        );
                    }
                } else if verbose {
                    if let Ok(meta) = std::fs::metadata(output_path) {
                        eprintln!(
                            "[LLVM] wrote object via file API: {} ({} bytes)",
                            output_path,
                            meta.len()
                        );
                    }
                }
                if verbose {
                    eprintln!("[LLVM] emit complete (Ok branch) for {}", output_path);
                }
                Ok(())
            }
            Err(e) => {
                // Fallback: memory buffer
                let buf = codegen
                    .target_machine
                    .write_to_memory_buffer(&codegen.module, inkwell::targets::FileType::Object)
                    .map_err(|ee| {
                        format!(
                            "Failed to write object ({}); and memory buffer failed: {}",
                            e, ee
                        )
                    })?;
                std::fs::write(output_path, buf.as_slice()).map_err(|ee| {
                    format!(
                        "Failed to write object to '{}': {} (original error: {})",
                        output_path, ee, e
                    )
                })?;
                if verbose {
                    eprintln!(
                        "[LLVM] wrote object via error fallback: {} ({} bytes)",
                        output_path,
                        buf.get_size()
                    );
                }
                if verbose {
                    eprintln!(
                        "[LLVM] emit complete (Err branch handled) for {}",
                        output_path
                    );
                }
                Ok(())
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_creation() {
        let compiler = LLVMCompiler::new();
        assert!(compiler.is_ok());
    }
}
