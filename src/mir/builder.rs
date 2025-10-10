/*!
 * MIR Builder - Converts AST to MIR/SSA form
 *
 * Implements AST → MIR conversion with SSA construction
 */

use super::{
    BasicBlock, BasicBlockId, BasicBlockIdGenerator, CompareOp, ConstValue, Effect, EffectMask,
    FunctionSignature, MirFunction, MirInstruction, MirModule, MirType, ValueId, ValueIdGenerator,
};
use crate::ast::{ASTNode, LiteralValue};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::OnceLock;
mod calls; // Call system modules (refactored from builder_calls)
pub mod birth; // Auto-birth policy/emitter (thin boxes)
mod builder_calls;
mod method_call_handlers; // Method call handler separation (Phase 3)
mod decls; // declarations lowering split
mod exprs; // expression lowering split
mod exprs_call; // call(expr)
// include lowering removed (using is handled in runner)
mod exprs_lambda; // lambda lowering
mod exprs_peek; // peek expression
mod exprs_qmark; // ?-propagate
mod exprs_legacy; // legacy big-match lowering
mod fields; // field access/assignment lowering split
pub(crate) mod loops;
mod ops;
mod phi;
mod phi_merge_helper; // PHI生成とマージの統一処理（箱理論実践）
mod if_form;
mod control_flow; // thin wrappers to centralize control-flow entrypoints
mod lifecycle; // prepare/lower_root/finalize split
// legacy large-match remains inline for now (planned extraction)
mod plugin_sigs; // plugin signature loader
mod stmts;
mod utils;
mod vars; // variables/scope helpers // small loop helpers (header/exit context)
mod origin; // P0: origin inference（me/Known）と PHI 伝播（軽量）
mod observe; // P0: dev-only observability helpers（ssa/resolve）
mod rewrite; // P1: Known rewrite & special consolidation
mod ssa; // LocalSSA helpers (in-block materialization)
mod schedule; // BlockScheduleBox（物理順序: PHI→materialize→body）
mod metadata; // MetadataPropagationBox（type/originの伝播）
mod emission;  // emission::*（Const/Compare/Branch の薄い発行箱）
mod types;     // types::annotation / inference（型注釈/推論の箱: 推論は後段）
mod router;    // RouterPolicyBox（Unified vs BoxCall）
mod emit_guard; // EmitGuardBox（emit直前の最終関所）
mod name_const; // NameConstBox（関数名Const生成）
mod infer; // ReceiverInferenceBox（受け手推定の一元化）
pub mod lowering; // Builtin→Extern lowering map（対応表）
// rewrite は既存モジュールを使用（gate サブモジュールを追加）
mod indexes; // InstanceMethodIndexBox（(Box,method,arity)登録/照会）
mod materialize; // MaterializeBox（Call直前の材化を一元化）
mod verify; // CallOrderVerifyBox（dev-only 検証ラッパ）
pub mod effects; // EffectResolverBox（効果決定の単一入口・既定OFF）
pub mod entry; // Public entrypoint wrapper (AST→MIR module)

