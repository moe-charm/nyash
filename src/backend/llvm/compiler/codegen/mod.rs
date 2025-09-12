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
            // Create basic blocks (prefix names with function label to avoid any ambiguity)
            let fn_label = sanitize(name);
            let (mut bb_map, entry_bb) = instructions::create_basic_blocks(&codegen, llvm_func, func, &fn_label);
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
            // Snapshot of values at the end of each basic block (for sealed-SSA PHI wiring)
            let mut block_end_values: HashMap<crate::mir::BasicBlockId, HashMap<ValueId, BasicValueEnum>> = HashMap::new();
            // Build successors map (for optional sealed-SSA PHI wiring)
            let mut succs: HashMap<crate::mir::BasicBlockId, Vec<crate::mir::BasicBlockId>> = HashMap::new();
            for (bid, block) in &func.blocks {
                let v: Vec<crate::mir::BasicBlockId> = block.successors.iter().copied().collect();
                succs.insert(*bid, v);
            }
            // Bind parameters
            for (i, pid) in func.params.iter().enumerate() {
                if let Some(av) = llvm_func.get_nth_param(i as u32) {
                    vmap.insert(*pid, av);
                }
            }
            // Gather block order once for fallthrough handling
            let block_ids: Vec<crate::mir::BasicBlockId> = func.block_ids().into_iter().collect();

            // Precreate phis
            for bid in &block_ids {
                let bb = *bb_map.get(bid).ok_or("missing bb in map")?;
                codegen.builder.position_at_end(bb);
                let block = func.blocks.get(bid).unwrap();
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
                            .entry(*bid)
                            .or_default()
                            .push((*dst, phi, inputs.clone()));
                        if std::env::var("NYASH_LLVM_TRACE_PHI").ok().as_deref() == Some("1") {
                            let ty_str = phi
                                .as_basic_value()
                                .get_type()
                                .print_to_string()
                                .to_string();
                            let mut pairs: Vec<String> = Vec::new();
                            for (pb, vid) in inputs {
                                pairs.push(format!("({}->{})", pb.as_u32(), vid.as_u32()));
                            }
                            eprintln!(
                                "[PHI:new] fn={} bb={} dst={} ty={} inputs={}",
                                fn_label,
                                bid.as_u32(),
                                dst.as_u32(),
                                ty_str,
                                pairs.join(",")
                            );
                        }
                    }
                }
            }

            // Map of const strings for Call resolution
            let const_strs = build_const_str_map(func);

            // Lower body
            let sealed_mode = std::env::var("NYASH_LLVM_PHI_SEALED").ok().as_deref() == Some("1");
            for (bi, bid) in block_ids.iter().enumerate() {
                let bb = *bb_map.get(bid).unwrap();
                if codegen
                    .builder
                    .get_insert_block()
                    .map(|b| b != bb)
                    .unwrap_or(true)
                {
                    codegen.builder.position_at_end(bb);
                }
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    eprintln!("[LLVM] lowering bb={}", bid.as_u32());
                }
                let block = func.blocks.get(bid).unwrap();
                for inst in &block.instructions {
                    match inst {
                        MirInstruction::NewBox { dst, box_type, args } => {
                            instructions::lower_newbox(&codegen, &mut vmap, *dst, box_type, args, &box_type_ids)?;
                        },
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
                        },
                        MirInstruction::Call { dst, func: callee, args, .. } => {
                            instructions::lower_call(&codegen, func, &mut vmap, dst, callee, args, &const_strs, &llvm_funcs)?;
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
                        },
                    MirInstruction::ExternCall { dst, iface_name, method_name, args, effects: _ } => {
                        instructions::lower_externcall(&codegen, func, &mut vmap, dst, iface_name, method_name, args)?;
                    },
                    MirInstruction::UnaryOp { dst, op, operand } => {
                        instructions::lower_unary(&codegen, &mut vmap, *dst, op, operand)?;
                    },
                    MirInstruction::BinOp { dst, op, lhs, rhs } => {
                        instructions::lower_binop(&codegen, func, &mut vmap, *dst, op, lhs, rhs)?;
                    },
                    MirInstruction::Compare { dst, op, lhs, rhs } => {
                        let out = instructions::lower_compare(&codegen, func, &vmap, op, lhs, rhs)?;
                        vmap.insert(*dst, out);
                    },
                    MirInstruction::Store { value, ptr } => {
                        instructions::lower_store(&codegen, &vmap, &mut allocas, &mut alloca_elem_types, value, ptr)?;
                    },
                    MirInstruction::Load { dst, ptr } => {
                        instructions::lower_load(&codegen, &mut vmap, &mut allocas, &mut alloca_elem_types, dst, ptr)?;
                    },
                    MirInstruction::Phi { .. } => {
                        // Already created in pre-pass; nothing to do here.
                    }
                    _ => { /* ignore other ops for 11.1 */ },
                }
                // Capture a snapshot of the value map at the end of this block's body
                block_end_values.insert(*bid, vmap.clone());
            }
            // Emit terminators and provide a conservative fallback when absent
            if let Some(term) = &block.terminator {
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    eprintln!("[LLVM] terminator present for bb={}", bid.as_u32());
                }
                // Ensure builder is positioned at current block before emitting terminator
                codegen.builder.position_at_end(bb);
                match term {
                    MirInstruction::Return { value } => {
                        instructions::emit_return(&codegen, func, &vmap, value)?;
                    }
                    MirInstruction::Jump { target } => {
                        instructions::emit_jump(&codegen, *bid, target, &bb_map, &phis_by_block, &vmap)?;
                    }
                    MirInstruction::Branch { condition, then_bb, else_bb } => {
                        instructions::emit_branch(&codegen, *bid, condition, then_bb, else_bb, &bb_map, &phis_by_block, &vmap)?;
                    }
                    _ => {
                        // Ensure builder is at this block before fallback branch
                        codegen.builder.position_at_end(bb);
                        // Unknown/unhandled terminator: conservatively branch forward
                        if let Some(next_bid) = block_ids.get(bi + 1) {
                            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                                eprintln!("[LLVM] unknown terminator fallback: bb={} -> next={}", bid.as_u32(), next_bid.as_u32());
                            }
                            instructions::emit_jump(&codegen, *bid, next_bid, &bb_map, &phis_by_block, &vmap)?;
                        } else {
                            let entry_first = func.entry_block;
                            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                                eprintln!("[LLVM] unknown terminator fallback: bb={} -> entry={}", bid.as_u32(), entry_first.as_u32());
                            }
                            instructions::emit_jump(&codegen, *bid, &entry_first, &bb_map, &phis_by_block, &vmap)?;
                        }
                    }
                }
            } else {
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    eprintln!("[LLVM] no terminator in MIR for bb={} (fallback)", bid.as_u32());
                }
                // Ensure builder is at this block before fallback branch
                codegen.builder.position_at_end(bb);
                // Fallback: branch to the next block if any; otherwise loop to entry
                if let Some(next_bid) = block_ids.get(bi + 1) {
                    instructions::emit_jump(&codegen, *bid, next_bid, &bb_map, &phis_by_block, &vmap)?;
                } else {
                    // last block, loop to entry to satisfy verifier
                    let entry_first = func.entry_block;
                    instructions::emit_jump(&codegen, *bid, &entry_first, &bb_map, &phis_by_block, &vmap)?;
                }
            }
            // Extra guard: if the current LLVM basic block still lacks a terminator for any reason,
            // insert a conservative branch to the next block (or entry if last) to satisfy verifier.
            if unsafe { bb.get_terminator() }.is_none() {
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    eprintln!("[LLVM] extra guard inserting fallback for bb={}", bid.as_u32());
                }
                // Ensure the builder is positioned at the end of this block before inserting the fallback terminator
                codegen.builder.position_at_end(bb);
                if let Some(next_bid) = block_ids.get(bi + 1) {
                    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                        eprintln!("[LLVM] fallback terminator: bb={} -> next={}", bid.as_u32(), next_bid.as_u32());
                    }
                    instructions::emit_jump(&codegen, *bid, next_bid, &bb_map, &phis_by_block, &vmap)?;
                } else {
                    let entry_first = func.entry_block;
                    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                        eprintln!("[LLVM] fallback terminator: bb={} -> entry={}", bid.as_u32(), entry_first.as_u32());
                    }
                    instructions::emit_jump(&codegen, *bid, &entry_first, &bb_map, &phis_by_block, &vmap)?;
                }
            }
                if sealed_mode {
                    instructions::flow::seal_block(&codegen, *bid, &succs, &bb_map, &phis_by_block, &block_end_values, &vmap)?;
                }
            }
        // Verify the fully-lowered function once, after all blocks
        if !llvm_func.verify(true) {
            return Err(format!("Function verification failed: {}", name));
        }

        }
        // End of per-function lowering loop

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
