/*!
 * MIR Builder - Converts AST to MIR/SSA form
 * 
 * Implements AST → MIR conversion with SSA construction
 */

use super::{
    MirInstruction, BasicBlock, BasicBlockId, MirFunction, MirModule, 
    FunctionSignature, ValueId, ConstValue, BinaryOp, UnaryOp, CompareOp,
    MirType, EffectMask, Effect, BasicBlockIdGenerator, ValueIdGenerator
};
use super::slot_registry::{get_or_assign_type_id, reserve_method_slot};
use super::slot_registry::resolve_slot_by_type_name;
use crate::ast::{ASTNode, LiteralValue, BinaryOperator};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
mod builder_calls;
mod stmts;
mod ops;
mod utils;
mod exprs; // expression lowering split
mod decls; // declarations lowering split
mod fields; // field access/assignment lowering split
mod exprs_call;    // call(expr)
mod exprs_qmark;   // ?-propagate
mod exprs_peek;    // peek expression
mod exprs_lambda;  // lambda lowering
mod exprs_include; // include lowering
mod plugin_sigs; // plugin signature loader

// moved helpers to builder/utils.rs

/// MIR builder for converting AST to SSA form
pub struct MirBuilder {
    /// Current module being built
    pub(super) current_module: Option<MirModule>,
    
    /// Current function being built
    pub(super) current_function: Option<MirFunction>,
    
    /// Current basic block being built
    pub(super) current_block: Option<BasicBlockId>,
    
    /// Value ID generator
    pub(super) value_gen: ValueIdGenerator,
    
    /// Basic block ID generator
    pub(super) block_gen: BasicBlockIdGenerator,
    
    /// Variable name to ValueId mapping (for SSA conversion)
    pub(super) variable_map: HashMap<String, ValueId>,
    
    /// Pending phi functions to be inserted
    #[allow(dead_code)]
    pub(super) pending_phis: Vec<(BasicBlockId, ValueId, String)>,

    /// Origin tracking for simple optimizations (e.g., object.method after new)
    /// Maps a ValueId to the class name if it was produced by NewBox of that class
    pub(super) value_origin_newbox: HashMap<ValueId, String>,

    /// Names of user-defined boxes declared in the current module
    pub(super) user_defined_boxes: HashSet<String>,

    /// Weak field registry: BoxName -> {weak field names}
    pub(super) weak_fields_by_box: HashMap<String, HashSet<String>>,

    /// Remember class of object fields after assignments: (base_id, field) -> class_name
    pub(super) field_origin_class: HashMap<(ValueId, String), String>,

    /// Optional per-value type annotations (MIR-level): ValueId -> MirType
    pub(super) value_types: HashMap<ValueId, super::MirType>,

    /// Plugin method return type signatures loaded from nyash_box.toml
    plugin_method_sigs: HashMap<(String, String), super::MirType>,
    /// Current static box name when lowering a static box body (e.g., "Main")
    current_static_box: Option<String>,

    /// Include guards: currently loading file canonical paths
    include_loading: HashSet<String>,
    /// Include visited cache: canonical path -> box name
    include_box_map: HashMap<String, String>,

    /// Loop context stacks for lowering break/continue inside nested control flow
    /// Top of stack corresponds to the innermost active loop
    pub(super) loop_header_stack: Vec<BasicBlockId>,
    pub(super) loop_exit_stack: Vec<BasicBlockId>,
}

impl MirBuilder {
    /// Emit a Box method call (unified: BoxCall)
    fn emit_box_or_plugin_call(
        &mut self,
        dst: Option<ValueId>,
        box_val: ValueId,
        method: String,
        method_id: Option<u16>,
        args: Vec<ValueId>,
        effects: EffectMask,
    ) -> Result<(), String> {
        // Emit instruction first
        self.emit_instruction(MirInstruction::BoxCall { dst, box_val, method: method.clone(), method_id, args, effects })?;
        // Heuristic return type inference for common builtin box methods
        if let Some(d) = dst {
            // Try to infer receiver box type from NewBox origin; fallback to current value_types
            let mut recv_box: Option<String> = self.value_origin_newbox.get(&box_val).cloned();
            if recv_box.is_none() {
                if let Some(t) = self.value_types.get(&box_val) {
                    match t {
                        super::MirType::String => recv_box = Some("StringBox".to_string()),
                        super::MirType::Box(name) => recv_box = Some(name.clone()),
                        _ => {}
                    }
                }
            }
            if let Some(bt) = recv_box {
                if let Some(mt) = self.plugin_method_sigs.get(&(bt.clone(), method.clone())) {
                    self.value_types.insert(d, mt.clone());
                } else {
                    let inferred: Option<super::MirType> = match (bt.as_str(), method.as_str()) {
                        // Built-in box methods
                        ("StringBox", "length") | ("StringBox", "len") => Some(super::MirType::Integer),
                        ("StringBox", "is_empty") => Some(super::MirType::Bool),
                        ("StringBox", "charCodeAt") => Some(super::MirType::Integer),
                        // String-producing methods (important for LLVM ret handling)
                        ("StringBox", "substring")
                        | ("StringBox", "concat")
                        | ("StringBox", "replace")
                        | ("StringBox", "trim")
                        | ("StringBox", "toUpper")
                        | ("StringBox", "toLower") => Some(super::MirType::String),
                        ("ArrayBox", "length") => Some(super::MirType::Integer),
                        // Core MapBox minimal inference (core-first)
                        ("MapBox", "size") => Some(super::MirType::Integer),
                        ("MapBox", "has") => Some(super::MirType::Bool),
                        ("MapBox", "get") => Some(super::MirType::Box("Any".to_string())),
                        _ => None,
                    };
                    if let Some(mt) = inferred {
                        self.value_types.insert(d, mt);
                    }
                }
            }
        }
        Ok(())
    }
    /// Create a new MIR builder
    pub fn new() -> Self {
        let plugin_method_sigs = plugin_sigs::load_plugin_method_sigs();
        Self {
            current_module: None,
            current_function: None,
            current_block: None,
            value_gen: ValueIdGenerator::new(),
            block_gen: BasicBlockIdGenerator::new(),
            variable_map: HashMap::new(),
            pending_phis: Vec::new(),
            value_origin_newbox: HashMap::new(),
            user_defined_boxes: HashSet::new(),
            weak_fields_by_box: HashMap::new(),
            field_origin_class: HashMap::new(),
            value_types: HashMap::new(),
            plugin_method_sigs,
            current_static_box: None,
            include_loading: HashSet::new(),
            include_box_map: HashMap::new(),
            loop_header_stack: Vec::new(),
            loop_exit_stack: Vec::new(),
        }
    }

