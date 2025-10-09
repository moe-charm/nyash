use crate::mir::optimizer::MirOptimizer;
use crate::mir::optimizer_stats::OptimizationStats;
use crate::mir::{BarrierOp, MirModule, TypeOpKind, ValueId, WeakRefOp};

// PluginInvoke retired: no-op stubs removed

pub fn normalize_legacy_instructions(
    _opt: &mut MirOptimizer,
    module: &mut MirModule,
) -> OptimizationStats {
    use crate::mir::MirInstruction as I;
    let mut stats = OptimizationStats::new();
    let rw_dbg = crate::config::env::rewrite_debug();
    let rw_sp = crate::config::env::rewrite_safepoint();
    let rw_future = crate::config::env::rewrite_future();
    // ArrayGet/Set → BoxCall を常時ON（レガシー撤退のため）
    let array_to_boxcall = true;
    for (_fname, function) in &mut module.functions {
        for (_bb, block) in &mut function.blocks {
            for inst in &mut block.instructions {
                match inst {
                    I::WeakNew { dst, box_val } => {
                        let d = *dst;
                        let v = *box_val;
                        *inst = I::WeakRef {
                            dst: d,
                            op: WeakRefOp::New,
                            value: v,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::WeakLoad { dst, weak_ref } => {
                        let d = *dst;
                        let v = *weak_ref;
                        *inst = I::WeakRef {
                            dst: d,
                            op: WeakRefOp::Load,
                            value: v,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::BarrierRead { ptr } => {
                        let p = *ptr;
                        *inst = I::Barrier {
                            op: BarrierOp::Read,
                            ptr: p,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::BarrierWrite { ptr } => {
                        let p = *ptr;
                        *inst = I::Barrier {
                            op: BarrierOp::Write,
                            ptr: p,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::Print { value, .. } => {
                        let v = *value;
                        *inst = I::Call {
                            dst: None,
                            func: ValueId::new(0),
                            callee: Some(crate::mir::definitions::call_unified::Callee::Extern(
                                "env.console.log".to_string(),
                            )),
                            args: vec![v],
                            effects: crate::mir::EffectMask::PURE.add(crate::mir::Effect::Io),
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    
                    
                    I::Debug { .. } if !rw_dbg => {
                        *inst = I::Nop;
                    }
                    I::Safepoint if !rw_sp => {
                        *inst = I::Nop;
                    }
                    I::FutureNew { dst, value } if rw_future => {
                        let d = *dst;
                        let v = *value;
                        *inst = I::Call {
                            dst: Some(d),
                            func: ValueId::new(0),
                            callee: Some(crate::mir::definitions::call_unified::Callee::Extern(
                                "env.future.new".to_string(),
                            )),
                            args: vec![v],
                            effects: crate::mir::EffectMask::PURE.add(crate::mir::Effect::Io),
                        };
                    }
                    I::FutureSet { future, value } if rw_future => {
                        let f = *future;
                        let v = *value;
                        *inst = I::Call {
                            dst: None,
                            func: ValueId::new(0),
                            callee: Some(crate::mir::definitions::call_unified::Callee::Extern(
                                "env.future.set".to_string(),
                            )),
                            args: vec![f, v],
                            effects: crate::mir::EffectMask::PURE.add(crate::mir::Effect::Io),
                        };
                    }
                    I::Await { dst, future } if rw_future => {
                        let d = *dst;
                        let f = *future;
                        *inst = I::Call {
                            dst: Some(d),
                            func: ValueId::new(0),
                            callee: Some(crate::mir::definitions::call_unified::Callee::Extern(
                                "env.future.await".to_string(),
                            )),
                            args: vec![f],
                            effects: crate::mir::EffectMask::PURE.add(crate::mir::Effect::Io),
                        };
                    }
                    _ => {}
                }
            }
            // terminator rewrite (subset migrated as needed)
            if let Some(term) = &mut block.terminator {
                match term {
                    I::TypeCheck {
                        dst,
                        value,
                        expected_type,
                    } => {
                        let ty = crate::mir::MirType::Box(expected_type.clone());
                        *term = I::TypeOp {
                            dst: *dst,
                            op: TypeOpKind::Check,
                            value: *value,
                            ty,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::Cast {
                        dst,
                        value,
                        target_type,
                    } => {
                        let ty = target_type.clone();
                        *term = I::TypeOp {
                            dst: *dst,
                            op: TypeOpKind::Cast,
                            value: *value,
                            ty,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::WeakNew { dst, box_val } => {
                        let d = *dst;
                        let v = *box_val;
                        *term = I::WeakRef {
                            dst: d,
                            op: WeakRefOp::New,
                            value: v,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::WeakLoad { dst, weak_ref } => {
                        let d = *dst;
                        let v = *weak_ref;
                        *term = I::WeakRef {
                            dst: d,
                            op: WeakRefOp::Load,
                            value: v,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::BarrierRead { ptr } => {
                        let p = *ptr;
                        *term = I::Barrier {
                            op: BarrierOp::Read,
                            ptr: p,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::BarrierWrite { ptr } => {
                        let p = *ptr;
                        *term = I::Barrier {
                            op: BarrierOp::Write,
                            ptr: p,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::Print { value, .. } => {
                        let v = *value;
                        *term = I::Call {
                            dst: None,
                            func: ValueId::new(0),
                            callee: Some(crate::mir::definitions::call_unified::Callee::Extern(
                                "env.console.log".to_string(),
                            )),
                            args: vec![v],
                            effects: crate::mir::EffectMask::PURE.add(crate::mir::Effect::Io),
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    _ => {}
                }
            }
        }
    }
    stats
}

pub fn normalize_ref_field_access(_opt: &mut MirOptimizer, _module: &mut MirModule) -> OptimizationStats {
    // RefGet/RefSet retired: no-op
    OptimizationStats::new()
}
