/*!
 * MIR Builder - Converts AST to MIR/SSA form
 *
 * Implements AST → MIR conversion with SSA construction
 */

use super::slot_registry::resolve_slot_by_type_name;
use super::{
    BasicBlock, BasicBlockId, BasicBlockIdGenerator, CompareOp, ConstValue, Effect, EffectMask,
    FunctionSignature, MirFunction, MirInstruction, MirModule, MirType, ValueId, ValueIdGenerator,
};
use crate::ast::{ASTNode, LiteralValue};
use std::collections::HashMap;
use std::collections::HashSet;
mod builder_calls;
mod decls; // declarations lowering split
mod exprs; // expression lowering split
mod exprs_call; // call(expr)
mod exprs_include; // include lowering
mod exprs_lambda; // lambda lowering
mod exprs_peek; // peek expression
mod exprs_qmark; // ?-propagate
mod exprs_legacy; // legacy big-match lowering
mod fields; // field access/assignment lowering split
pub(crate) mod loops;
mod ops;
mod phi;
mod if_form;
mod control_flow; // thin wrappers to centralize control-flow entrypoints
mod lifecycle; // prepare/lower_root/finalize split
// legacy large-match remains inline for now (planned extraction)
mod plugin_sigs; // plugin signature loader
mod stmts;
mod utils;
mod vars; // variables/scope helpers // small loop helpers (header/exit context)

// Unified member property kinds for computed/once/birth_once
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum PropertyKind {
    Computed,
    Once,
    BirthOnce,
}

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

    /// Unified members: BoxName -> {propName -> Kind}
    pub(super) property_getters_by_box: HashMap<String, HashMap<String, PropertyKind>>,

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

    /// If/merge context stack (innermost first). Used to make merge targets explicit
    /// when lowering nested conditionals and to simplify jump generation.
    pub(super) if_merge_stack: Vec<BasicBlockId>,

    /// Whether PHI emission is disabled (edge-copy mode)
    pub(super) no_phi_mode: bool,

    // ---- Try/Catch/Cleanup lowering context ----
    /// When true, `return` statements are deferred: they assign to `return_defer_slot`
    /// and jump to `return_defer_target` (typically the cleanup/exit block).
    pub(super) return_defer_active: bool,
    /// Slot value to receive deferred return values (edge-copy mode friendly).
    pub(super) return_defer_slot: Option<ValueId>,
    /// Target block to jump to on deferred return.
    pub(super) return_defer_target: Option<BasicBlockId>,
    /// Set to true when a deferred return has been emitted in the current context.
    pub(super) return_deferred_emitted: bool,
    /// True while lowering the cleanup block.
    pub(super) in_cleanup_block: bool,
    /// Policy flags (snapshotted at entry of try/catch lowering)
    pub(super) cleanup_allow_return: bool,
    pub(super) cleanup_allow_throw: bool,

    /// Hint sink (zero-cost guidance; currently no-op)
    pub(super) hint_sink: crate::mir::hints::HintSink,
}

