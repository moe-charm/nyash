use crate::mir::optimizer::MirOptimizer;
use crate::mir::optimizer_stats::OptimizationStats;
use crate::mir::{BarrierOp, MirModule, TypeOpKind, ValueId, WeakRefOp};

pub fn force_plugin_invoke(_opt: &mut MirOptimizer, module: &mut MirModule) -> OptimizationStats {
    use crate::mir::MirInstruction as I;
    let mut stats = OptimizationStats::new();
    for (_fname, function) in &mut module.functions {
        for (_bb, block) in &mut function.blocks {
            for inst in &mut block.instructions {
                if let I::BoxCall {
                    dst,
                    box_val,
                    method,
                    args,
                    effects,
                    ..
                } = inst.clone()
                {
                    *inst = I::PluginInvoke {
                        dst,
                        box_val,
                        method,
                        args,
                        effects,
                    };
                    stats.intrinsic_optimizations += 1;
                }
            }
        }
    }
    stats
}

pub fn normalize_python_helper_calls(
    _opt: &mut MirOptimizer,
    module: &mut MirModule,
) -> OptimizationStats {
    use crate::mir::MirInstruction as I;
    let mut stats = OptimizationStats::new();
    for (_fname, function) in &mut module.functions {
        for (_bb, block) in &mut function.blocks {
            for inst in &mut block.instructions {
                if let I::PluginInvoke {
                    box_val,
                    method,
                    args,
                    ..
                } = inst
                {
                    if method == "getattr" && args.len() >= 2 {
                        let new_recv = args[0];
                        args.remove(0);
                        *box_val = new_recv;
                        stats.intrinsic_optimizations += 1;
                    } else if method == "call" && !args.is_empty() {
                        let new_recv = args[0];
                        args.remove(0);
                        *box_val = new_recv;
                        stats.intrinsic_optimizations += 1;
                    }
                }
            }
        }
    }
    stats
}

