/*!
 * MIR Optimizer - Phase 3 Implementation
 * 
 * Implements Effect System based optimizations for the new 26-instruction MIR
 * - Pure instruction reordering and CSE (Common Subexpression Elimination)
 * - BoxFieldLoad/Store dependency analysis
 * - Intrinsic function optimization
 * - Dead code elimination
 */

use super::{MirModule, MirFunction, MirInstruction, ValueId, MirType, EffectMask, Effect};
use crate::mir::optimizer_stats::OptimizationStats;
use std::collections::{HashMap, HashSet};

/// MIR optimization passes
pub struct MirOptimizer {
    /// Enable debug output for optimization passes
    debug: bool,
}

impl MirOptimizer {
    /// Create new optimizer
    pub fn new() -> Self {
        Self {
            debug: false,
        }
    }
    
    /// Enable debug output
    pub fn with_debug(mut self) -> Self {
        self.debug = true;
        self
    }
    
    
    /// Run all optimization passes on a MIR module
    pub fn optimize_module(&mut self, module: &mut MirModule) -> OptimizationStats {
        let mut stats = OptimizationStats::new();
        
        if self.debug {
            println!("🚀 Starting MIR optimization passes");
        }
        
        // Env toggles for phased MIR cleanup
        let core13 = crate::config::env::mir_core13();
        let mut ref_to_boxcall = crate::config::env::mir_ref_boxcall();
        if core13 { ref_to_boxcall = true; }

        // Pass 0: Normalize legacy instructions to unified forms
        //  - Includes optional Array→BoxCall guarded by env (inside the pass)
        stats.merge(crate::mir::optimizer_passes::normalize::normalize_legacy_instructions(self, module));
        // Pass 0.1: RefGet/RefSet → BoxCall(getField/setField) (guarded)
        if ref_to_boxcall {
            stats.merge(crate::mir::optimizer_passes::normalize::normalize_ref_field_access(self, module));
        }
        
        // Option: Force BoxCall → PluginInvoke (env)
        if crate::config::env::mir_plugin_invoke()
            || crate::config::env::plugin_only() {
            stats.merge(crate::mir::optimizer_passes::normalize::force_plugin_invoke(self, module));
        }

        // Normalize Python helper form: py.getattr(obj, name) → obj.getattr(name)
        stats.merge(crate::mir::optimizer_passes::normalize::normalize_python_helper_calls(self, module));

        // Pass 1: Dead code elimination (modularized pass)
        {
            let eliminated = crate::mir::passes::dce::eliminate_dead_code(module);
            stats.dead_code_eliminated += eliminated;
        }
        
        // Pass 2: Pure instruction CSE (modularized)
        {
            let eliminated = crate::mir::passes::cse::eliminate_common_subexpressions(module);
            stats.cse_eliminated += eliminated;
        }
        
        // Pass 3: Pure instruction reordering for better locality
        stats.merge(crate::mir::optimizer_passes::reorder::reorder_pure_instructions(self, module));
        
        // Pass 4: Intrinsic function optimization
        stats.merge(crate::mir::optimizer_passes::intrinsics::optimize_intrinsic_calls(self, module));

        // Safety-net passesは削除（Phase 2: 変換の一本化）。診断のみ後段で実施。
        
        // Pass 5: BoxField dependency optimization
        stats.merge(crate::mir::optimizer_passes::boxfield::optimize_boxfield_operations(self, module));

        // Pass 6: 受け手型ヒントの伝搬（callsite→callee）
        // 目的: helper(arr){ return arr.length() } のようなケースで、
        //       呼び出し元の引数型（String/Integer/Bool/Float）を callee の params に反映し、
        //       Lowererがより正確にBox種別を選べるようにする。
        let updates = crate::mir::passes::type_hints::propagate_param_type_hints(module);
        if updates > 0 { stats.intrinsic_optimizations += updates as usize; }

        // Pass 7 (optional): Core-13 pure normalization
        if crate::config::env::mir_core13_pure() {
            stats.merge(self.normalize_pure_core13(module));
        }

        if self.debug {
            println!("✅ Optimization complete: {}", stats);
        }
        // Diagnostics (informational): report unlowered patterns
        let diag1 = crate::mir::optimizer_passes::diagnostics::diagnose_unlowered_type_ops(self, module);
        stats.merge(diag1);
        // Diagnostics (policy): detect legacy (pre-unified) instructions when requested
        let diag2 = crate::mir::optimizer_passes::diagnostics::diagnose_legacy_instructions(self, module);
        stats.merge(diag2);
        
        stats
    }

    
    /// Eliminate dead code (unused values)
    fn eliminate_dead_code(&mut self, module: &mut MirModule) -> OptimizationStats {
        let mut stats = OptimizationStats::new();
        
        for (func_name, function) in &mut module.functions {
            if self.debug {
                println!("  🗑️  Dead code elimination in function: {}", func_name);
            }
            
            let eliminated = self.eliminate_dead_code_in_function(function);
            stats.dead_code_eliminated += eliminated;
        }
        
        stats
    }