// Unified member property kinds for computed/once/birth_once
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum PropertyKind {
    Computed,
    Once,
    BirthOnce,
}


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
    /// Names of static boxes (including flow) declared in the current module
    pub(super) static_box_names: HashSet<String>,

    /// Weak field registry: BoxName -> {weak field names}
    pub(super) weak_fields_by_box: HashMap<String, HashSet<String>>,

    /// Unified members: BoxName -> {propName -> Kind}
    pub(super) property_getters_by_box: HashMap<String, HashMap<String, PropertyKind>>,

    /// Remember class of object fields after assignments: (base_id, field) -> class_name
    pub(super) field_origin_class: HashMap<(ValueId, String), String>,
    /// Class-level field origin (cross-function heuristic): (BaseBoxName, field) -> FieldBoxName
    pub(super) field_origin_by_box: HashMap<(String, String), String>,

    /// Optional per-value type annotations (MIR-level): ValueId -> MirType
    pub(super) value_types: HashMap<ValueId, super::MirType>,

    /// Plugin method return type signatures loaded from nyash_box.toml
    plugin_method_sigs: HashMap<(String, String), super::MirType>,
    /// Current static box name when lowering a static box body (e.g., "Main")
    current_static_box: Option<String>,
    /// Index of static methods seen during lowering: name -> [(BoxName, arity)]
    pub(super) static_method_index: std::collections::HashMap<String, Vec<(String, usize)>>,
    /// Index of instance methods seen during lowering: (BoxName, method, arity)
    pub(super) instance_method_index: std::collections::HashSet<(String, String, usize)>,

    /// Fast lookup: method+arity tail → candidate function names (e.g., ".str/0" → ["JsonNode.str/0", ...])
    pub(super) method_tail_index: std::collections::HashMap<String, Vec<String>>,
    /// Source size snapshot to detect when to rebuild the tail index
    pub(super) method_tail_index_source_len: usize,

    // include guards removed

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

    /// Internal counter for temporary pin slots (block-crossing ephemeral values)
    temp_slot_counter: u32,
    /// If true, skip entry materialization of pinned slots on the next start_new_block call.
    suppress_pin_entry_copy_next: bool,

    // ----------------------
    // Debug scope context (dev only; zero-cost when unused)
    // ----------------------
    /// Stack of region identifiers like "loop#1/header" or "join#3/join".
    debug_scope_stack: Vec<String>,
    /// Monotonic counters for region IDs (deterministic across a run).
    debug_loop_counter: u32,
    debug_join_counter: u32,

    /// Local SSA cache: ensure per-block materialization for critical operands (e.g., recv)
    /// Key: (bb, original ValueId, kind) -> local ValueId
    /// kind: 0=recv, 1+ reserved for future (args etc.)
    pub(super) local_ssa_map: HashMap<(BasicBlockId, ValueId, u8), ValueId>,
    /// BlockSchedule cache: deduplicate materialize copies per (bb, src)
    pub(super) schedule_mat_map: HashMap<(BasicBlockId, ValueId), ValueId>,
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
            static_box_names: HashSet::new(),
            weak_fields_by_box: HashMap::new(),
            property_getters_by_box: HashMap::new(),
            field_origin_class: HashMap::new(),
            field_origin_by_box: HashMap::new(),
            value_types: HashMap::new(),
            plugin_method_sigs,
            current_static_box: None,
            static_method_index: std::collections::HashMap::new(),
            instance_method_index: std::collections::HashSet::new(),
            method_tail_index: std::collections::HashMap::new(),
            method_tail_index_source_len: 0,
            
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
            temp_slot_counter: 0,
            suppress_pin_entry_copy_next: false,

            // Debug scope context
            debug_scope_stack: Vec::new(),
            debug_loop_counter: 0,
            debug_join_counter: 0,

            local_ssa_map: HashMap::new(),
            schedule_mat_map: HashMap::new(),
        }
    }

    /// Push/pop helpers for If merge context (best-effort; optional usage)
    pub(super) fn push_if_merge(&mut self, bb: BasicBlockId) { self.if_merge_stack.push(bb); }
    pub(super) fn pop_if_merge(&mut self) { let _ = self.if_merge_stack.pop(); }

    /// Suppress entry pin copy for the next start_new_block (used for merge blocks).
    pub(super) fn suppress_next_entry_pin_copy(&mut self) { self.suppress_pin_entry_copy_next = true; }

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

    // ----------------------
    // Debug scope helpers (region_id for DebugHub events)
    // ----------------------
    #[inline]
    pub(crate) fn debug_next_loop_id(&mut self) -> u32 {
        let id = self.debug_loop_counter;
        self.debug_loop_counter = self.debug_loop_counter.saturating_add(1);
        id
    }

    #[inline]
    pub(crate) fn debug_next_join_id(&mut self) -> u32 {
        let id = self.debug_join_counter;
        self.debug_join_counter = self.debug_join_counter.saturating_add(1);
        id
    }

    #[inline]
    pub(crate) fn debug_push_region<S: Into<String>>(&mut self, region: S) {
        self.debug_scope_stack.push(region.into());
    }

    #[inline]
    pub(crate) fn debug_pop_region(&mut self) {
        let _ = self.debug_scope_stack.pop();
    }

    #[inline]
    pub(crate) fn debug_replace_region<S: Into<String>>(&mut self, region: S) {
        if let Some(top) = self.debug_scope_stack.last_mut() {
            *top = region.into();
        } else {
            self.debug_scope_stack.push(region.into());
        }
    }

    #[inline]
    pub(crate) fn debug_current_region_id(&self) -> Option<String> {
        self.debug_scope_stack.last().cloned()
    }

    // ----------------------
    // Method tail index (performance helper)
    // ----------------------
    fn rebuild_method_tail_index(&mut self) {
        self.method_tail_index.clear();
        if let Some(ref module) = self.current_module {
            for name in module.functions.keys() {
                if let (Some(dot), Some(slash)) = (name.rfind('.'), name.rfind('/')) {
                    if slash > dot {
                        let tail = &name[dot..];
                        self.method_tail_index
                            .entry(tail.to_string())
                            .or_insert_with(Vec::new)
                            .push(name.clone());
                    }
                }
            }
            self.method_tail_index_source_len = module.functions.len();
        } else {
            self.method_tail_index_source_len = 0;
        }
    }

    fn ensure_method_tail_index(&mut self) {
        let need_rebuild = match self.current_module {
            Some(ref refmod) => self.method_tail_index_source_len != refmod.functions.len(),
            None => self.method_tail_index_source_len != 0,
        };
        if need_rebuild {
            self.rebuild_method_tail_index();
        }
    }

    pub(super) fn method_candidates(&mut self, method: &str, arity: usize) -> Vec<String> {
        self.ensure_method_tail_index();
        let tail = format!(".{}{}", method, format!("/{}", arity));
        self.method_tail_index
            .get(&tail)
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn method_candidates_tail<S: AsRef<str>>(&mut self, tail: S) -> Vec<String> {
        self.ensure_method_tail_index();
        self.method_tail_index
            .get(tail.as_ref())
            .cloned()
            .unwrap_or_default()
    }


    /// Build a complete MIR module from AST
    pub fn build_module(&mut self, ast: ASTNode) -> Result<MirModule, String> {
        self.prepare_module()?;
        let result_value = self.lower_root(ast)?;
        self.finalize_module(result_value)
    }


    /// Build an expression and return its value ID
    pub(super) fn build_expression(&mut self, ast: ASTNode) -> Result<ValueId, String> {
        // Delegated to exprs.rs to keep this file lean
        self.build_expression_impl(ast)
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

        // Emit via ConstantEmissionBox（仕様不変の統一ルート）
        let dst = match literal {
            LiteralValue::Integer(n) => crate::mir::builder::emission::constant::emit_integer(self, n),
            LiteralValue::Float(f) => crate::mir::builder::emission::constant::emit_float(self, f),
            LiteralValue::String(s) => crate::mir::builder::emission::constant::emit_string(self, s),
            LiteralValue::Bool(b) => crate::mir::builder::emission::constant::emit_bool(self, b),
            LiteralValue::Null => crate::mir::builder::emission::constant::emit_null(self),
            LiteralValue::Void => crate::mir::builder::emission::constant::emit_void(self),
        };
        // Annotate type
        if let Some(ty) = ty_for_dst {
            self.value_types.insert(dst, ty);
        }

        Ok(dst)
    }


    /// Build variable access
    pub(super) fn build_variable_access(&mut self, name: String) -> Result<ValueId, String> {
        if let Some(&value_id) = self.variable_map.get(&name) {
            Ok(value_id)
        } else {
            // Enhance diagnostics using Using simple registry (Phase 1)
            let cur_fn = self.current_function.as_ref().map(|f| f.signature.name.clone()).unwrap_or_else(|| "<unknown>".to_string());
            let mut msg = format!("Undefined variable: {} (in function {})", name, cur_fn);
            let suggest = crate::using::simple_registry::suggest_using_for_symbol(&name);
            if !suggest.is_empty() {
                msg.push_str("\nHint: symbol appears in using module(s): ");
                msg.push_str(&suggest.join(", "));
                msg.push_str("\nConsider adding 'using <module> [as Alias]' or check nyash.toml [using].");
            }
            Err(msg)
        }
    }

    /// Build assignment
    pub(super) fn build_assignment(
        &mut self,
        var_name: String,
        value: ASTNode,
    ) -> Result<ValueId, String> {
        let raw_value_id = self.build_expression(value)?;
        // Correctness-first: assignment results may be used across control-flow joins.
        // Pin to a slot so the value has a block-local def and participates in PHI merges.
        let value_id = self
            .pin_to_slot(raw_value_id, "@assign")
            .unwrap_or(raw_value_id);

        // In SSA form, each assignment creates a new value
        // Dev-stability: inside ParserBox.* methods, avoid binding `me`'s ValueId to other variable names
        // (rare mis-binding can cause downstream ops like `j+1` to see BoxRef(ParserBox)).
        let value_id = if let Some(fun) = self.current_function.as_ref() {
            if fun.signature.name.starts_with("ParserBox.") {
                if let Some(&me_vid) = self.variable_map.get("me") {
                    if me_vid == value_id {
                        // materialize a local copy and bind that instead of `me`
                        if let Ok(loc) = self.materialize_local(value_id) {
                            loc
                        } else {
                            value_id
                        }
                    } else { value_id }
                } else { value_id }
            } else { value_id }
        } else { value_id };
        self.variable_map.insert(var_name.clone(), value_id);

        Ok(value_id)
    }

    fn origin_trace_enabled() -> bool {
        static FLAG: OnceLock<bool> = OnceLock::new();
        *FLAG.get_or_init(|| matches!(
            std::env::var("NYASH_ORIGIN_TRACE").ok().as_deref(),
            Some("1" | "true" | "on")
        ))
    }

    pub fn origin_tracker(&mut self) -> crate::mir::builder::origin::tracker::OriginTrackerBox<'_> {
        crate::mir::builder::origin::tracker::OriginTrackerBox::new(
            &mut self.value_origin_newbox,
            Self::origin_trace_enabled(),
        )
    }

    pub fn origin_register<S: Into<String>>(&mut self, value_id: ValueId, class_name: S) {
        let mut tracker = self.origin_tracker();
        tracker.register_newbox(value_id, class_name);
    }

    pub fn origin_propagate(&mut self, from: ValueId, to: ValueId) {
        let mut tracker = self.origin_tracker();
        tracker.propagate(from, to);
    }

    pub fn origin_get(&self, value_id: ValueId) -> Option<&str> {
        self.value_origin_newbox.get(&value_id).map(|s| s.as_str())
    }

    pub fn origin_snapshot(&mut self) -> std::collections::HashMap<ValueId, String> {
        std::mem::take(&mut self.value_origin_newbox)
    }

    pub fn origin_restore(&mut self, snapshot: std::collections::HashMap<ValueId, String>) {
        self.value_origin_newbox = snapshot;
    }



    /// Emit an instruction to the current basic block
    pub(super) fn emit_instruction(&mut self, instruction: MirInstruction) -> Result<(), String> {
        let block_id = self.current_block.ok_or("No current basic block")?;

        // Precompute debug metadata to avoid borrow conflicts later
        let _dbg_fn_name = self
            .current_function
            .as_ref()
            .map(|f| f.signature.name.clone());
        let _dbg_region_id = self.debug_current_region_id();
        // P0: PHI の軽量補強と観測は、関数ブロック取得前に実施して借用競合を避ける
        if let MirInstruction::Phi { dst, inputs } = &instruction {
            origin::phi::propagate_phi_meta(self, *dst, inputs);
            observe::ssa::emit_phi(self, *dst, inputs);
        }

        if let Some(ref mut function) = self.current_function {
            // Fail-Fast: do not emit any instruction after a terminator in the current block
            let already_terminated = function
                .get_block(block_id)
                .map(|b| b.is_terminated())
                .unwrap_or(false);
            if already_terminated {
                let fname = _dbg_fn_name.as_deref().unwrap_or("<unknown>");
                return Err(format!(
                    "Builder emit after terminator forbidden: function='{}' block={}",
                    fname, block_id
                ));
            }
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
                            MirInstruction::Call { func, callee, args, dst, .. } => {
                                if let Some(c) = callee {
                                    match c {
                                        crate::mir::definitions::Callee::ModuleFunction(name) =>
                                            format!("call MF:{}({:?}) -> {:?}", name, args, dst),
                                        crate::mir::definitions::Callee::Global(name) =>
                                            format!("call G:{}({:?}) -> {:?}", name, args, dst),
                                        crate::mir::definitions::Callee::Method { box_name, method, .. } =>
                                            format!("call M:{}.{}/{} -> {:?}", box_name, method, args.len(), dst),
                                        _ => format!("call {:?}({:?}) -> {:?}", c, args, dst),
                                    }
                                } else {
                                    format!("call {}({:?}) -> {:?}", func, args, dst)
                                }
                            },
                            MirInstruction::NewBox { dst, box_type, args, .. } => format!("new {}({:?}) -> {}", box_type, args, dst),
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
                crate::mir::builder::effects::verify_instruction_effects(&instruction);
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


    /// Build new expression: new ClassName(arguments)
    pub(super) fn build_new_expression(
        &mut self,
        class: String,
        arguments: Vec<ASTNode>,
    ) -> Result<ValueId, String> {
        if std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
            eprintln!("[BUILDER] enter build_new_expression class={} argc={}", class, arguments.len());
        }

        // Forbid instantiation of static/flow boxes declared in source
        if self.static_box_names.contains(&class) {
            return Err(format!(
                "Cannot instantiate static/flow box '{}' (use 'flow {} {{ ... }}' with static methods)",
                class, class
            ));
        }
        // Phase 9.78a: Unified Box creation using NewBox instruction
        // Core‑13 pure mode removed; normal path only.

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
        // Decide auto_birth (env-gated)
        let auto_birth_enabled = match std::env::var("NYASH_BUILDER_NEWBOX_AUTOBIRTH").ok().as_deref() {
            Some("0") | Some("false") | Some("off") => false,
            _ => true,
        };
        let mut auto_birth: Option<String> = None;
        if auto_birth_enabled && class != "StringBox" {
            let birth_name = crate::mir::resolve::call_name_resolver::CallNameResolverBox::make_birth_name(&class, arg_values.len());
            // Do not limit to current_module: embed auto_birth name; VM will call only if exists
            auto_birth = Some(birth_name);
        }
        self.emit_instruction(MirInstruction::NewBox {
            dst,
            box_type: class.clone(),
            args: arg_values.clone(),
            auto_birth,
        })?;
        // Dev trace: log NewBox emission when enabled
        if std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
            eprintln!(
                "[BUILDER] new {}(arity={}) -> v{}",
                class,
                arg_values.len(),
                dst
            );
        }
        // Phase 15.5: Unified box type handling
        // All boxes (including former core boxes) are treated uniformly as Box types
        self.value_types
            .insert(dst, super::MirType::Box(class.clone()));

        // Record origin for optimization: dst was created by NewBox of class
        self.origin_register(dst, class.clone());

        // birth 呼び出し（Builder 正規化）
        // 優先: 低下済みグローバル関数 `<Class>.birth/Arity`（Arity は me を含まない）
        // 代替: 既存互換として BoxCall("birth")（プラグイン/ビルトインの初期化に対応）
        if auto_birth_enabled == false {
            if let Some(call_insn) = crate::mir::builder::birth::emitter::BirthCallEmitterBox::try_build(
            self,
            &class,
            dst,
            arg_values.clone(),
        ) {
            if std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                eprintln!("[BUILDER] auto-birth via ModuleFunction for v{}", dst);
            }
            self.emit_instruction(call_insn)?;
            }
        }
        

        Ok(dst)
    }


    /// Check if the current basic block is terminated
    fn is_current_block_terminated(&self) -> bool {
        if let (Some(block_id), Some(ref function)) = (self.current_block, &self.current_function) {
            if let Some(block) = function.get_block(block_id) {
                return block.is_terminated();
            }
        }
        false
    }

    /// Check if a specific block is terminated (has return/jump/branch as final instruction)
    pub(super) fn is_block_terminated(&self, block_id: super::BasicBlockId) -> bool {
        if let Some(ref function) = self.current_function {
            if let Some(block) = function.get_block(block_id) {
                return block.is_terminated();
            }
        }
        false
    }

    /// Check if a specific block ends with return or throw (not jump/branch)
    /// Used for PHI predecessor判定: jump is reachable, return/throw is unreachable
    pub(super) fn is_block_ends_with_return_or_throw(&self, block_id: super::BasicBlockId) -> bool {
        if let Some(ref function) = self.current_function {
            return crate::mir::phi_core::common::is_unreachable_pred(function, block_id);
        }
        false
    }

    // Note: Legacy ExternCall helper removed; use emit_unified_call with CallTarget::Extern instead.

}


impl Default for MirBuilder {
    fn default() -> Self {
        Self::new()
    }
}
