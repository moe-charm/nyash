use crate::mir::optimizer::MirOptimizer;
use crate::mir::optimizer_stats::OptimizationStats;
use crate::mir::{BinaryOp, CompareOp, EffectMask, MirInstruction as I, MirModule, ValueId};

/// Core-13 "pure" normalization: rewrite a few non-13 ops to allowed forms.
/// - Load(dst, ptr)  => ExternCall(Some dst, env.local.get, [ptr])
/// - Store(val, ptr) => ExternCall(None, env.local.set, [ptr, val])
/// - NewBox(dst, T, args...) => ExternCall(Some dst, env.box.new, [Const String(T), args...])
/// - UnaryOp:
///     Neg x    => BinOp(Sub, Const 0, x)
///     Not x    => Compare(Eq, x, Const false)
///     BitNot x => BinOp(BitXor, x, Const(-1))
pub fn normalize_pure_core13(_opt: &mut MirOptimizer, module: &mut MirModule) -> OptimizationStats {
    use crate::mir::instruction::ConstValue;
    let mut stats = OptimizationStats::new();
    for (_fname, function) in &mut module.functions {
        for (_bb, block) in &mut function.blocks {
            let mut out: Vec<I> = Vec::with_capacity(block.instructions.len() + 8);
            let old = std::mem::take(&mut block.instructions);
            for inst in old.into_iter() {
                match inst {
                    I::Load { dst, ptr } => {
                        out.push(I::ExternCall {
                            dst: Some(dst),
                            iface_name: "env.local".to_string(),
                            method_name: "get".to_string(),
                            args: vec![ptr],
                            effects: EffectMask::READ,
                        });
                        stats.intrinsic_optimizations += 1;
                    }
                    I::Store { value, ptr } => {
                        out.push(I::ExternCall {
                            dst: None,
                            iface_name: "env.local".to_string(),
                            method_name: "set".to_string(),
                            args: vec![ptr, value],
                            effects: EffectMask::WRITE,
                        });
                        stats.intrinsic_optimizations += 1;
                    }
                    I::NewBox { dst, box_type, mut args } => {
                        // prepend type name as Const String
                        let ty_id = ValueId::new(function.next_value_id);
                        function.next_value_id += 1;
                        out.push(I::Const { dst: ty_id, value: ConstValue::String(box_type) });
                        let mut call_args = Vec::with_capacity(1 + args.len());
                        call_args.push(ty_id);
                        call_args.append(&mut args);
                        out.push(I::ExternCall {
                            dst: Some(dst),
                            iface_name: "env.box".to_string(),
                            method_name: "new".to_string(),
                            args: call_args,
                            effects: EffectMask::PURE, // constructor is logically alloc; conservatively PURE here
                        });
                        stats.intrinsic_optimizations += 1;
                    }
                    I::UnaryOp { dst, op, operand } => {
                        match op {
                            crate::mir::UnaryOp::Neg => {
                                let zero = ValueId::new(function.next_value_id);
                                function.next_value_id += 1;
                                out.push(I::Const { dst: zero, value: ConstValue::Integer(0) });
                                out.push(I::BinOp { dst, op: BinaryOp::Sub, lhs: zero, rhs: operand });
                            }
                            crate::mir::UnaryOp::Not => {
                                let f = ValueId::new(function.next_value_id);
                                function.next_value_id += 1;
                                out.push(I::Const { dst: f, value: ConstValue::Bool(false) });
                                out.push(I::Compare { dst, op: CompareOp::Eq, lhs: operand, rhs: f });
                            }
                            crate::mir::UnaryOp::BitNot => {
                                let all1 = ValueId::new(function.next_value_id);
                                function.next_value_id += 1;
                                out.push(I::Const { dst: all1, value: ConstValue::Integer(-1) });
                                out.push(I::BinOp { dst, op: BinaryOp::BitXor, lhs: operand, rhs: all1 });
                            }
                        }
                        stats.intrinsic_optimizations += 1;
                    }
                    other => out.push(other),
                }
            }
            block.instructions = out;

            if let Some(term) = block.terminator.take() {
                block.terminator = Some(match term {
                    I::Load { dst, ptr } => I::ExternCall {
                        dst: Some(dst),
                        iface_name: "env.local".to_string(),
                        method_name: "get".to_string(),
                        args: vec![ptr],
                        effects: EffectMask::READ,
                    },
                    I::Store { value, ptr } => I::ExternCall {
                        dst: None,
                        iface_name: "env.local".to_string(),
                        method_name: "set".to_string(),
                        args: vec![ptr, value],
                        effects: EffectMask::WRITE,
                    },
                    I::NewBox { dst, box_type, mut args } => {
                        let ty_id = ValueId::new(function.next_value_id);
                        function.next_value_id += 1;
                        block.instructions.push(I::Const { dst: ty_id, value: ConstValue::String(box_type) });
                        let mut call_args = Vec::with_capacity(1 + args.len());
                        call_args.push(ty_id);
                        call_args.append(&mut args);
                        I::ExternCall {
                            dst: Some(dst),
                            iface_name: "env.box".to_string(),
                            method_name: "new".to_string(),
                            args: call_args,
                            effects: EffectMask::PURE,
                        }
                    }
                    I::UnaryOp { dst, op, operand } => match op {
                        crate::mir::UnaryOp::Neg => {
                            let zero = ValueId::new(function.next_value_id);
                            function.next_value_id += 1;
                            block.instructions.push(I::Const { dst: zero, value: ConstValue::Integer(0) });
                            I::BinOp { dst, op: BinaryOp::Sub, lhs: zero, rhs: operand }
                        }
                        crate::mir::UnaryOp::Not => {
                            let f = ValueId::new(function.next_value_id);
                            function.next_value_id += 1;
                            block.instructions.push(I::Const { dst: f, value: ConstValue::Bool(false) });
                            I::Compare { dst, op: CompareOp::Eq, lhs: operand, rhs: f }
                        }
                        crate::mir::UnaryOp::BitNot => {
                            let all1 = ValueId::new(function.next_value_id);
                            function.next_value_id += 1;
                            block.instructions.push(I::Const { dst: all1, value: ConstValue::Integer(-1) });
                            I::BinOp { dst, op: BinaryOp::BitXor, lhs: operand, rhs: all1 }
                        }
                    },
                    other => other,
                });
            }
        }
    }
    stats
}
