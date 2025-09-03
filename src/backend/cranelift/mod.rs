/*!
 * Cranelift Backend (skeleton) - Compile MIR to native code for JIT/AOT
 *
 * Phase 11.7 kick-off: minimal stubs behind the `cranelift-jit` feature.
 */

#![cfg(feature = "cranelift-jit")]

use crate::mir::{function::MirModule, MirInstruction, ConstValue, ValueId, BinaryOp};
#[cfg(feature = "cranelift-jit")]
use crate::semantics::Semantics;
use crate::box_trait::{NyashBox, IntegerBox, BoolBox, StringBox, VoidBox};
use std::collections::HashMap;
use crate::jit::lower::{builder::{NoopBuilder, IRBuilder}, core::LowerCore};
use crate::jit::semantics::clif::ClifSemanticsSkeleton;

pub mod context; // Context/Block/Value env wrappers
pub mod builder; // Clif IRBuilder implementation (skeleton)
pub mod lower {}
pub mod jit; // JIT compile/execute using Cranelift (minimal)
pub mod object {}

/// JIT: compile and execute a MIR module (skeleton)
pub fn compile_and_execute(mir_module: &MirModule, _temp_name: &str) -> Result<Box<dyn NyashBox>, String> {
    // Minimal semantics: Const/Return/Add only (straight-line code)
    let main = mir_module.functions.get("main").ok_or("missing main function")?;

    // Minimal ClifSem lowering pass (NoopBuilder): Const/Return/Add カバレッジ確認
    if std::env::var("NYASH_JIT_DUMP").ok().as_deref() == Some("1") {
        let mut builder = NoopBuilder::new();
        let mut lower = LowerCore::new();
        let _ = lower.lower_function(main, &mut builder);
        eprintln!("[CLIF-LOWER] covered={} unsupported={}", lower.covered, lower.unsupported);
    }

    // まずは新JITエンジン経路を試す（LowerCore -> CraneliftBuilder -> 実行）
    #[cfg(feature = "cranelift-jit")]
    {
        let mut engine = crate::jit::engine::JitEngine::new();
        if let Some(h) = engine.compile_function(&main.signature.name, main) {
            // 実行（引数なし）。戻り値は MIR の型に合わせて変換
            let out = engine.execute_handle(h, &[]);
            if let Some(jv) = out {
                let vmv = crate::jit::boundary::CallBoundaryBox::to_vm(&main.signature.return_type, jv);
                let boxed: Box<dyn NyashBox> = match vmv {
                    crate::backend::vm::VMValue::Integer(i) => Box::new(IntegerBox::new(i)),
                    crate::backend::vm::VMValue::Float(f) => Box::new(crate::boxes::FloatBox::new(f)),
                    crate::backend::vm::VMValue::Bool(b) => Box::new(BoolBox::new(b)),
                    crate::backend::vm::VMValue::String(s) => Box::new(StringBox::new(&s)),
                    crate::backend::vm::VMValue::BoxRef(b) => b.share_box(),
                    crate::backend::vm::VMValue::Future(fu) => Box::new(fu),
                    crate::backend::vm::VMValue::Void => Box::new(VoidBox::new()),
                };
                return Ok(boxed);
            }
        }
        // 失敗した場合はミニマルJITへフォールバック
        if let Ok(i) = crate::backend::cranelift::jit::compile_and_execute_minimal(main) {
            return Ok(Box::new(IntegerBox::new(i)));
        }
    }
    let mut regs: HashMap<ValueId, crate::backend::vm::VMValue> = HashMap::new();
    let mut cur = main.entry_block;
    let mut last_pred: Option<crate::mir::BasicBlockId> = None;
    loop {
        let bb = main.blocks.get(&cur).ok_or_else(|| format!("invalid bb {:?}", cur))?;
        // PHI (very minimal): choose first input or predecessor match
        for inst in &bb.instructions {
            if let MirInstruction::Phi { dst, inputs } = inst {
                if let Some(pred) = last_pred {
                    if let Some((_, v)) = inputs.iter().find(|(b, _)| *b == pred) { if let Some(val) = regs.get(v).cloned() { regs.insert(*dst, val); } }
                } else if let Some((_, v)) = inputs.first() { if let Some(val) = regs.get(v).cloned() { regs.insert(*dst, val); } }
            }
        }
    let mut sem = ClifSemanticsSkeleton::new();
    for inst in &bb.instructions {
        match inst {
            MirInstruction::Const { dst, value } => {
                    let vv = match value {
                        ConstValue::Integer(i) => sem.const_i64(*i),
                        ConstValue::Float(f) => sem.const_f64(*f),
                        ConstValue::Bool(b) => sem.const_bool(*b),
                        ConstValue::String(s) => sem.const_str(s),
                        ConstValue::Null | ConstValue::Void => sem.const_null(),
                    };
                    regs.insert(*dst, vv);
            }
            MirInstruction::BinOp { dst, op, lhs, rhs } if matches!(op, BinaryOp::Add) => {
                use crate::backend::vm::VMValue as V;
                let a = regs.get(lhs).cloned().ok_or_else(|| format!("undef {:?}", lhs))?;
                let b = regs.get(rhs).cloned().ok_or_else(|| format!("undef {:?}", rhs))?;
                let out = sem.add(a, b);
                regs.insert(*dst, out);
            }
                MirInstruction::Copy { dst, src } => {
                    if let Some(v) = regs.get(src).cloned() { regs.insert(*dst, v); }
                }
                MirInstruction::Debug { .. } | MirInstruction::Print { .. } | MirInstruction::Barrier { .. } | MirInstruction::BarrierRead { .. } | MirInstruction::BarrierWrite { .. } | MirInstruction::Safepoint | MirInstruction::Load { .. } | MirInstruction::Store { .. } | MirInstruction::TypeOp { .. } | MirInstruction::Compare { .. } | MirInstruction::NewBox { .. } | MirInstruction::PluginInvoke { .. } | MirInstruction::BoxCall { .. } | MirInstruction::ExternCall { .. } | MirInstruction::RefGet { .. } | MirInstruction::RefSet { .. } | MirInstruction::WeakRef { .. } | MirInstruction::FutureNew { .. } | MirInstruction::FutureSet { .. } | MirInstruction::Await { .. } | MirInstruction::Throw { .. } | MirInstruction::Catch { .. } => {
                    // ignore for minimal path
                }
                MirInstruction::Phi { .. } => { /* handled above */ }
                _ => {}
            }
        }
        match &bb.terminator {
            Some(MirInstruction::Return { value }) => {
                use crate::backend::vm::VMValue as V;
                let vb = match value {
                    Some(v) => regs.get(v).cloned().unwrap_or(V::Void),
                    None => V::Void,
                };
                // Box to NyashBox
                let out: Box<dyn NyashBox> = match vb {
                    V::Integer(i) => Box::new(IntegerBox::new(i)),
                    V::Bool(b) => Box::new(BoolBox::new(b)),
                    V::String(s) => Box::new(StringBox::new(&s)),
                    V::Float(f) => Box::new(crate::boxes::FloatBox::new(f)),
                    V::Void => Box::new(VoidBox::new()),
                    V::Future(fu) => Box::new(fu),
                    V::BoxRef(b) => b.share_box(),
                };
                return Ok(out);
            }
            Some(MirInstruction::Jump { target }) => { last_pred = Some(bb.id); cur = *target; }
            Some(MirInstruction::Branch { condition, then_bb, else_bb }) => {
                // Minimal: integer/bool truthiness
                let c = regs.get(condition).cloned().unwrap_or(crate::backend::vm::VMValue::Void);
                let t = match c { crate::backend::vm::VMValue::Bool(b) => b, crate::backend::vm::VMValue::Integer(i) => i != 0, _ => false };
                last_pred = Some(bb.id);
                cur = if t { *then_bb } else { *else_bb };
            }
            Some(other) => return Err(format!("unsupported terminator {:?}", other)),
            None => return Err(format!("unterminated block {:?}", bb.id)),
        }
    }
}

/// AOT: compile to object file (not yet implemented in skeleton)
pub fn compile_to_object(_mir_module: &MirModule, _out_path: &str) -> Result<(), String> {
    Err("Cranelift AOT emit not implemented (skeleton)".to_string())
}
