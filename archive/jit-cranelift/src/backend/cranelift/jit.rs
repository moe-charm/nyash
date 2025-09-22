#![cfg(feature = "cranelift-jit")]

use cranelift_codegen::ir::{
    condcodes::IntCC, types, AbiParam, InstBuilder, Signature, StackSlot, StackSlotData,
    StackSlotKind,
};
use cranelift_codegen::isa;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};

use crate::mir::{BasicBlockId, CompareOp, ConstValue, MirFunction, MirInstruction, ValueId};

/// Compile a minimal subset of MIR(main) to a native function and execute it.
/// Supported: Const(Integer), BinOp(Add for integers), Return(Integer or default 0).
pub fn compile_and_execute_minimal(main: &MirFunction) -> Result<i64, String> {
    // ISA (native)
    let isa_builder = cranelift_native::builder().map_err(|e| e.to_string())?;
    let flag_builder = cranelift_codegen::settings::builder();
    let flags = cranelift_codegen::settings::Flags::new(flag_builder);
    let isa = isa_builder.finish(flags).map_err(|e| e.to_string())?;

    let jit_builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
    let mut module = JITModule::new(jit_builder);

    // Signature: () -> i64
    let mut sig = Signature::new(module.target_config().default_call_conv);
    sig.returns
        .push(AbiParam::new(cranelift_codegen::ir::types::I64));
    let func_id = module
        .declare_function("ny_main", Linkage::Export, &sig)
        .map_err(|e| e.to_string())?;

    let mut ctx = module.make_context();
    ctx.func.signature = sig;
    let mut fb_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);

    // Prepare blocks for the entire MIR function
    use std::collections::HashMap;
    let mut clif_blocks: HashMap<BasicBlockId, cranelift_codegen::ir::Block> = HashMap::new();
    let mut vals: HashMap<ValueId, cranelift_codegen::ir::Value> = HashMap::new();
    let mut slots: HashMap<ValueId, StackSlot> = HashMap::new();

    for (bb_id, _) in &main.blocks {
        clif_blocks.insert(*bb_id, builder.create_block());
    }
    // Switch to entry
    let entry = *clif_blocks.get(&main.entry_block).unwrap();
    builder.switch_to_block(entry);
    builder.append_block_params_for_function_params(entry);

    // Emit each block
    // Deterministic order by id
    let mut bb_ids: Vec<_> = main.blocks.keys().copied().collect();
    bb_ids.sort_by_key(|b| b.0);
    for bb_id in bb_ids {
        let bb = main.blocks.get(&bb_id).unwrap();
        let cb = *clif_blocks.get(&bb_id).unwrap();
        builder.switch_to_block(cb);
        for inst in &bb.instructions {
            match inst {
                MirInstruction::Const { dst, value } => {
                    let v = match value {
                        ConstValue::Integer(i) => builder.ins().iconst(types::I64, *i),
                        ConstValue::Bool(b) => {
                            builder.ins().iconst(types::I64, if *b { 1 } else { 0 })
                        }
                        ConstValue::Float(f) => {
                            let fv = builder.ins().f64const(*f);
                            builder.ins().fcvt_to_sint(types::I64, fv)
                        }
                        ConstValue::String(_) | ConstValue::Null | ConstValue::Void => {
                            builder.ins().iconst(types::I64, 0)
                        }
                    };
                    vals.insert(*dst, v);
                }
                MirInstruction::BinOp { dst, op, lhs, rhs } => {
                    use crate::mir::BinaryOp;
                    let l = *vals.get(lhs).ok_or_else(|| format!("undef {:?}", lhs))?;
                    let r = *vals.get(rhs).ok_or_else(|| format!("undef {:?}", rhs))?;
                    let out = match op {
                        BinaryOp::Add => builder.ins().iadd(l, r),
                        BinaryOp::Sub => builder.ins().isub(l, r),
                        BinaryOp::Mul => builder.ins().imul(l, r),
                        BinaryOp::Div => builder.ins().sdiv(l, r),
                        BinaryOp::Mod => builder.ins().srem(l, r),
                        _ => builder.ins().iconst(types::I64, 0),
                    };
                    vals.insert(*dst, out);
                }
                MirInstruction::Compare { dst, op, lhs, rhs } => {
                    let l = *vals.get(lhs).ok_or_else(|| format!("undef {:?}", lhs))?;
                    let r = *vals.get(rhs).ok_or_else(|| format!("undef {:?}", rhs))?;
                    let cc = match op {
                        CompareOp::Eq => IntCC::Equal,
                        CompareOp::Ne => IntCC::NotEqual,
                        CompareOp::Lt => IntCC::SignedLessThan,
                        CompareOp::Le => IntCC::SignedLessThanOrEqual,
                        CompareOp::Gt => IntCC::SignedGreaterThan,
                        CompareOp::Ge => IntCC::SignedGreaterThanOrEqual,
                    };
                    let b1 = builder.ins().icmp(cc, l, r);
                    let one = builder.ins().iconst(types::I64, 1);
                    let zero = builder.ins().iconst(types::I64, 0);
                    let i64v = builder.ins().select(b1, one, zero);
                    vals.insert(*dst, i64v);
                }
                MirInstruction::Load { dst, ptr } => {
                    if let Some(ss) = slots.get(ptr).copied() {
                        let v = builder.ins().stack_load(types::I64, ss, 0);
                        vals.insert(*dst, v);
                    } else {
                        vals.insert(*dst, builder.ins().iconst(types::I64, 0));
                    }
                }
                MirInstruction::Store { value, ptr } => {
                    let v = *vals
                        .get(value)
                        .ok_or_else(|| format!("undef {:?}", value))?;
                    let ss = *slots.entry(*ptr).or_insert_with(|| {
                        builder.create_sized_stack_slot(StackSlotData::new(
                            StackSlotKind::ExplicitSlot,
                            8,
                        ))
                    });
                    builder.ins().stack_store(v, ss, 0);
                }
                MirInstruction::Copy { dst, src } => {
                    let v = *vals.get(src).ok_or_else(|| format!("undef {:?}", src))?;
                    vals.insert(*dst, v);
                }
                _ => { /* ignore unhandled for now */ }
            }
        }
        // Terminator
        match &bb.terminator {
            Some(MirInstruction::Return { value }) => {
                let retv = if let Some(v) = value {
                    *vals.get(v).unwrap_or(&builder.ins().iconst(types::I64, 0))
                } else {
                    builder.ins().iconst(types::I64, 0)
                };
                builder.ins().return_(&[retv]);
            }
            Some(MirInstruction::Jump { target }) => {
                let t = *clif_blocks.get(target).unwrap();
                builder.ins().jump(t, &[]);
            }
            Some(MirInstruction::Branch {
                condition,
                then_bb,
                else_bb,
            }) => {
                let cond_i64 = *vals
                    .get(condition)
                    .unwrap_or(&builder.ins().iconst(types::I64, 0));
                let is_true = builder.ins().icmp_imm(IntCC::NotEqual, cond_i64, 0);
                let tb = *clif_blocks.get(then_bb).unwrap();
                let eb = *clif_blocks.get(else_bb).unwrap();
                builder.ins().brif(is_true, tb, &[], eb, &[]);
            }
            _ => {
                /* fallthrough not allowed: insert return 0 to keep verifier happy */
                let z = builder.ins().iconst(types::I64, 0);
                builder.ins().return_(&[z]);
            }
        }
        builder.seal_block(cb);
    }

    builder.finalize();

    module
        .define_function(func_id, &mut ctx)
        .map_err(|e| e.to_string())?;
    module.clear_context(&mut ctx);
    let _ = module.finalize_definitions();

    let code = module.get_finalized_function(func_id);
    let func = unsafe { std::mem::transmute::<_, extern "C" fn() -> i64>(code) };
    let result = func();
    Ok(result)
}