pub fn normalize_legacy_instructions(
    _opt: &mut MirOptimizer,
    module: &mut MirModule,
) -> OptimizationStats {
    use crate::mir::MirInstruction as I;
    let mut stats = OptimizationStats::new();
    let rw_dbg = crate::config::env::rewrite_debug();
    let rw_sp = crate::config::env::rewrite_safepoint();
    let rw_future = crate::config::env::rewrite_future();
    let core13 = crate::config::env::mir_core13();
    let mut array_to_boxcall = crate::config::env::mir_array_boxcall();
    if core13 {
        array_to_boxcall = true;
    }
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
                        *inst = I::ExternCall {
                            dst: None,
                            iface_name: "env.console".to_string(),
                            method_name: "log".to_string(),
                            args: vec![v],
                            effects: crate::mir::EffectMask::PURE.add(crate::mir::Effect::Io),
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::ArrayGet { dst, array, index } if array_to_boxcall => {
                        let d = *dst;
                        let a = *array;
                        let i = *index;
                        let mid =
                            crate::mir::slot_registry::resolve_slot_by_type_name("ArrayBox", "get");
                        *inst = I::BoxCall {
                            dst: Some(d),
                            box_val: a,
                            method: "get".to_string(),
                            method_id: mid,
                            args: vec![i],
                            effects: crate::mir::EffectMask::READ,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::ArraySet {
                        array,
                        index,
                        value,
                    } if array_to_boxcall => {
                        let a = *array;
                        let i = *index;
                        let v = *value;
                        let mid =
                            crate::mir::slot_registry::resolve_slot_by_type_name("ArrayBox", "set");
                        *inst = I::BoxCall {
                            dst: None,
                            box_val: a,
                            method: "set".to_string(),
                            method_id: mid,
                            args: vec![i, v],
                            effects: crate::mir::EffectMask::WRITE,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::PluginInvoke {
                        dst,
                        box_val,
                        method,
                        args,
                        effects,
                    } => {
                        let d = *dst;
                        let recv = *box_val;
                        let m = method.clone();
                        let as_ = args.clone();
                        let eff = *effects;
                        *inst = I::BoxCall {
                            dst: d,
                            box_val: recv,
                            method: m,
                            method_id: None,
                            args: as_,
                            effects: eff,
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
                        *inst = I::ExternCall {
                            dst: Some(d),
                            iface_name: "env.future".to_string(),
                            method_name: "new".to_string(),
                            args: vec![v],
                            effects: crate::mir::EffectMask::PURE.add(crate::mir::Effect::Io),
                        };
                    }
                    I::FutureSet { future, value } if rw_future => {
                        let f = *future;
                        let v = *value;
                        *inst = I::ExternCall {
                            dst: None,
                            iface_name: "env.future".to_string(),
                            method_name: "set".to_string(),
                            args: vec![f, v],
                            effects: crate::mir::EffectMask::PURE.add(crate::mir::Effect::Io),
                        };
                    }
                    I::Await { dst, future } if rw_future => {
                        let d = *dst;
                        let f = *future;
                        *inst = I::ExternCall {
                            dst: Some(d),
                            iface_name: "env.future".to_string(),
                            method_name: "await".to_string(),
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
                        let ty = crate::mir::instruction::MirType::Box(expected_type.clone());
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
                        *term = I::ExternCall {
                            dst: None,
                            iface_name: "env.console".to_string(),
                            method_name: "log".to_string(),
                            args: vec![v],
                            effects: crate::mir::EffectMask::PURE,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::ArrayGet { dst, array, index } if array_to_boxcall => {
                        let d = *dst;
                        let a = *array;
                        let i = *index;
                        *term = I::BoxCall {
                            dst: Some(d),
                            box_val: a,
                            method: "get".to_string(),
                            method_id: None,
                            args: vec![i],
                            effects: crate::mir::EffectMask::READ,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::ArraySet {
                        array,
                        index,
                        value,
                    } if array_to_boxcall => {
                        let a = *array;
                        let i = *index;
                        let v = *value;
                        *term = I::BoxCall {
                            dst: None,
                            box_val: a,
                            method: "set".to_string(),
                            method_id: None,
                            args: vec![i, v],
                            effects: crate::mir::EffectMask::WRITE,
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

pub fn normalize_ref_field_access(
    _opt: &mut MirOptimizer,
    module: &mut MirModule,
) -> OptimizationStats {
    use crate::mir::MirInstruction as I;
    let mut stats = OptimizationStats::new();
    for (_fname, function) in &mut module.functions {
        for (_bb, block) in &mut function.blocks {
            let mut out: Vec<I> = Vec::with_capacity(block.instructions.len() + 2);
            let old = std::mem::take(&mut block.instructions);
            for inst in old.into_iter() {
                match inst {
                    I::RefGet {
                        dst,
                        reference,
                        field,
                    } => {
                        let new_id = ValueId::new(function.next_value_id);
                        function.next_value_id += 1;
                        out.push(I::Const {
                            dst: new_id,
                            value: crate::mir::instruction::ConstValue::String(field),
                        });
                        out.push(I::BoxCall {
                            dst: Some(dst),
                            box_val: reference,
                            method: "getField".to_string(),
                            method_id: None,
                            args: vec![new_id],
                            effects: crate::mir::EffectMask::READ,
                        });
                        stats.intrinsic_optimizations += 1;
                    }
                    I::RefSet {
                        reference,
                        field,
                        value,
                    } => {
                        let new_id = ValueId::new(function.next_value_id);
                        function.next_value_id += 1;
                        out.push(I::Const {
                            dst: new_id,
                            value: crate::mir::instruction::ConstValue::String(field),
                        });
                        out.push(I::Barrier {
                            op: BarrierOp::Write,
                            ptr: reference,
                        });
                        out.push(I::BoxCall {
                            dst: None,
                            box_val: reference,
                            method: "setField".to_string(),
                            method_id: None,
                            args: vec![new_id, value],
                            effects: crate::mir::EffectMask::WRITE,
                        });
                        stats.intrinsic_optimizations += 1;
                    }
                    other => out.push(other),
                }
            }
            block.instructions = out;

            if let Some(term) = block.terminator.take() {
                block.terminator = Some(match term {
                    I::RefGet {
                        dst,
                        reference,
                        field,
                    } => {
                        let new_id = ValueId::new(function.next_value_id);
                        function.next_value_id += 1;
                        block.instructions.push(I::Const {
                            dst: new_id,
                            value: crate::mir::instruction::ConstValue::String(field),
                        });
                        I::BoxCall {
                            dst: Some(dst),
                            box_val: reference,
                            method: "getField".to_string(),
                            method_id: None,
                            args: vec![new_id],
                            effects: crate::mir::EffectMask::READ,
                        }
                    }
                    I::RefSet {
                        reference,
                        field,
                        value,
                    } => {
                        let new_id = ValueId::new(function.next_value_id);
                        function.next_value_id += 1;
                        block.instructions.push(I::Const {
                            dst: new_id,
                            value: crate::mir::instruction::ConstValue::String(field),
                        });
                        block.instructions.push(I::Barrier {
                            op: BarrierOp::Write,
                            ptr: reference,
                        });
                        I::BoxCall {
                            dst: None,
                            box_val: reference,
                            method: "setField".to_string(),
                            method_id: None,
                            args: vec![new_id, value],
                            effects: crate::mir::EffectMask::WRITE,
                        }
                    }
                    other => other,
                });
            }
        }
    }
    stats
}