impl MirBuilder {
    /// Create a new MIR builder
    pub fn new() -> Self {
        let plugin_method_sigs = plugin_sigs::load_plugin_method_sigs();
        let no_phi_mode = crate::config::env::mir_no_phi();
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
            property_getters_by_box: HashMap::new(),
            field_origin_class: HashMap::new(),
            value_types: HashMap::new(),
            plugin_method_sigs,
            current_static_box: None,
            include_loading: HashSet::new(),
            include_box_map: HashMap::new(),
            loop_header_stack: Vec::new(),
            loop_exit_stack: Vec::new(),
            if_merge_stack: Vec::new(),
            no_phi_mode,
            return_defer_active: false,
            return_defer_slot: None,
            return_defer_target: None,
            return_deferred_emitted: false,
            in_cleanup_block: false,
            cleanup_allow_return: false,
            cleanup_allow_throw: false,
            hint_sink: crate::mir::hints::HintSink::new(),
        }
    }

    /// Push/pop helpers for If merge context (best-effort; optional usage)
    pub(super) fn push_if_merge(&mut self, bb: BasicBlockId) { self.if_merge_stack.push(bb); }
    pub(super) fn pop_if_merge(&mut self) { let _ = self.if_merge_stack.pop(); }

    // ---- Hint helpers (no-op by default) ----
    #[inline]
    pub(crate) fn hint_loop_header(&mut self) { self.hint_sink.loop_header(); }
    #[inline]
    pub(crate) fn hint_loop_latch(&mut self) { self.hint_sink.loop_latch(); }
    #[inline]
    pub(crate) fn hint_scope_enter(&mut self, id: u32) { self.hint_sink.scope_enter(id); }
    #[inline]
    pub(crate) fn hint_scope_leave(&mut self, id: u32) { self.hint_sink.scope_leave(id); }
    #[inline]
    pub(crate) fn hint_join_result<S: Into<String>>(&mut self, var: S) { self.hint_sink.join_result(var.into()); }
    #[inline]
    pub(crate) fn hint_loop_carrier<S: Into<String>>(&mut self, vars: impl IntoIterator<Item = S>) {
        self.hint_sink.loop_carrier(vars.into_iter().map(|s| s.into()).collect::<Vec<_>>());
    }

    // moved to builder_calls.rs: lower_method_as_function

    /// Build a complete MIR module from AST
    pub fn build_module(&mut self, ast: ASTNode) -> Result<MirModule, String> {
        self.prepare_module()?;
        let result_value = self.lower_root(ast)?;
        self.finalize_module(result_value)
    }

    // prepare_module/lower_root/finalize_module moved to builder/lifecycle.rs

    /// Build an expression and return its value ID
    pub(super) fn build_expression(&mut self, ast: ASTNode) -> Result<ValueId, String> {
        // Delegated to exprs.rs to keep this file lean
        self.build_expression_impl(ast)
    }

    // build_expression_impl_legacy moved to builder/exprs_legacy.rs
    /*
    pub(super) fn build_expression_impl_legacy(&mut self, ast: ASTNode) -> Result<ValueId, String> {
        match ast {
            ASTNode::Literal { value, .. } => self.build_literal(value),

            ASTNode::BinaryOp {
                left,
                operator,
                right,
                ..
            } => self.build_binary_op(*left, operator, *right),

            ASTNode::UnaryOp {
                operator, operand, ..
            } => {
                let op_string = match operator {
                    crate::ast::UnaryOperator::Minus => "-".to_string(),
                    crate::ast::UnaryOperator::Not => "not".to_string(),
                };
                self.build_unary_op(op_string, *operand)
            }

            ASTNode::Variable { name, .. } => self.build_variable_access(name.clone()),

            ASTNode::Me { .. } => self.build_me_expression(),

            ASTNode::MethodCall {
                object,
                method,
                arguments,
                ..
            } => {
                // Early TypeOp lowering for method-style is()/as()
                if (method == "is" || method == "as") && arguments.len() == 1 {
                    if let Some(type_name) = Self::extract_string_literal(&arguments[0]) {
                        let obj_val = self.build_expression(*object.clone())?;
                        let ty = Self::parse_type_name_to_mir(&type_name);
                        let dst = self.value_gen.next();
                        let op = if method == "is" {
                            super::TypeOpKind::Check
                        } else {
                            super::TypeOpKind::Cast
                        };
                        self.emit_instruction(MirInstruction::TypeOp {
                            dst,
                            op,
                            value: obj_val,
                            ty,
                        })?;
                        return Ok(dst);
                    }
                }
                self.build_method_call(*object.clone(), method.clone(), arguments.clone())
            }

            ASTNode::FromCall {
                parent,
                method,
                arguments,
                ..
            } => self.build_from_expression(parent.clone(), method.clone(), arguments.clone()),

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
            }

            ASTNode::FunctionCall {
                name, arguments, ..
            } => {
                // Early TypeOp lowering for function-style isType()/asType()
                if (name == "isType" || name == "asType") && arguments.len() == 2 {
                    if let Some(type_name) = Self::extract_string_literal(&arguments[1]) {
                        let val = self.build_expression(arguments[0].clone())?;
                        let ty = Self::parse_type_name_to_mir(&type_name);
                        let dst = self.value_gen.next();
                        let op = if name == "isType" {
                            super::TypeOpKind::Check
                        } else {
                            super::TypeOpKind::Cast
                        };
                        self.emit_instruction(MirInstruction::TypeOp {
                            dst,
                            op,
                            value: val,
                            ty,
                        })?;
                        return Ok(dst);
                    }
                }
                self.build_function_call(name.clone(), arguments.clone())
            }
            ASTNode::Call {
                callee, arguments, ..
            } => {
                // P1.5: Lambdaはインライン、それ以外は Call に正規化
                if let ASTNode::Lambda { params, body, .. } = callee.as_ref() {
                    if params.len() != arguments.len() {
                        return Err(format!(
                            "Lambda expects {} args, got {}",
                            params.len(),
                            arguments.len()
                        ));
                    }
                    let mut arg_vals: Vec<ValueId> = Vec::new();
                    for a in arguments {
                        arg_vals.push(self.build_expression(a)?);
                    }
                    let saved_vars = self.variable_map.clone();
                    for (p, v) in params.iter().zip(arg_vals.iter()) {
                        self.variable_map.insert(p.clone(), *v);
                    }
                    let prog = ASTNode::Program {
                        statements: body.clone(),
                        span: crate::ast::Span::unknown(),
                    };
                    let out = self.build_expression(prog)?;
                    self.variable_map = saved_vars;
                    Ok(out)
                } else {
                    // callee/args を評価し、Call を発行（VM 側で FunctionBox/関数名の両対応）
                    let callee_id = self.build_expression(*callee.clone())?;
                    let mut arg_ids = Vec::new();
                    for a in arguments {
                        arg_ids.push(self.build_expression(a)?);
                    }
                    let dst = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Call {
                        dst: Some(dst),
                        func: callee_id,
                        args: arg_ids,
                        effects: EffectMask::PURE,
                    })?;
                    Ok(dst)
                }
            }

            ASTNode::QMarkPropagate { expression, .. } => {
                // Lower: ok = expr.isOk(); br ok then else; else => return expr; then => expr.getValue()
                let res_val = self.build_expression(*expression.clone())?;
                let ok_id = self.value_gen.next();
                self.emit_instruction(MirInstruction::BoxCall {
                    dst: Some(ok_id),
                    box_val: res_val,
                    method: "isOk".to_string(),
                    method_id: None,
                    args: vec![],
                    effects: EffectMask::PURE,
                })?;
                let then_block = self.block_gen.next();
                let else_block = self.block_gen.next();
                self.emit_instruction(MirInstruction::Branch {
                    condition: ok_id,
                    then_bb: then_block,
                    else_bb: else_block,
                })?;
                // else: return res_val
                self.current_block = Some(else_block);
                self.ensure_block_exists(else_block)?;
                self.emit_instruction(MirInstruction::Return {
                    value: Some(res_val),
                })?;
                // then: getValue()
                self.current_block = Some(then_block);
                self.ensure_block_exists(then_block)?;
                let val_id = self.value_gen.next();
                self.emit_instruction(MirInstruction::BoxCall {
                    dst: Some(val_id),
                    box_val: res_val,
                    method: "getValue".to_string(),
                    method_id: None,
                    args: vec![],
                    effects: EffectMask::PURE,
                })?;
                self.value_types.insert(val_id, super::MirType::Unknown);
                Ok(val_id)
            }

            ASTNode::Print { expression, .. } => self.build_print_statement(*expression.clone()),

            ASTNode::Program { statements, .. } => self.build_block(statements.clone()),

            ASTNode::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
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
                    else_ast,
                )
            }

            ASTNode::Loop {
                condition, body, ..
            } => self.build_loop_statement(*condition.clone(), body.clone()),

            ASTNode::TryCatch {
                try_body,
                catch_clauses,
                finally_body,
                ..
            } => self.build_try_catch_statement(
                try_body.clone(),
                catch_clauses.clone(),
                finally_body.clone(),
            ),

            ASTNode::Throw { expression, .. } => self.build_throw_statement(*expression.clone()),

            // P1: Lower peek expression into if-else chain with phi
            ASTNode::PeekExpr {
                scrutinee,
                arms,
                else_expr,
                ..
            } => {
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
                    self.emit_instruction(super::MirInstruction::Compare {
                        dst: cond_id,
                        op: super::CompareOp::Eq,
                        lhs: scr_val,
                        rhs: lit_id,
                    })?;

                    // Create then and next blocks
                    let then_block = self.block_gen.next();
                    let next_block = self.block_gen.next();
                    self.emit_instruction(super::MirInstruction::Branch {
                        condition: cond_id,
                        then_bb: then_block,
                        else_bb: next_block,
                    })?;

                    // then: evaluate arm expr, jump to merge
                    self.current_block = Some(then_block);
                    self.ensure_block_exists(then_block)?;
                    let then_val = self.build_expression(arm_expr)?;
                    if !self.is_current_block_terminated() {
                        self.emit_instruction(super::MirInstruction::Jump {
                            target: merge_block,
                        })?;
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
                    self.emit_instruction(super::MirInstruction::Jump {
                        target: merge_block,
                    })?;
                }
                phi_inputs.push((cur_block, else_val));

                // Merge and phi
                self.current_block = Some(merge_block);
                self.ensure_block_exists(merge_block)?;
                let result_val = self.value_gen.next();
                self.emit_instruction(super::MirInstruction::Phi {
                    dst: result_val,
                    inputs: phi_inputs,
                })?;
                Ok(result_val)
            }

            ASTNode::Lambda { params, body, .. } => {
                // Lambda→FunctionBox 値 Lower（最小 + 簡易キャプチャ解析）
                let dst = self.value_gen.next();
                // Collect free variable names: variables used in body but not in params, and not 'me'/'this'
                use std::collections::HashSet;
                let mut used: HashSet<String> = HashSet::new();
                let mut locals: HashSet<String> = HashSet::new();
                for p in params.iter() {
                    locals.insert(p.clone());
                }
                for st in body.iter() {
                    vars::collect_free_vars(st, &mut used, &mut locals);
                }
                // Materialize captures from current variable_map if known
                let mut captures: Vec<(String, ValueId)> = Vec::new();
                for name in used.into_iter() {
                    if let Some(&vid) = self.variable_map.get(&name) {
                        captures.push((name, vid));
                    }
                }
                // me capture（存在すれば）
                let me = self.variable_map.get("me").copied();
                self.emit_instruction(MirInstruction::FunctionNew {
                    dst,
                    params: params.clone(),
                    body: body.clone(),
                    captures,
                    me,
                })?;
                self.value_types
                    .insert(dst, super::MirType::Box("FunctionBox".to_string()));
                Ok(dst)
            }

            ASTNode::Return { value, .. } => self.build_return_statement(value.clone()),

            ASTNode::Local {
                variables,
                initial_values,
                ..
            } => self.build_local_statement(variables.clone(), initial_values.clone()),

            ASTNode::BoxDeclaration {
                name,
                methods,
                is_static,
                fields,
                constructors,
                weak_fields,
                ..
            } => {
                if is_static && name == "Main" {
                    self.build_static_main_box(name.clone(), methods.clone())
                } else {
                    // Support user-defined boxes - handle as statement, return void
                    // Track as user-defined (eligible for method lowering)
                    self.user_defined_boxes.insert(name.clone());
                    self.build_box_declaration(
                        name.clone(),
                        methods.clone(),
                        fields.clone(),
                        weak_fields.clone(),
                    )?;

                    // Phase 2: Lower constructors (birth/N) into MIR functions
                    // Function name pattern: "{BoxName}.{constructor_key}" (e.g., "Person.birth/1")
                    for (ctor_key, ctor_ast) in constructors.clone() {
                        if let ASTNode::FunctionDeclaration { params, body, .. } = ctor_ast {
                            let func_name = format!("{}.{}", name, ctor_key);
                            self.lower_method_as_function(
                                func_name,
                                name.clone(),
                                params.clone(),
                                body.clone(),
                            )?;
                        }
                    }

                    // Phase 3: Lower instance methods into MIR functions
                    // Function name pattern: "{BoxName}.{method}/{N}"
                    for (method_name, method_ast) in methods.clone() {
                        if let ASTNode::FunctionDeclaration {
                            params,
                            body,
                            is_static,
                            ..
                        } = method_ast
                        {
                            if !is_static {
                                let func_name = format!(
                                    "{}.{}{}",
                                    name,
                                    method_name,
                                    format!("/{}", params.len())
                                );
                                self.lower_method_as_function(
                                    func_name,
                                    name.clone(),
                                    params.clone(),
                                    body.clone(),
                                )?;
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
            }

            ASTNode::FieldAccess { object, field, .. } => {
                self.build_field_access(*object.clone(), field.clone())
            }

            ASTNode::New {
                class, arguments, ..
            } => self.build_new_expression(class.clone(), arguments.clone()),

            ASTNode::ArrayLiteral { elements, .. } => {
                // Lower: new ArrayBox(); for each elem: .push(elem)
                let arr_id = self.value_gen.next();
                self.emit_instruction(MirInstruction::NewBox {
                    dst: arr_id,
                    box_type: "ArrayBox".to_string(),
                    args: vec![],
                })?;
                for e in elements {
                    let v = self.build_expression(e)?;
                    self.emit_instruction(MirInstruction::BoxCall {
                        dst: None,
                        box_val: arr_id,
                        method: "push".to_string(),
                        method_id: None,
                        args: vec![v],
                        effects: super::EffectMask::MUT,
                    })?;
                }
                Ok(arr_id)
            }

            // Phase 7: Async operations
            ASTNode::Nowait {
                variable,
                expression,
                ..
            } => self.build_nowait_statement(variable.clone(), *expression.clone()),

            ASTNode::AwaitExpression { expression, .. } => {
                self.build_await_expression(*expression.clone())
            }

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
                        if let crate::ast::ASTNode::BoxDeclaration {
                            name, is_static, ..
                        } = st
                        {
                            if *is_static {
                                box_name = Some(name.clone());
                                break;
                            }
                        }
                    }
                }
                let bname = box_name
                    .ok_or_else(|| format!("Include target '{}' has no static box", filename))?;
                // Lower included AST into current MIR (register types/methods)
                let _ = self.build_expression(included_ast)?;
                // Mark caches
                self.include_loading.remove(&path);
                self.include_box_map.insert(path.clone(), bname.clone());
                // Return a new instance of included box (no args)
                self.build_new_expression(bname, vec![])
            }

            _ => Err(format!("Unsupported AST node type: {:?}", ast)),
        }
    }
    */

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
        if let Some(ty) = ty_for_dst {
            self.value_types.insert(dst, ty);
        }

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
    pub(super) fn build_assignment(
        &mut self,
        var_name: String,
        value: ASTNode,
    ) -> Result<ValueId, String> {
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
                    eprintln!(
                        "[BUILDER] emit @bb{} -> {}",
                        block_id,
                        match &instruction {
                            MirInstruction::TypeOp { dst, op, value, ty } =>
                                format!("typeop {:?} {} {:?} -> {}", op, value, ty, dst),
                            MirInstruction::Print { value, .. } => format!("print {}", value),
                            MirInstruction::BoxCall {
                                box_val,
                                method,
                                method_id,
                                args,
                                dst,
                                ..
                            } => {
                                if let Some(mid) = method_id {
                                    format!(
                                        "boxcall {}.{}[#{}]({:?}) -> {:?}",
                                        box_val, method, mid, args, dst
                                    )
                                } else {
                                    format!(
                                        "boxcall {}.{}({:?}) -> {:?}",
                                        box_val, method, args, dst
                                    )
                                }
                            }
                            MirInstruction::Call {
                                func, args, dst, ..
                            } => format!("call {}({:?}) -> {:?}", func, args, dst),
                            MirInstruction::NewBox {
                                dst,
                                box_type,
                                args,
                            } => format!("new {}({:?}) -> {}", box_type, args, dst),
                            MirInstruction::Const { dst, value } =>
                                format!("const {:?} -> {}", value, dst),
                            MirInstruction::Branch {
                                condition,
                                then_bb,
                                else_bb,
                            } => format!("br {}, {}, {}", condition, then_bb, else_bb),
                            MirInstruction::Jump { target } => format!("br {}", target),
                            _ => format!("{:?}", instruction),
                        }
                    );
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

    pub(super) fn is_no_phi_mode(&self) -> bool {
        self.no_phi_mode
    }

    /// Insert a Copy instruction into `block_id`, defining `dst` from `src`.
    /// Skips blocks that terminate via return/throw, and avoids duplicate copies.
    pub(super) fn insert_edge_copy(
        &mut self,
        block_id: BasicBlockId,
        dst: ValueId,
        src: ValueId,
    ) -> Result<(), String> {
        // No-op self copy guard to avoid useless instructions and accidental reordering issues
        if dst == src {
            return Ok(());
        }
        if let Some(ref mut function) = self.current_function {
            let block = function
                .get_block_mut(block_id)
                .ok_or_else(|| format!("Basic block {} does not exist", block_id))?;
            if let Some(term) = &block.terminator {
                if matches!(
                    term,
                    MirInstruction::Return { .. } | MirInstruction::Throw { .. }
                ) {
                    return Ok(());
                }
            }
            let already_present = block.instructions.iter().any(|inst| {
                matches!(inst, MirInstruction::Copy { dst: existing_dst, .. } if *existing_dst == dst)
            });
            if !already_present {
                block.add_instruction(MirInstruction::Copy { dst, src });
            }
            if let Some(ty) = self.value_types.get(&src).cloned() {
                self.value_types.insert(dst, ty);
            }
            Ok(())
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
    pub(super) fn build_new_expression(
        &mut self,
        class: String,
        arguments: Vec<ASTNode>,
    ) -> Result<ValueId, String> {
        // Phase 9.78a: Unified Box creation using NewBox instruction
        // Core-13 pure mode: emit ExternCall(env.box.new) with type name const only
        if crate::config::env::mir_core13_pure() {
            // Emit Const String for type name
            let ty_id = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const {
                dst: ty_id,
                value: ConstValue::String(class.clone()),
            })?;
            // Evaluate arguments (pass through to env.box.new shim)
            let mut arg_vals: Vec<ValueId> = Vec::with_capacity(arguments.len());
            for a in arguments {
                arg_vals.push(self.build_expression(a)?);
            }
            // Build arg list: [type, a1, a2, ...]
            let mut args: Vec<ValueId> = Vec::with_capacity(1 + arg_vals.len());
            args.push(ty_id);
            args.extend(arg_vals);
            // Call env.box.new
            let dst = self.value_gen.next();
            self.emit_instruction(MirInstruction::ExternCall {
                dst: Some(dst),
                iface_name: "env.box".to_string(),
                method_name: "new".to_string(),
                args,
                effects: EffectMask::PURE,
            })?;
            // 型注釈（最小）
            self.value_types
                .insert(dst, super::MirType::Box(class.clone()));
            return Ok(dst);
        }

        // Optimization: Primitive wrappers → emit Const directly when possible
        if class == "IntegerBox" && arguments.len() == 1 {
            if let ASTNode::Literal {
                value: LiteralValue::Integer(n),
                ..
            } = arguments[0].clone()
            {
                let dst = self.value_gen.next();
                self.emit_instruction(MirInstruction::Const {
                    dst,
                    value: ConstValue::Integer(n),
                })?;
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
            "IntegerBox" => {
                self.value_types.insert(dst, super::MirType::Integer);
            }
            "FloatBox" => {
                self.value_types.insert(dst, super::MirType::Float);
            }
            "BoolBox" => {
                self.value_types.insert(dst, super::MirType::Bool);
            }
            "StringBox" => {
                self.value_types.insert(dst, super::MirType::String);
            }
            other => {
                self.value_types
                    .insert(dst, super::MirType::Box(other.to_string()));
            }
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