    /// Core-13 "pure" normalization: rewrite a few non-13 ops to allowed forms.
    /// - Load(dst, ptr)  => ExternCall(Some dst, env.local.get, [ptr])
    /// - Store(val, ptr) => ExternCall(None, env.local.set, [ptr, val])
    /// - NewBox(dst, T, args...) => ExternCall(Some dst, env.box.new, [Const String(T), args...])
    /// - UnaryOp:
    ///     Neg x    => BinOp(Sub, Const 0, x)
    ///     Not x    => Compare(Eq, x, Const false)
    ///     BitNot x => BinOp(BitXor, x, Const(-1))
    fn normalize_pure_core13(&mut self, module: &mut MirModule) -> OptimizationStats {
        use super::instruction::ConstValue;
        use super::{MirInstruction as I, BinaryOp, CompareOp};
        let mut stats = OptimizationStats::new();
        for (_fname, function) in &mut module.functions {
            for (_bb, block) in &mut function.blocks {
                let mut out: Vec<I> = Vec::with_capacity(block.instructions.len() + 8);
                let old = std::mem::take(&mut block.instructions);
                for inst in old.into_iter() {
                    match inst {
                        I::Load { dst, ptr } => {
                            out.push(I::ExternCall {
                                dst: Some(dst), iface_name: "env.local".to_string(), method_name: "get".to_string(),
                                args: vec![ptr], effects: super::EffectMask::READ,
                            });
                            stats.intrinsic_optimizations += 1;
                        }
                        I::Store { value, ptr } => {
                            out.push(I::ExternCall {
                                dst: None, iface_name: "env.local".to_string(), method_name: "set".to_string(),
                                args: vec![ptr, value], effects: super::EffectMask::WRITE,
                            });
                            stats.intrinsic_optimizations += 1;
                        }
                        I::NewBox { dst, box_type, mut args } => {
                            // prepend type name as Const String
                            let ty_id = super::ValueId::new(function.next_value_id);
                            function.next_value_id += 1;
                            out.push(I::Const { dst: ty_id, value: ConstValue::String(box_type) });
                            let mut call_args = Vec::with_capacity(1 + args.len());
                            call_args.push(ty_id);
                            call_args.append(&mut args);
                            out.push(I::ExternCall {
                                dst: Some(dst), iface_name: "env.box".to_string(), method_name: "new".to_string(),
                                args: call_args, effects: super::EffectMask::PURE, // constructor is logically alloc; conservatively PURE here
                            });
                            stats.intrinsic_optimizations += 1;
                        }
                        I::UnaryOp { dst, op, operand } => {
                            match op {
                                super::UnaryOp::Neg => {
                                    let zero = super::ValueId::new(function.next_value_id); function.next_value_id += 1;
                                    out.push(I::Const { dst: zero, value: ConstValue::Integer(0) });
                                    out.push(I::BinOp { dst, op: BinaryOp::Sub, lhs: zero, rhs: operand });
                                }
                                super::UnaryOp::Not => {
                                    let f = super::ValueId::new(function.next_value_id); function.next_value_id += 1;
                                    out.push(I::Const { dst: f, value: ConstValue::Bool(false) });
                                    out.push(I::Compare { dst, op: CompareOp::Eq, lhs: operand, rhs: f });
                                }
                                super::UnaryOp::BitNot => {
                                    let all1 = super::ValueId::new(function.next_value_id); function.next_value_id += 1;
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
                        I::Load { dst, ptr } => {
                            I::ExternCall { dst: Some(dst), iface_name: "env.local".to_string(), method_name: "get".to_string(), args: vec![ptr], effects: super::EffectMask::READ }
                        }
                        I::Store { value, ptr } => {
                            I::ExternCall { dst: None, iface_name: "env.local".to_string(), method_name: "set".to_string(), args: vec![ptr, value], effects: super::EffectMask::WRITE }
                        }
                        I::NewBox { dst, box_type, mut args } => {
                            let ty_id = super::ValueId::new(function.next_value_id);
                            function.next_value_id += 1;
                            block.instructions.push(I::Const { dst: ty_id, value: ConstValue::String(box_type) });
                            let mut call_args = Vec::with_capacity(1 + args.len());
                            call_args.push(ty_id);
                            call_args.append(&mut args);
                            I::ExternCall { dst: Some(dst), iface_name: "env.box".to_string(), method_name: "new".to_string(), args: call_args, effects: super::EffectMask::PURE }
                        }
                        I::UnaryOp { dst, op, operand } => {
                            match op {
                                super::UnaryOp::Neg => {
                                    let zero = super::ValueId::new(function.next_value_id); function.next_value_id += 1;
                                    block.instructions.push(I::Const { dst: zero, value: ConstValue::Integer(0) });
                                    I::BinOp { dst, op: BinaryOp::Sub, lhs: zero, rhs: operand }
                                }
                                super::UnaryOp::Not => {
                                    let f = super::ValueId::new(function.next_value_id); function.next_value_id += 1;
                                    block.instructions.push(I::Const { dst: f, value: ConstValue::Bool(false) });
                                    I::Compare { dst, op: CompareOp::Eq, lhs: operand, rhs: f }
                                }
                                super::UnaryOp::BitNot => {
                                    let all1 = super::ValueId::new(function.next_value_id); function.next_value_id += 1;
                                    block.instructions.push(I::Const { dst: all1, value: ConstValue::Integer(-1) });
                                    I::BinOp { dst, op: BinaryOp::BitXor, lhs: operand, rhs: all1 }
                                }
                            }
                        }
                        other => other,
                    });
                }
            }
        }
        stats
    }
    
    /// Eliminate dead code in a single function
    fn eliminate_dead_code_in_function(&mut self, function: &mut MirFunction) -> usize {
        // Collect all used values
        let mut used_values = HashSet::new();
        
        // Mark values used in terminators and side-effect instructions
        for (_, block) in &function.blocks {
            for instruction in &block.instructions {
                // Always keep instructions with side effects
                if !instruction.effects().is_pure() {
                    if let Some(dst) = instruction.dst_value() {
                        used_values.insert(dst);
                    }
                    for used in instruction.used_values() {
                        used_values.insert(used);
                    }
                }
            }
            
            // Mark values used in terminators
            if let Some(terminator) = &block.terminator {
                for used in terminator.used_values() {
                    used_values.insert(used);
                }
            }
        }
        
        // Propagate usage backwards
        let mut changed = true;
        while changed {
            changed = false;
            for (_, block) in &function.blocks {
                for instruction in &block.instructions {
                    if let Some(dst) = instruction.dst_value() {
                        if used_values.contains(&dst) {
                            for used in instruction.used_values() {
                                if used_values.insert(used) {
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Remove unused pure instructions
        let mut eliminated = 0;
        for (bbid, block) in &mut function.blocks {
            block.instructions.retain(|instruction| {
                if instruction.effects().is_pure() {
                    if let Some(dst) = instruction.dst_value() {
                        if !used_values.contains(&dst) {
                            opt_debug(&format!("DCE drop @{}: {:?}", bbid.as_u32(), instruction));
                            eliminated += 1;
                            return false;
                        }
                    }
                }
                true
            });
        }
        
        eliminated
    }
    
    /// Common Subexpression Elimination for pure instructions
    fn common_subexpression_elimination(&mut self, module: &mut MirModule) -> OptimizationStats {
        let mut stats = OptimizationStats::new();
        
        for (func_name, function) in &mut module.functions {
            if self.debug {
                println!("  🔄 CSE in function: {}", func_name);
            }
            
            let eliminated = self.cse_in_function(function);
            stats.cse_eliminated += eliminated;
        }
        
        stats
    }
    
    /// CSE in a single function
    fn cse_in_function(&mut self, function: &mut MirFunction) -> usize {
        let mut expression_map: HashMap<String, ValueId> = HashMap::new();
        let mut replacements: HashMap<ValueId, ValueId> = HashMap::new();
        let mut eliminated = 0;
        
        for (_, block) in &mut function.blocks {
            for instruction in &mut block.instructions {
                // Only optimize pure instructions
                if instruction.effects().is_pure() {
                    let expr_key = self.instruction_to_key(instruction);
                    
                    if let Some(&existing_value) = expression_map.get(&expr_key) {
                        // Found common subexpression
                        if let Some(dst) = instruction.dst_value() {
                            replacements.insert(dst, existing_value);
                            eliminated += 1;
                        }
                    } else {
                        // First occurrence of this expression
                        if let Some(dst) = instruction.dst_value() {
                            expression_map.insert(expr_key, dst);
                        }
                    }
                }
            }
        }
        
        // Apply replacements (simplified - in full implementation would need proper SSA update)
        eliminated
    }
    
    /// Convert instruction to string key for CSE
    fn instruction_to_key(&self, instruction: &MirInstruction) -> String {
        match instruction {
            MirInstruction::Const { value, .. } => format!("const_{:?}", value),
            MirInstruction::BinOp { op, lhs, rhs, .. } => format!("binop_{:?}_{}_{}", op, lhs.as_u32(), rhs.as_u32()),
            MirInstruction::Compare { op, lhs, rhs, .. } => format!("cmp_{:?}_{}_{}", op, lhs.as_u32(), rhs.as_u32()),
            // BoxFieldLoad removed from instruction set
            // MirInstruction::BoxFieldLoad { box_val, field, .. } => format!("boxload_{}_{}", box_val.as_u32(), field),
            MirInstruction::Call { func, args, .. } => {
                let args_str = args.iter().map(|v| v.as_u32().to_string()).collect::<Vec<_>>().join(",");
                format!("call_{}_{}", func.as_u32(), args_str)
            },
            _ => format!("other_{:?}", instruction),
        }
    }
    
    // Reorder/Intrinsics/BoxField passes moved to optimizer_passes/* modules
}

impl MirOptimizer {
    /// Expose debug flag for helper modules
    pub(crate) fn debug_enabled(&self) -> bool { self.debug }
}

impl MirOptimizer {
    /// Rewrite all BoxCall to PluginInvoke to force plugin path (no builtin fallback)
    fn force_plugin_invoke(&mut self, module: &mut MirModule) -> OptimizationStats {
        crate::mir::optimizer_passes::normalize::force_plugin_invoke(self, module)
    }

    /// Normalize Python helper calls that route via PyRuntimeBox into proper receiver form.
    ///
    /// Rewrites: PluginInvoke { box_val=py (PyRuntimeBox), method="getattr"|"call", args=[obj, rest...] }
    ///        →  PluginInvoke { box_val=obj, method, args=[rest...] }
    fn normalize_python_helper_calls(&mut self, module: &mut MirModule) -> OptimizationStats {
        crate::mir::optimizer_passes::normalize::normalize_python_helper_calls(self, module)
    }
    /// Normalize legacy instructions into unified MIR26 forms.
    /// - TypeCheck/Cast → TypeOp(Check/Cast)
    /// - WeakNew/WeakLoad → WeakRef(New/Load)
    /// - BarrierRead/BarrierWrite → Barrier(Read/Write)
    /// - Print → ExternCall(env.console.log)
    fn normalize_legacy_instructions(&mut self, module: &mut MirModule) -> OptimizationStats {
        use super::{TypeOpKind, WeakRefOp, BarrierOp, MirInstruction as I, MirType};
        let mut stats = OptimizationStats::new();
        let rw_dbg = crate::config::env::rewrite_debug();
        let rw_sp = crate::config::env::rewrite_safepoint();
        let rw_future = crate::config::env::rewrite_future();
        // Phase 11.8 toggles
        let core13 = crate::config::env::mir_core13();
        let mut array_to_boxcall = crate::config::env::mir_array_boxcall();
        if core13 { array_to_boxcall = true; }
        for (_fname, function) in &mut module.functions {
            for (_bb, block) in &mut function.blocks {
                // Rewrite in-place for normal instructions
                for inst in &mut block.instructions {
                    match inst {
                        I::TypeCheck { dst, value, expected_type } => {
                            let ty = MirType::Box(expected_type.clone());
                            *inst = I::TypeOp { dst: *dst, op: TypeOpKind::Check, value: *value, ty };
                            stats.reorderings += 0; // no-op; keep stats structure alive
                        }
                        I::Cast { dst, value, target_type } => {
                            let ty = target_type.clone();
                            *inst = I::TypeOp { dst: *dst, op: TypeOpKind::Cast, value: *value, ty };
                        }
                        I::WeakNew { dst, box_val } => {
                            let val = *box_val;
                            *inst = I::WeakRef { dst: *dst, op: WeakRefOp::New, value: val };
                        }
                        I::WeakLoad { dst, weak_ref } => {
                            let val = *weak_ref;
                            *inst = I::WeakRef { dst: *dst, op: WeakRefOp::Load, value: val };
                        }
                        I::BarrierRead { ptr } => {
                            let val = *ptr;
                            *inst = I::Barrier { op: BarrierOp::Read, ptr: val };
                        }
                        I::BarrierWrite { ptr } => {
                            let val = *ptr;
                            *inst = I::Barrier { op: BarrierOp::Write, ptr: val };
                        }
                        I::Print { value, .. } => {
                            let v = *value;
                            *inst = I::ExternCall { dst: None, iface_name: "env.console".to_string(), method_name: "log".to_string(), args: vec![v], effects: EffectMask::PURE.add(Effect::Io) };
                        }
                        I::RefGet { .. } | I::RefSet { .. } => { /* handled in normalize_ref_field_access pass (guarded) */ }
                        I::ArrayGet { dst, array, index } if array_to_boxcall => {
                            let d = *dst; let a = *array; let i = *index;
                            let mid = crate::mir::slot_registry::resolve_slot_by_type_name("ArrayBox", "get");
                            *inst = I::BoxCall { dst: Some(d), box_val: a, method: "get".to_string(), method_id: mid, args: vec![i], effects: EffectMask::READ };
                        }
                        I::ArraySet { array, index, value } if array_to_boxcall => {
                            let a = *array; let i = *index; let v = *value;
                            let mid = crate::mir::slot_registry::resolve_slot_by_type_name("ArrayBox", "set");
                            *inst = I::BoxCall { dst: None, box_val: a, method: "set".to_string(), method_id: mid, args: vec![i, v], effects: EffectMask::WRITE };
                        }
                        I::PluginInvoke { dst, box_val, method, args, effects } => {
                            let d = *dst; let recv = *box_val; let m = method.clone(); let as_ = args.clone(); let eff = *effects;
                            *inst = I::BoxCall { dst: d, box_val: recv, method: m, method_id: None, args: as_, effects: eff };
                        }
                        I::Debug { value, .. } if rw_dbg => {
                            let v = *value;
                            *inst = I::ExternCall { dst: None, iface_name: "env.debug".to_string(), method_name: "trace".to_string(), args: vec![v], effects: EffectMask::PURE.add(Effect::Debug) };
                        }
                        I::Safepoint if rw_sp => {
                            *inst = I::ExternCall { dst: None, iface_name: "env.runtime".to_string(), method_name: "checkpoint".to_string(), args: vec![], effects: EffectMask::PURE };
                        }
                        // Future/Await の段階移行: ExternCall(env.future.*) に書き換え（トグル）
                        I::FutureNew { dst, value } if rw_future => {
                            let d = *dst; let v = *value;
                            *inst = I::ExternCall { dst: Some(d), iface_name: "env.future".to_string(), method_name: "new".to_string(), args: vec![v], effects: EffectMask::PURE.add(Effect::Io) };
                        }
                        I::FutureSet { future, value } if rw_future => {
                            let f = *future; let v = *value;
                            *inst = I::ExternCall { dst: None, iface_name: "env.future".to_string(), method_name: "set".to_string(), args: vec![f, v], effects: EffectMask::PURE.add(Effect::Io) };
                        }
                        I::Await { dst, future } if rw_future => {
                            let d = *dst; let f = *future;
                            *inst = I::ExternCall { dst: Some(d), iface_name: "env.future".to_string(), method_name: "await".to_string(), args: vec![f], effects: EffectMask::PURE.add(Effect::Io) };
                        }
                        _ => {}
                    }
                }
                // Rewrite terminator, if any
                if let Some(term) = &mut block.terminator {
                    match term {
                        I::TypeCheck { dst, value, expected_type } => {
                            let ty = MirType::Box(expected_type.clone());
                            *term = I::TypeOp { dst: *dst, op: TypeOpKind::Check, value: *value, ty };
                        }
                        I::Cast { dst, value, target_type } => {
                            let ty = target_type.clone();
                            *term = I::TypeOp { dst: *dst, op: TypeOpKind::Cast, value: *value, ty };
                        }
                        I::WeakNew { dst, box_val } => {
                            let val = *box_val;
                            *term = I::WeakRef { dst: *dst, op: WeakRefOp::New, value: val };
                        }
                        I::WeakLoad { dst, weak_ref } => {
                            let val = *weak_ref;
                            *term = I::WeakRef { dst: *dst, op: WeakRefOp::Load, value: val };
                        }
                        I::BarrierRead { ptr } => {
                            let val = *ptr;
                            *term = I::Barrier { op: BarrierOp::Read, ptr: val };
                        }
                        I::BarrierWrite { ptr } => {
                            let val = *ptr;
                            *term = I::Barrier { op: BarrierOp::Write, ptr: val };
                        }
                        I::Print { value, .. } => {
                            let v = *value;
                            *term = I::ExternCall { dst: None, iface_name: "env.console".to_string(), method_name: "log".to_string(), args: vec![v], effects: EffectMask::PURE.add(Effect::Io) };
                        }
                        I::RefGet { .. } | I::RefSet { .. } => { /* handled in normalize_ref_field_access pass (guarded) */ }
                        I::ArrayGet { dst, array, index } if array_to_boxcall => {
                            let mid = crate::mir::slot_registry::resolve_slot_by_type_name("ArrayBox", "get");
                            *term = I::BoxCall { dst: Some(*dst), box_val: *array, method: "get".to_string(), method_id: mid, args: vec![*index], effects: EffectMask::READ };
                        }
                        I::ArraySet { array, index, value } if array_to_boxcall => {
                            let mid = crate::mir::slot_registry::resolve_slot_by_type_name("ArrayBox", "set");
                            *term = I::BoxCall { dst: None, box_val: *array, method: "set".to_string(), method_id: mid, args: vec![*index, *value], effects: EffectMask::WRITE };
                        }
                        I::PluginInvoke { dst, box_val, method, args, effects } => {
                            *term = I::BoxCall { dst: *dst, box_val: *box_val, method: method.clone(), method_id: None, args: args.clone(), effects: *effects };
                        }
                        I::Debug { value, .. } if rw_dbg => {
                            let v = *value;
                            *term = I::ExternCall { dst: None, iface_name: "env.debug".to_string(), method_name: "trace".to_string(), args: vec![v], effects: EffectMask::PURE.add(Effect::Debug) };
                        }
                        I::Safepoint if rw_sp => {
                            *term = I::ExternCall { dst: None, iface_name: "env.runtime".to_string(), method_name: "checkpoint".to_string(), args: vec![], effects: EffectMask::PURE };
                        }
                        // Future/Await （終端側）
                        I::FutureNew { dst, value } if rw_future => {
                            let d = *dst; let v = *value;
                            *term = I::ExternCall { dst: Some(d), iface_name: "env.future".to_string(), method_name: "new".to_string(), args: vec![v], effects: EffectMask::PURE.add(Effect::Io) };
                        }
                        I::FutureSet { future, value } if rw_future => {
                            let f = *future; let v = *value;
                            *term = I::ExternCall { dst: None, iface_name: "env.future".to_string(), method_name: "set".to_string(), args: vec![f, v], effects: EffectMask::PURE.add(Effect::Io) };
                        }
                        I::Await { dst, future } if rw_future => {
                            let d = *dst; let f = *future;
                            *term = I::ExternCall { dst: Some(d), iface_name: "env.future".to_string(), method_name: "await".to_string(), args: vec![f], effects: EffectMask::PURE.add(Effect::Io) };
                        }
                        _ => {}
                    }
                }
            }
        }
        stats
    }

    /// Normalize RefGet/RefSet to BoxCall("getField"/"setField") with Const String field argument.
    fn normalize_ref_field_access(&mut self, module: &mut MirModule) -> OptimizationStats {
        use super::{MirInstruction as I, BarrierOp};
        let mut stats = OptimizationStats::new();
        for (_fname, function) in &mut module.functions {
            for (_bb, block) in &mut function.blocks {
                let mut out: Vec<I> = Vec::with_capacity(block.instructions.len() + 2);
                let old = std::mem::take(&mut block.instructions);
                for inst in old.into_iter() {
                    match inst {
                        I::RefGet { dst, reference, field } => {
                            let new_id = super::ValueId::new(function.next_value_id);
                            function.next_value_id += 1;
                            out.push(I::Const { dst: new_id, value: super::instruction::ConstValue::String(field) });
                            out.push(I::BoxCall { dst: Some(dst), box_val: reference, method: "getField".to_string(), method_id: None, args: vec![new_id], effects: super::EffectMask::READ });
                            stats.intrinsic_optimizations += 1;
                        }
                        I::RefSet { reference, field, value } => {
                            let new_id = super::ValueId::new(function.next_value_id);
                            function.next_value_id += 1;
                            out.push(I::Const { dst: new_id, value: super::instruction::ConstValue::String(field) });
                            // Prepend an explicit write barrier before setField to make side-effects visible
                            out.push(I::Barrier { op: BarrierOp::Write, ptr: reference });
                            out.push(I::BoxCall { dst: None, box_val: reference, method: "setField".to_string(), method_id: None, args: vec![new_id, value], effects: super::EffectMask::WRITE });
                            stats.intrinsic_optimizations += 1;
                        }
                        other => out.push(other),
                    }
                }
                block.instructions = out;

                if let Some(term) = block.terminator.take() {
                    block.terminator = Some(match term {
                        I::RefGet { dst, reference, field } => {
                            let new_id = super::ValueId::new(function.next_value_id);
                            function.next_value_id += 1;
                            block.instructions.push(I::Const { dst: new_id, value: super::instruction::ConstValue::String(field) });
                            I::BoxCall { dst: Some(dst), box_val: reference, method: "getField".to_string(), method_id: None, args: vec![new_id], effects: super::EffectMask::READ }
                        }
                        I::RefSet { reference, field, value } => {
                            let new_id = super::ValueId::new(function.next_value_id);
                            function.next_value_id += 1;
                            block.instructions.push(I::Const { dst: new_id, value: super::instruction::ConstValue::String(field) });
                            block.instructions.push(I::Barrier { op: BarrierOp::Write, ptr: reference });
                            I::BoxCall { dst: None, box_val: reference, method: "setField".to_string(), method_id: None, args: vec![new_id, value], effects: super::EffectMask::WRITE }
                        }
                        other => other,
                    });
                }
            }
        }
        stats
    }
}

/// Map string type name to MIR type (optimizer-level helper)
fn map_type_name(name: &str) -> MirType {
    match name {
        "Integer" | "Int" | "I64" => MirType::Integer,
        "Float" | "F64" => MirType::Float,
        "Bool" | "Boolean" => MirType::Bool,
        "String" => MirType::String,
        "Void" | "Unit" => MirType::Void,
        other => MirType::Box(other.to_string()),
    }
}

fn opt_debug_enabled() -> bool { crate::config::env::opt_debug() }
fn opt_debug(msg: &str) { if opt_debug_enabled() { eprintln!("[OPT] {}", msg); } }

/// Resolve a MIR type from a value id that should represent a type name
/// Supports: Const String("T") and NewBox(StringBox, Const String("T"))
fn resolve_type_from_value(
    function: &MirFunction,
    def_map: &std::collections::HashMap<ValueId, (super::basic_block::BasicBlockId, usize)>,
    id: ValueId,
) -> Option<MirType> {
    use super::instruction::ConstValue;
    if let Some((bb, idx)) = def_map.get(&id).copied() {
        if let Some(block) = function.blocks.get(&bb) {
            if idx < block.instructions.len() {
                match &block.instructions[idx] {
                    MirInstruction::Const { value: ConstValue::String(s), .. } => {
                        return Some(map_type_name(s));
                    }
                    MirInstruction::NewBox { box_type, args, .. } if box_type == "StringBox" && args.len() == 1 => {
                        let inner = args[0];
                        if let Some((sbb, sidx)) = def_map.get(&inner).copied() {
                            if let Some(sblock) = function.blocks.get(&sbb) {
                                if sidx < sblock.instructions.len() {
                                    if let MirInstruction::Const { value: ConstValue::String(s), .. } = &sblock.instructions[sidx] {
                                        return Some(map_type_name(s));
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    None
}

impl Default for MirOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

// OptimizationStats moved to crate::mir::optimizer_stats

impl MirOptimizer {
    /// Diagnostic: detect unlowered is/as/isType/asType after Builder
    fn diagnose_unlowered_type_ops(&mut self, module: &MirModule) -> OptimizationStats {
        let mut stats = OptimizationStats::new();
        let diag_on = self.debug || crate::config::env::opt_diag();
        for (fname, function) in &module.functions {
            // def map for resolving constants
            let mut def_map: std::collections::HashMap<ValueId, (super::basic_block::BasicBlockId, usize)> = std::collections::HashMap::new();
            for (bb_id, block) in &function.blocks {
                for (i, inst) in block.instructions.iter().enumerate() {
                    if let Some(dst) = inst.dst_value() { def_map.insert(dst, (*bb_id, i)); }
                }
                if let Some(term) = &block.terminator { if let Some(dst) = term.dst_value() { def_map.insert(dst, (*bb_id, usize::MAX)); } }
            }
            let mut count = 0usize;
            for (_bb, block) in &function.blocks {
                for inst in &block.instructions {
                    match inst {
                        MirInstruction::BoxCall { method, .. } if method == "is" || method == "as" || method == "isType" || method == "asType" => { count += 1; }
                        MirInstruction::Call { func, .. } => {
                            if let Some((bb, idx)) = def_map.get(func).copied() {
                                if let Some(b) = function.blocks.get(&bb) {
                                    if idx < b.instructions.len() {
                                        if let MirInstruction::Const { value: super::instruction::ConstValue::String(s), .. } = &b.instructions[idx] {
                                            if s == "isType" || s == "asType" { count += 1; }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            if count > 0 {
                stats.diagnostics_reported += count;
                if diag_on {
                    eprintln!("[OPT][DIAG] Function '{}' has {} unlowered type-op calls", fname, count);
                }
            }
        }
        stats
    }

    /// Diagnostic: detect legacy instructions that should be unified
    /// Legacy set: TypeCheck/Cast/WeakNew/WeakLoad/BarrierRead/BarrierWrite/ArrayGet/ArraySet/RefGet/RefSet/PluginInvoke
    /// When NYASH_OPT_DIAG or NYASH_OPT_DIAG_FORBID_LEGACY is set, prints diagnostics.
    fn diagnose_legacy_instructions(&mut self, module: &MirModule) -> OptimizationStats {
        let mut stats = OptimizationStats::new();
        let diag_on = self.debug
            || crate::config::env::opt_diag()
            || crate::config::env::opt_diag_forbid_legacy();
        for (fname, function) in &module.functions {
            let mut count = 0usize;
            for (_bb, block) in &function.blocks {
                for inst in &block.instructions {
                    match inst {
                        MirInstruction::TypeCheck { .. }
                        | MirInstruction::Cast { .. }
                        | MirInstruction::WeakNew { .. }
                        | MirInstruction::WeakLoad { .. }
                        | MirInstruction::BarrierRead { .. }
                        | MirInstruction::BarrierWrite { .. }
                        | MirInstruction::ArrayGet { .. }
                        | MirInstruction::ArraySet { .. }
                        | MirInstruction::RefGet { .. }
                        | MirInstruction::RefSet { .. }
                        | MirInstruction::PluginInvoke { .. } => { count += 1; }
                        _ => {}
                    }
                }
                if let Some(term) = &block.terminator {
                    match term {
                        MirInstruction::TypeCheck { .. }
                        | MirInstruction::Cast { .. }
                        | MirInstruction::WeakNew { .. }
                        | MirInstruction::WeakLoad { .. }
                        | MirInstruction::BarrierRead { .. }
                        | MirInstruction::BarrierWrite { .. }
                        | MirInstruction::ArrayGet { .. }
                        | MirInstruction::ArraySet { .. }
                        | MirInstruction::RefGet { .. }
                        | MirInstruction::RefSet { .. }
                        | MirInstruction::PluginInvoke { .. } => { count += 1; }
                        _ => {}
                    }
                }
            }
            if count > 0 {
                stats.diagnostics_reported += count;
                if diag_on {
                    eprintln!(
                        "[OPT][DIAG] Function '{}' has {} legacy MIR ops: unify to Core‑13 (TypeOp/WeakRef/Barrier/BoxCall)",
                        fname, count
                    );
                    if crate::config::env::opt_diag_forbid_legacy() {
                        panic!("NYASH_OPT_DIAG_FORBID_LEGACY=1: legacy MIR ops detected in '{}': {}", fname, count);
                    }
                }
            }
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{MirModule, MirFunction, FunctionSignature, MirType, TypeOpKind, BasicBlock, BasicBlockId, ValueId, ConstValue};
    
    #[test]
    fn test_optimizer_creation() {
        let optimizer = MirOptimizer::new();
        assert!(!optimizer.debug);
        
        let debug_optimizer = MirOptimizer::new().with_debug();
        assert!(debug_optimizer.debug);
    }
    
    #[test]
    fn test_optimization_stats() {
        let mut stats = OptimizationStats::new();
        assert_eq!(stats.total_optimizations(), 0);
        
        stats.dead_code_eliminated = 5;
        stats.cse_eliminated = 3;
        assert_eq!(stats.total_optimizations(), 8);
        
        let other_stats = OptimizationStats {
            dead_code_eliminated: 2,
            cse_eliminated: 1,
            ..Default::default()
        };
        
        stats.merge(other_stats);
        assert_eq!(stats.dead_code_eliminated, 7);
        assert_eq!(stats.cse_eliminated, 4);
        assert_eq!(stats.total_optimizations(), 11);
    }
    
    #[test]
    fn test_instruction_to_key() {
        let optimizer = MirOptimizer::new();
        
        let const_instr = MirInstruction::Const {
            dst: ValueId::new(1),
            value: ConstValue::Integer(42),
        };
        
        let key = optimizer.instruction_to_key(&const_instr);
        assert!(key.contains("const"));
        assert!(key.contains("42"));
    }

    #[test]
    fn test_dce_does_not_drop_typeop_used_by_console_log() {
        // Build: %v=TypeOp(check); extern_call env.console.log(%v); ensure TypeOp remains after optimize
        let signature = FunctionSignature {
            name: "main".to_string(),
            params: vec![],
            return_type: MirType::Void,
            effects: super::super::effect::EffectMask::PURE,
        };
        let mut func = MirFunction::new(signature, BasicBlockId::new(0));
        let bb0 = BasicBlockId::new(0);
        let mut b0 = BasicBlock::new(bb0);
        let v0 = ValueId::new(0);
        let v1 = ValueId::new(1);
        b0.add_instruction(MirInstruction::NewBox { dst: v0, box_type: "IntegerBox".to_string(), args: vec![] });
        b0.add_instruction(MirInstruction::TypeOp { dst: v1, op: TypeOpKind::Check, value: v0, ty: MirType::Integer });
        b0.add_instruction(MirInstruction::ExternCall { dst: None, iface_name: "env.console".to_string(), method_name: "log".to_string(), args: vec![v1], effects: super::super::effect::EffectMask::IO });
        b0.add_instruction(MirInstruction::Return { value: None });
        func.add_block(b0);
        let mut module = MirModule::new("test".to_string());
        module.add_function(func);

        let mut opt = MirOptimizer::new();
        let _stats = opt.optimize_module(&mut module);

        // Ensure TypeOp remains in bb0
        let f = module.get_function("main").unwrap();
        let block = f.get_block(bb0).unwrap();
        let has_typeop = block.all_instructions().any(|i| matches!(i, MirInstruction::TypeOp { .. }));
        assert!(has_typeop, "TypeOp should not be dropped by DCE when used by console.log (ExternCall)");
    }
}
