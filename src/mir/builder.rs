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
mod call_resolution; // ChatGPT5 Pro: Type-safe call resolution utilities
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

    // フェーズM: no_phi_modeフィールド削除（常にPHI使用）

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
        // フェーズM: no_phi_mode初期化削除
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
            // フェーズM: no_phi_modeフィールド削除
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

    // build_expression_impl_legacy moved to builder/exprs_legacy.rs (legacy body removed)

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

    // フェーズM: is_no_phi_mode()メソッド削除

    // フェーズM: insert_edge_copy()メソッド削除（no_phi_mode撤廃により不要）

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

// Unit tests moved to `tests/mir_builder_unit.rs` to keep this file lean