    /// Emit a type check instruction (Unified: TypeOp(Check))
    #[allow(dead_code)]
    pub(super) fn emit_type_check(&mut self, value: ValueId, expected_type: String) -> Result<ValueId, String> {
        let dst = self.value_gen.next();
        self.emit_instruction(MirInstruction::TypeOp { dst, op: super::TypeOpKind::Check, value, ty: super::MirType::Box(expected_type) })?;
        Ok(dst)
    }

    /// Emit a cast instruction (Unified: TypeOp(Cast))
    #[allow(dead_code)]
    pub(super) fn emit_cast(&mut self, value: ValueId, target_type: super::MirType) -> Result<ValueId, String> {
        let dst = self.value_gen.next();
        self.emit_instruction(MirInstruction::TypeOp { dst, op: super::TypeOpKind::Cast, value, ty: target_type.clone() })?;
        Ok(dst)
    }

    /// Emit a weak reference creation (Unified: WeakRef(New))
    #[allow(dead_code)]
    pub(super) fn emit_weak_new(&mut self, box_val: ValueId) -> Result<ValueId, String> {
        if crate::config::env::mir_core13_pure() {
            // Pure mode: avoid WeakRef emission; pass-through
            return Ok(box_val);
        }
        let dst = self.value_gen.next();
        self.emit_instruction(MirInstruction::WeakRef { dst, op: super::WeakRefOp::New, value: box_val })?;
        Ok(dst)
    }

    /// Emit a weak reference load (Unified: WeakRef(Load))
    #[allow(dead_code)]
    pub(super) fn emit_weak_load(&mut self, weak_ref: ValueId) -> Result<ValueId, String> {
        if crate::config::env::mir_core13_pure() {
            // Pure mode: avoid WeakRef emission; pass-through
            return Ok(weak_ref);
        }
        let dst = self.value_gen.next();
        self.emit_instruction(MirInstruction::WeakRef { dst, op: super::WeakRefOp::Load, value: weak_ref })?;
        Ok(dst)
    }

    /// Emit a barrier read (Unified: Barrier(Read))
    #[allow(dead_code)]
    pub(super) fn emit_barrier_read(&mut self, ptr: ValueId) -> Result<(), String> {
        self.emit_instruction(MirInstruction::Barrier { op: super::BarrierOp::Read, ptr })
    }

    /// Emit a barrier write (Unified: Barrier(Write))
    #[allow(dead_code)]
    pub(super) fn emit_barrier_write(&mut self, ptr: ValueId) -> Result<(), String> {
        self.emit_instruction(MirInstruction::Barrier { op: super::BarrierOp::Write, ptr })
    }

    // moved to builder_calls.rs: lower_method_as_function
    
    /// Build a complete MIR module from AST
    pub fn build_module(&mut self, ast: ASTNode) -> Result<MirModule, String> {
        // Create a new module
        let module = MirModule::new("main".to_string());
        
        // Create a main function to contain the AST
        let main_signature = FunctionSignature {
            name: "main".to_string(),
            params: vec![],
            return_type: MirType::Void,
            effects: EffectMask::PURE,
        };
        
        let entry_block = self.block_gen.next();
        let mut main_function = MirFunction::new(main_signature, entry_block);
        main_function.metadata.is_entry_point = true;
        
        // Set up building context
        self.current_module = Some(module);
        self.current_function = Some(main_function);
        self.current_block = Some(entry_block);
        
        // Optional: Add safepoint at function entry (disabled by default)
        if std::env::var("NYASH_BUILDER_SAFEPOINT_ENTRY").ok().as_deref() == Some("1") {
            self.emit_instruction(MirInstruction::Safepoint)?;
        }
        
        // Convert AST to MIR
        let result_value = self.build_expression(ast)?;
        
        // Add return instruction if needed
        if let Some(block_id) = self.current_block {
            if let Some(ref mut function) = self.current_function {
                if let Some(block) = function.get_block_mut(block_id) {
                    if !block.is_terminated() {
                        block.add_instruction(MirInstruction::Return {
                            value: Some(result_value),
                        });
                    }
                    // Infer return type from TyEnv (value_types)
                    if let Some(mt) = self.value_types.get(&result_value).cloned() {
                        function.signature.return_type = mt;
                    }
                }
            }
        }
        
        // Finalize and return module
        let mut module = self.current_module.take().unwrap();
        let mut function = self.current_function.take().unwrap();
        // Flush value_types (TyEnv) into function metadata
        function.metadata.value_types = self.value_types.clone();
        // 補助: 本文中に明示的な return <expr> が存在する場合、
        // 末尾returnを挿入しなかった関数でも戻り型を推定する。
        if matches!(function.signature.return_type, super::MirType::Void | super::MirType::Unknown) {
            let mut inferred: Option<super::MirType> = None;
            'outer: for (_bid, bb) in function.blocks.iter() {
                for inst in bb.instructions.iter() {
                    if let super::MirInstruction::Return { value: Some(v) } = inst {
                        if let Some(mt) = self.value_types.get(v).cloned() { inferred = Some(mt); break 'outer; }
                        // 追加: v が PHI の場合は入力側の型から推定
                if let Some(mt) = utils::infer_type_from_phi(&function, *v, &self.value_types) { inferred = Some(mt); break 'outer; }
                    }
                }
                if let Some(super::MirInstruction::Return { value: Some(v) }) = &bb.terminator {
                    if let Some(mt) = self.value_types.get(v).cloned() { inferred = Some(mt); break; }
                    if let Some(mt) = utils::infer_type_from_phi(&function, *v, &self.value_types) { inferred = Some(mt); break; }
                }
            }
            if let Some(mt) = inferred { function.signature.return_type = mt; }
        }
        module.add_function(function);

        
        Ok(module)
    }
    
    /// Build an expression and return its value ID
    pub(super) fn build_expression(&mut self, ast: ASTNode) -> Result<ValueId, String> {
        // Delegated to exprs.rs to keep this file lean
        self.build_expression_impl(ast)
    }
    
    // Moved implementation to exprs.rs; keeping a small shim here improves readability
    pub(super) fn build_expression_impl_legacy(&mut self, ast: ASTNode) -> Result<ValueId, String> {
        match ast {
            ASTNode::Literal { value, .. } => {
                self.build_literal(value)
            },
            
            ASTNode::BinaryOp { left, operator, right, .. } => {
                self.build_binary_op(*left, operator, *right)
            },
            
            ASTNode::UnaryOp { operator, operand, .. } => {
                let op_string = match operator {
                    crate::ast::UnaryOperator::Minus => "-".to_string(),
                    crate::ast::UnaryOperator::Not => "not".to_string(),
                };
                self.build_unary_op(op_string, *operand)
            },
            
            ASTNode::Variable { name, .. } => {
                self.build_variable_access(name.clone())
            },
            
            ASTNode::Me { .. } => {
                self.build_me_expression()
            },
            
            ASTNode::MethodCall { object, method, arguments, .. } => {
                // Early TypeOp lowering for method-style is()/as()
                if (method == "is" || method == "as") && arguments.len() == 1 {
                    if let Some(type_name) = Self::extract_string_literal(&arguments[0]) {
                        let obj_val = self.build_expression(*object.clone())?;
                        let ty = Self::parse_type_name_to_mir(&type_name);
                        let dst = self.value_gen.next();
                        let op = if method == "is" { super::TypeOpKind::Check } else { super::TypeOpKind::Cast };
                        self.emit_instruction(MirInstruction::TypeOp { dst, op, value: obj_val, ty })?;
                        return Ok(dst);
                    }
                }
                self.build_method_call(*object.clone(), method.clone(), arguments.clone())
            },
            
            ASTNode::FromCall { parent, method, arguments, .. } => {
                self.build_from_expression(parent.clone(), method.clone(), arguments.clone())
            },
            
            ASTNode::Assignment { target, value, .. } => {
                // Check if target is a field access for RefSet
                if let ASTNode::FieldAccess { object, field, .. } = target.as_ref() {
                    self.build_field_assignment(*object.clone(), field.clone(), *value.clone())
                } else if let ASTNode::Variable { name, .. } = target.as_ref() {
                    // Plain variable assignment - existing behavior
                    self.build_assignment(name.clone(), *value.clone())
                } else {
                    Err("Complex assignment targets not yet supported in MIR".to_string())
                }
            },
            
            ASTNode::FunctionCall { name, arguments, .. } => {
                // Early TypeOp lowering for function-style isType()/asType()
                if (name == "isType" || name == "asType") && arguments.len() == 2 {
                    if let Some(type_name) = Self::extract_string_literal(&arguments[1]) {
                        let val = self.build_expression(arguments[0].clone())?;
                        let ty = Self::parse_type_name_to_mir(&type_name);
                        let dst = self.value_gen.next();
                        let op = if name == "isType" { super::TypeOpKind::Check } else { super::TypeOpKind::Cast };
                        self.emit_instruction(MirInstruction::TypeOp { dst, op, value: val, ty })?;
                        return Ok(dst);
                    }
                }
                self.build_function_call(name.clone(), arguments.clone())
            },
            ASTNode::Call { callee, arguments, .. } => {
                // P1.5: Lambdaはインライン、それ以外は Call に正規化
                if let ASTNode::Lambda { params, body, .. } = callee.as_ref() {
                    if params.len() != arguments.len() {
                        return Err(format!("Lambda expects {} args, got {}", params.len(), arguments.len()));
                    }
                    let mut arg_vals: Vec<ValueId> = Vec::new();
                    for a in arguments { arg_vals.push(self.build_expression(a)?); }
                    let saved_vars = self.variable_map.clone();
                    for (p, v) in params.iter().zip(arg_vals.iter()) { self.variable_map.insert(p.clone(), *v); }
                    let prog = ASTNode::Program { statements: body.clone(), span: crate::ast::Span::unknown() };
                    let out = self.build_expression(prog)?;
                    self.variable_map = saved_vars;
                    Ok(out)
                } else {
                    // callee/args を評価し、Call を発行（VM 側で FunctionBox/関数名の両対応）
                    let callee_id = self.build_expression(*callee.clone())?;
                    let mut arg_ids = Vec::new();
                    for a in arguments { arg_ids.push(self.build_expression(a)?); }
                    let dst = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Call { dst: Some(dst), func: callee_id, args: arg_ids, effects: EffectMask::PURE })?;
                    Ok(dst)
                }
            },
            
            ASTNode::QMarkPropagate { expression, .. } => {
                // Lower: ok = expr.isOk(); br ok then else; else => return expr; then => expr.getValue()
                let res_val = self.build_expression(*expression.clone())?;
                let ok_id = self.value_gen.next();
                self.emit_instruction(MirInstruction::BoxCall { dst: Some(ok_id), box_val: res_val, method: "isOk".to_string(), method_id: None, args: vec![], effects: EffectMask::PURE })?;
                let then_block = self.block_gen.next();
                let else_block = self.block_gen.next();
                self.emit_instruction(MirInstruction::Branch { condition: ok_id, then_bb: then_block, else_bb: else_block })?;
                // else: return res_val
                self.current_block = Some(else_block);
                self.ensure_block_exists(else_block)?;
                self.emit_instruction(MirInstruction::Return { value: Some(res_val) })?;
                // then: getValue()
                self.current_block = Some(then_block);
                self.ensure_block_exists(then_block)?;
                let val_id = self.value_gen.next();
                self.emit_instruction(MirInstruction::BoxCall { dst: Some(val_id), box_val: res_val, method: "getValue".to_string(), method_id: None, args: vec![], effects: EffectMask::PURE })?;
                self.value_types.insert(val_id, super::MirType::Unknown);
                Ok(val_id)
            },
            
            ASTNode::Print { expression, .. } => {
                self.build_print_statement(*expression.clone())
            },
            
            ASTNode::Program { statements, .. } => {
                self.build_block(statements.clone())
            },
            
            ASTNode::If { condition, then_body, else_body, .. } => {
                let else_ast = if let Some(else_statements) = else_body {
                    Some(ASTNode::Program {
                        statements: else_statements.clone(),
                        span: crate::ast::Span::unknown(),
                    })
                } else {
                    None
                };
                
                self.build_if_statement(
                    *condition.clone(), 
                    ASTNode::Program {
                        statements: then_body.clone(),
                        span: crate::ast::Span::unknown(),
                    },
                    else_ast
                )
            },
            
            ASTNode::Loop { condition, body, .. } => {
                self.build_loop_statement(*condition.clone(), body.clone())
            },
            
            ASTNode::TryCatch { try_body, catch_clauses, finally_body, .. } => {
                self.build_try_catch_statement(try_body.clone(), catch_clauses.clone(), finally_body.clone())
            },
            
            ASTNode::Throw { expression, .. } => {
                self.build_throw_statement(*expression.clone())
            },

            // P1: Lower peek expression into if-else chain with phi
            ASTNode::PeekExpr { scrutinee, arms, else_expr, .. } => {
                // Evaluate scrutinee once
                let scr_val = self.build_expression(*scrutinee.clone())?;

                // Prepare a merge block and collect phi inputs
                let merge_block = self.block_gen.next();
                let mut phi_inputs: Vec<(super::BasicBlockId, super::ValueId)> = Vec::new();

                // Start chaining from the current block
                for (lit, arm_expr) in arms.into_iter() {
                    // Build condition: scr_val == lit
                    let lit_id = self.build_literal(lit)?;
                    let cond_id = self.value_gen.next();
                    self.emit_instruction(super::MirInstruction::Compare { dst: cond_id, op: super::CompareOp::Eq, lhs: scr_val, rhs: lit_id })?;

                    // Create then and next blocks
                    let then_block = self.block_gen.next();
                    let next_block = self.block_gen.next();
                    self.emit_instruction(super::MirInstruction::Branch { condition: cond_id, then_bb: then_block, else_bb: next_block })?;

                    // then: evaluate arm expr, jump to merge
                    self.current_block = Some(then_block);
                    self.ensure_block_exists(then_block)?;
                    let then_val = self.build_expression(arm_expr)?;
                    if !self.is_current_block_terminated() {
                        self.emit_instruction(super::MirInstruction::Jump { target: merge_block })?;
                    }
                    phi_inputs.push((then_block, then_val));

                    // else path continues chaining
                    self.current_block = Some(next_block);
                    self.ensure_block_exists(next_block)?;
                    // Loop continues from next_block
                }

                // Final else branch
                let cur_block = self.current_block.ok_or("No current basic block")?;
                let else_val = self.build_expression(*else_expr.clone())?;
                if !self.is_current_block_terminated() {
                    self.emit_instruction(super::MirInstruction::Jump { target: merge_block })?;
                }
                phi_inputs.push((cur_block, else_val));

                // Merge and phi
                self.current_block = Some(merge_block);
                self.ensure_block_exists(merge_block)?;
                let result_val = self.value_gen.next();
                self.emit_instruction(super::MirInstruction::Phi { dst: result_val, inputs: phi_inputs })?;
                Ok(result_val)
            },
            
            ASTNode::Lambda { params, body, .. } => {
                // Lambda→FunctionBox 値 Lower（最小 + 簡易キャプチャ解析）
                let dst = self.value_gen.next();
                // Collect free variable names: variables used in body but not in params, and not 'me'/'this'
                use std::collections::HashSet;
                let mut used: HashSet<String> = HashSet::new();
                let mut locals: HashSet<String> = HashSet::new();
                for p in params.iter() { locals.insert(p.clone()); }
                fn collect_vars(node: &crate::ast::ASTNode, used: &mut HashSet<String>, locals: &mut HashSet<String>) {
                    match node {
                        crate::ast::ASTNode::Variable { name, .. } => {
                            if name != "me" && name != "this" && !locals.contains(name) {
                                used.insert(name.clone());
                            }
                        }
                        crate::ast::ASTNode::Local { variables, .. } => { for v in variables { locals.insert(v.clone()); } }
                        crate::ast::ASTNode::Assignment { target, value, .. } => { collect_vars(target, used, locals); collect_vars(value, used, locals); }
                        crate::ast::ASTNode::BinaryOp { left, right, .. } => { collect_vars(left, used, locals); collect_vars(right, used, locals); }
                        crate::ast::ASTNode::UnaryOp { operand, .. } => { collect_vars(operand, used, locals); }
                        crate::ast::ASTNode::MethodCall { object, arguments, .. } => { collect_vars(object, used, locals); for a in arguments { collect_vars(a, used, locals); } }
                        crate::ast::ASTNode::FunctionCall { arguments, .. } => { for a in arguments { collect_vars(a, used, locals); } }
                        crate::ast::ASTNode::Call { callee, arguments, .. } => { collect_vars(callee, used, locals); for a in arguments { collect_vars(a, used, locals); } }
                        crate::ast::ASTNode::FieldAccess { object, .. } => { collect_vars(object, used, locals); }
                        crate::ast::ASTNode::New { arguments, .. } => { for a in arguments { collect_vars(a, used, locals); } }
                        crate::ast::ASTNode::If { condition, then_body, else_body, .. } => {
                            collect_vars(condition, used, locals);
                            for st in then_body { collect_vars(st, used, locals); }
                            if let Some(eb) = else_body { for st in eb { collect_vars(st, used, locals); } }
                        }
                        crate::ast::ASTNode::Loop { condition, body, .. } => { collect_vars(condition, used, locals); for st in body { collect_vars(st, used, locals); } }
                        crate::ast::ASTNode::TryCatch { try_body, catch_clauses, finally_body, .. } => {
                            for st in try_body { collect_vars(st, used, locals); }
                            for c in catch_clauses { for st in &c.body { collect_vars(st, used, locals); } }
                            if let Some(fb) = finally_body { for st in fb { collect_vars(st, used, locals); } }
                        }
                        crate::ast::ASTNode::Throw { expression, .. } => { collect_vars(expression, used, locals); }
                        crate::ast::ASTNode::Print { expression, .. } => { collect_vars(expression, used, locals); }
                        crate::ast::ASTNode::Return { value, .. } => { if let Some(v) = value { collect_vars(v, used, locals); } }
                        crate::ast::ASTNode::AwaitExpression { expression, .. } => { collect_vars(expression, used, locals); }
                        crate::ast::ASTNode::PeekExpr { scrutinee, arms, else_expr, .. } => {
                            collect_vars(scrutinee, used, locals);
                            for (_, e) in arms { collect_vars(e, used, locals); }
                            collect_vars(else_expr, used, locals);
                        }
                        crate::ast::ASTNode::Program { statements, .. } => { for st in statements { collect_vars(st, used, locals); } }
                        crate::ast::ASTNode::FunctionDeclaration { params, body, .. } => {
                            let mut inner = locals.clone();
                            for p in params { inner.insert(p.clone()); }
                            for st in body { collect_vars(st, used, &mut inner); }
                        }
                        _ => {}
                    }
                }
                for st in body.iter() { collect_vars(st, &mut used, &mut locals); }
                // Materialize captures from current variable_map if known
                let mut captures: Vec<(String, ValueId)> = Vec::new();
                for name in used.into_iter() {
                    if let Some(&vid) = self.variable_map.get(&name) { captures.push((name, vid)); }
                }
                // me capture（存在すれば）
                let me = self.variable_map.get("me").copied();
                self.emit_instruction(MirInstruction::FunctionNew { dst, params: params.clone(), body: body.clone(), captures, me })?;
                self.value_types.insert(dst, super::MirType::Box("FunctionBox".to_string()));
                Ok(dst)
            },
            
            ASTNode::Return { value, .. } => {
                self.build_return_statement(value.clone())
            },
            
            ASTNode::Local { variables, initial_values, .. } => {
                self.build_local_statement(variables.clone(), initial_values.clone())
            },
            
            ASTNode::BoxDeclaration { name, methods, is_static, fields, constructors, weak_fields, .. } => {
                if is_static && name == "Main" {
                    self.build_static_main_box(name.clone(), methods.clone())
                } else {
                    // Support user-defined boxes - handle as statement, return void
                    // Track as user-defined (eligible for method lowering)
                    self.user_defined_boxes.insert(name.clone());
                    self.build_box_declaration(name.clone(), methods.clone(), fields.clone(), weak_fields.clone())?;

                    // Phase 2: Lower constructors (birth/N) into MIR functions
                    // Function name pattern: "{BoxName}.{constructor_key}" (e.g., "Person.birth/1")
                    for (ctor_key, ctor_ast) in constructors.clone() {
                        if let ASTNode::FunctionDeclaration { params, body, .. } = ctor_ast {
                            let func_name = format!("{}.{}", name, ctor_key);
                            self.lower_method_as_function(func_name, name.clone(), params.clone(), body.clone())?;
                        }
                    }

                    // Phase 3: Lower instance methods into MIR functions
                    // Function name pattern: "{BoxName}.{method}/{N}"
                    for (method_name, method_ast) in methods.clone() {
                        if let ASTNode::FunctionDeclaration { params, body, is_static, .. } = method_ast {
                            if !is_static {
                                let func_name = format!("{}.{}{}", name, method_name, format!("/{}", params.len()));
                                self.lower_method_as_function(func_name, name.clone(), params.clone(), body.clone())?;
                            }
                        }
                    }
                    
                    // Return a void value since this is a statement
                    let void_val = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Const {
                        dst: void_val,
                        value: ConstValue::Void,
                    })?;
                    Ok(void_val)
                }
            },
            
            ASTNode::FieldAccess { object, field, .. } => {
                self.build_field_access(*object.clone(), field.clone())
            },
            
            ASTNode::New { class, arguments, .. } => {
                self.build_new_expression(class.clone(), arguments.clone())
            },
            
            // Phase 7: Async operations
            ASTNode::Nowait { variable, expression, .. } => {
                self.build_nowait_statement(variable.clone(), *expression.clone())
            },
            
            ASTNode::AwaitExpression { expression, .. } => {
                self.build_await_expression(*expression.clone())
            },
            
            ASTNode::Include { filename, .. } => {
                // Resolve and read included file
                let mut path = utils::resolve_include_path_builder(&filename);
                if std::path::Path::new(&path).is_dir() {
                    path = format!("{}/index.nyash", path.trim_end_matches('/'));
                } else if std::path::Path::new(&path).extension().is_none() {
                    path.push_str(".nyash");
                }
                // Cycle detection
                if self.include_loading.contains(&path) {
                    return Err(format!("Circular include detected: {}", path));
                }
                // Cache hit: build only the instance
                if let Some(name) = self.include_box_map.get(&path).cloned() {
                    return self.build_new_expression(name, vec![]);
                }
                self.include_loading.insert(path.clone());
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Include read error '{}': {}", filename, e))?;
                // Parse to AST
                let included_ast = crate::parser::NyashParser::parse_from_string(&content)
                    .map_err(|e| format!("Include parse error '{}': {:?}", filename, e))?;
                // Find first static box name
                let mut box_name: Option<String> = None;
                if let crate::ast::ASTNode::Program { statements, .. } = &included_ast {
                    for st in statements {
                        if let crate::ast::ASTNode::BoxDeclaration { name, is_static, .. } = st {
                            if *is_static { box_name = Some(name.clone()); break; }
                        }
                    }
                }
                let bname = box_name.ok_or_else(|| format!("Include target '{}' has no static box", filename))?;
                // Lower included AST into current MIR (register types/methods)
                let _ = self.build_expression(included_ast)?;
                // Mark caches
                self.include_loading.remove(&path);
                self.include_box_map.insert(path.clone(), bname.clone());
                // Return a new instance of included box (no args)
                self.build_new_expression(bname, vec![])
            },
            
            _ => {
                Err(format!("Unsupported AST node type: {:?}", ast))
            }
        }
    }
    
    /// Build a literal value
    pub(super) fn build_literal(&mut self, literal: LiteralValue) -> Result<ValueId, String> {
        // Determine type without moving literal
        let ty_for_dst = match &literal {
            LiteralValue::Integer(_) => Some(super::MirType::Integer),
            LiteralValue::Float(_) => Some(super::MirType::Float),
            LiteralValue::Bool(_) => Some(super::MirType::Bool),
            LiteralValue::String(_) => Some(super::MirType::String),
            _ => None,
        };

        let const_value = match literal {
            LiteralValue::Integer(n) => ConstValue::Integer(n),
            LiteralValue::Float(f) => ConstValue::Float(f),
            LiteralValue::String(s) => ConstValue::String(s),
            LiteralValue::Bool(b) => ConstValue::Bool(b),
            LiteralValue::Null => ConstValue::Null,
            LiteralValue::Void => ConstValue::Void,
        };
        
        let dst = self.value_gen.next();
        self.emit_instruction(MirInstruction::Const {
            dst,
            value: const_value,
        })?;
        // Annotate type
        if let Some(ty) = ty_for_dst { self.value_types.insert(dst, ty); }
        
        Ok(dst)
    }
    
    // build_binary_op moved to builder/ops.rs
    // build_unary_op moved to builder/ops.rs
    
    /// Build variable access
    pub(super) fn build_variable_access(&mut self, name: String) -> Result<ValueId, String> {
        if let Some(&value_id) = self.variable_map.get(&name) {
            Ok(value_id)
        } else {
            Err(format!("Undefined variable: {}", name))
        }
    }
    
    /// Build assignment
    pub(super) fn build_assignment(&mut self, var_name: String, value: ASTNode) -> Result<ValueId, String> {
        let value_id = self.build_expression(value)?;
        
        // In SSA form, each assignment creates a new value
        self.variable_map.insert(var_name.clone(), value_id);
        
        Ok(value_id)
    }
    
    // build_function_call_legacy removed (use builder_calls::build_function_call)
    
    // build_print_statement_legacy moved to builder/stmts.rs
    
    // build_block_legacy moved to builder/stmts.rs
    
    // build_if_statement_legacy moved to builder/stmts.rs

    // extract_assigned_var moved to builder/stmts.rs (as module helper)
    
    /// Emit an instruction to the current basic block
    pub(super) fn emit_instruction(&mut self, instruction: MirInstruction) -> Result<(), String> {
        let block_id = self.current_block.ok_or("No current basic block")?;
        
        if let Some(ref mut function) = self.current_function {
            if let Some(block) = function.get_block_mut(block_id) {
                if utils::builder_debug_enabled() {
                    eprintln!("[BUILDER] emit @bb{} -> {}", block_id, match &instruction {
                        MirInstruction::TypeOp { dst, op, value, ty } => format!("typeop {:?} {} {:?} -> {}", op, value, ty, dst),
                        MirInstruction::Print { value, .. } => format!("print {}", value),
                        MirInstruction::BoxCall { box_val, method, method_id, args, dst, .. } => {
                            if let Some(mid) = method_id { 
                                format!("boxcall {}.{}[#{}]({:?}) -> {:?}", box_val, method, mid, args, dst)
                            } else {
                                format!("boxcall {}.{}({:?}) -> {:?}", box_val, method, args, dst)
                            }
                        },
                        MirInstruction::Call { func, args, dst, .. } => format!("call {}({:?}) -> {:?}", func, args, dst),
                        MirInstruction::NewBox { dst, box_type, args } => format!("new {}({:?}) -> {}", box_type, args, dst),
                        MirInstruction::Const { dst, value } => format!("const {:?} -> {}", value, dst),
                        MirInstruction::Branch { condition, then_bb, else_bb } => format!("br {}, {}, {}", condition, then_bb, else_bb),
                        MirInstruction::Jump { target } => format!("br {}", target),
                        _ => format!("{:?}", instruction),
                    });
                }
                block.add_instruction(instruction);
                Ok(())
            } else {
                Err(format!("Basic block {} does not exist", block_id))
            }
        } else {
            Err("No current function".to_string())
        }
    }
    
    // moved to builder/utils.rs: ensure_block_exists
    
    // build_loop_statement_legacy moved to builder/stmts.rs
    
    // build_try_catch_statement_legacy moved to builder/stmts.rs
    
    // build_throw_statement_legacy moved to builder/stmts.rs
    
    // build_local_statement_legacy moved to builder/stmts.rs
    
    // build_return_statement_legacy moved to builder/stmts.rs
    
    // moved to builder/decls.rs: build_static_main_box
    
    // moved to builder/fields.rs: build_field_access
    
    /// Build new expression: new ClassName(arguments)
    pub(super) fn build_new_expression(&mut self, class: String, arguments: Vec<ASTNode>) -> Result<ValueId, String> {
        // Phase 9.78a: Unified Box creation using NewBox instruction
        // Core-13 pure mode: emit ExternCall(env.box.new) with type name const only
        if crate::config::env::mir_core13_pure() {
            // Emit Const String for type name
            let ty_id = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const { dst: ty_id, value: ConstValue::String(class.clone()) })?;
            // Evaluate arguments (pass through to env.box.new shim)
            let mut arg_vals: Vec<ValueId> = Vec::with_capacity(arguments.len());
            for a in arguments { arg_vals.push(self.build_expression(a)?); }
            // Build arg list: [type, a1, a2, ...]
            let mut args: Vec<ValueId> = Vec::with_capacity(1 + arg_vals.len());
            args.push(ty_id);
            args.extend(arg_vals);
            // Call env.box.new
            let dst = self.value_gen.next();
            self.emit_instruction(MirInstruction::ExternCall {
                dst: Some(dst), iface_name: "env.box".to_string(), method_name: "new".to_string(), args, effects: EffectMask::PURE,
            })?;
            // 型注釈（最小）
            self.value_types.insert(dst, super::MirType::Box(class.clone()));
            return Ok(dst);
        }
        
        // Optimization: Primitive wrappers → emit Const directly when possible
        if class == "IntegerBox" && arguments.len() == 1 {
            if let ASTNode::Literal { value: LiteralValue::Integer(n), .. } = arguments[0].clone() {
                let dst = self.value_gen.next();
                self.emit_instruction(MirInstruction::Const { dst, value: ConstValue::Integer(n) })?;
                self.value_types.insert(dst, super::MirType::Integer);
                return Ok(dst);
            }
        }
        
        // First, evaluate all arguments to get their ValueIds
        let mut arg_values = Vec::new();
        for arg in arguments {
            let arg_value = self.build_expression(arg)?;
            arg_values.push(arg_value);
        }
        
        // Generate the destination ValueId
        let dst = self.value_gen.next();
        
        // Emit NewBox instruction for all Box types
        // VM will handle optimization for basic types internally
        self.emit_instruction(MirInstruction::NewBox {
            dst,
            box_type: class.clone(),
            args: arg_values.clone(),
        })?;
        // Annotate primitive boxes
        match class.as_str() {
            "IntegerBox" => { self.value_types.insert(dst, super::MirType::Integer); },
            "FloatBox" => { self.value_types.insert(dst, super::MirType::Float); },
            "BoolBox" => { self.value_types.insert(dst, super::MirType::Bool); },
            "StringBox" => { self.value_types.insert(dst, super::MirType::String); },
            other => { self.value_types.insert(dst, super::MirType::Box(other.to_string())); }
        }

        // Record origin for optimization: dst was created by NewBox of class
        self.value_origin_newbox.insert(dst, class.clone());

        // For plugin/builtin boxes, call birth(...). For user-defined boxes, skip (InstanceBox already constructed)
        // Special-case: StringBox is already fully constructed via from_i8_string in LLVM lowering; skip birth
        if !self.user_defined_boxes.contains(&class) && class != "StringBox" {
            let birt_mid = resolve_slot_by_type_name(&class, "birth");
            self.emit_box_or_plugin_call(
                None,
                dst,
                "birth".to_string(),
                birt_mid,
                arg_values,
                EffectMask::READ.add(Effect::ReadHeap),
            )?;
        }
        
        Ok(dst)
    }
    
    // moved to builder/fields.rs: build_field_assignment
    
    // moved to builder/utils.rs: start_new_block
    
    /// Check if the current basic block is terminated
    fn is_current_block_terminated(&self) -> bool {
        if let (Some(block_id), Some(ref function)) = (self.current_block, &self.current_function) {
            if let Some(block) = function.get_block(block_id) {
                return block.is_terminated();
            }
        }
        false
    }
    
    // convert_binary_operator moved to builder/ops.rs
    // convert_unary_operator moved to builder/ops.rs
    
    // build_nowait_statement_legacy moved to builder/stmts.rs
    
    // build_await_expression_legacy moved to builder/stmts.rs
    
    // build_me_expression_legacy moved to builder/stmts.rs
    
    // build_method_call_legacy removed (use builder_calls::build_method_call)

    // parse_type_name_to_mir_legacy removed (use builder_calls::parse_type_name_to_mir)
    // extract_string_literal_legacy removed (use builder_calls::extract_string_literal)
    // build_from_expression_legacy removed (use builder_calls::build_from_expression)

    // lower_static_method_as_function_legacy removed (use builder_calls::lower_static_method_as_function)
    
    // moved to builder/decls.rs: build_box_declaration
}

// BinaryOpType moved to builder/ops.rs

impl Default for MirBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ASTNode, LiteralValue, Span};
    
    #[test]
    fn test_literal_building() {
        let mut builder = MirBuilder::new();
        
        let ast = ASTNode::Literal {
            value: LiteralValue::Integer(42),
            span: Span::unknown(),
        };
        
        let result = builder.build_module(ast);
        assert!(result.is_ok());
        
        let module = result.unwrap();
        assert_eq!(module.function_names().len(), 1);
        assert!(module.get_function("main").is_some());
    }
    
    #[test]
    fn test_binary_op_building() {
        let mut builder = MirBuilder::new();
        
        let ast = ASTNode::BinaryOp {
            left: Box::new(ASTNode::Literal {
                value: LiteralValue::Integer(10),
                span: Span::unknown(),
            }),
            operator: BinaryOperator::Add,
            right: Box::new(ASTNode::Literal {
                value: LiteralValue::Integer(32),
                span: Span::unknown(),
            }),
            span: Span::unknown(),
        };
        
        let result = builder.build_module(ast);
        assert!(result.is_ok());
        
        let module = result.unwrap();
        let function = module.get_function("main").unwrap();
        
        // Should have constants and binary operation
        let stats = function.stats();
        assert!(stats.instruction_count >= 3); // 2 constants + 1 binop + 1 return
    }
    
    #[test]
    fn test_if_statement_building() {
        let mut builder = MirBuilder::new();
        
        // Adapt test to current AST: If with statement bodies
        let ast = ASTNode::If {
            condition: Box::new(ASTNode::Literal {
                value: LiteralValue::Bool(true),
                span: Span::unknown(),
            }),
            then_body: vec![ASTNode::Literal {
                value: LiteralValue::Integer(1),
                span: Span::unknown(),
            }],
            else_body: Some(vec![ASTNode::Literal {
                value: LiteralValue::Integer(2),
                span: Span::unknown(),
            }]),
            span: Span::unknown(),
        };
        
        let result = builder.build_module(ast);
        assert!(result.is_ok());
        
        let module = result.unwrap();
        let function = module.get_function("main").unwrap();
        
        // Should have multiple blocks for if/then/else/merge
        assert!(function.blocks.len() >= 3);
        
        // Should have phi function in merge block
        let stats = function.stats();
        assert!(stats.phi_count >= 1);
    }
}
