#![allow(unreachable_patterns, unused_variables)]
use super::builder::{BinOpKind, IRBuilder};
use crate::mir::{ConstValue, MirFunction, MirInstruction, ValueId};

mod analysis;
mod cfg;
mod ops_ext;
mod string_len;

/// Lower(Core-1): Minimal lowering skeleton for Const/Move/BinOp/Cmp/Branch/Ret
/// This does not emit real CLIF yet; it only walks MIR and validates coverage.
pub struct LowerCore {
    pub(crate) unsupported: usize,
    pub(crate) covered: usize,
    /// Minimal constant propagation for i64 to feed host-call args
    pub(super) known_i64: std::collections::HashMap<ValueId, i64>,
    /// Minimal constant propagation for f64 (math.* signature checks)
    pub(super) known_f64: std::collections::HashMap<ValueId, f64>,
    /// Minimal constant propagation for String literals
    pub(super) known_str: std::collections::HashMap<ValueId, String>,
    /// Parameter index mapping for ValueId
    pub(super) param_index: std::collections::HashMap<ValueId, usize>,
    /// Track values produced by Phi (for minimal PHI path)
    pub(super) phi_values: std::collections::HashSet<ValueId>,
    /// Map (block, phi dst) -> param index in that block (for multi-PHI)
    pub(super) phi_param_index:
        std::collections::HashMap<(crate::mir::BasicBlockId, ValueId), usize>,
    /// Track values that are boolean (b1) results, e.g., Compare destinations
    pub(super) bool_values: std::collections::HashSet<ValueId>,
    /// Track PHI destinations that are boolean (all inputs derived from bool_values)
    pub(super) bool_phi_values: std::collections::HashSet<ValueId>,
    /// Track values that are FloatBox instances (for arg type classification)
    pub(super) float_box_values: std::collections::HashSet<ValueId>,
    /// Track values that are plugin handles (generic box/handle, type unknown at compile time)
    pub(super) handle_values: std::collections::HashSet<ValueId>,
    // Per-function statistics (last lowered)
    last_phi_total: u64,
    last_phi_b1: u64,
    last_ret_bool_hint_used: bool,
    // Minimal local slot mapping for Load/Store (ptr ValueId -> slot index)
    pub(super) local_index: std::collections::HashMap<ValueId, usize>,
    pub(super) next_local: usize,
    /// Track NewBox origins: ValueId -> box type name (e.g., "PyRuntimeBox")
    pub(super) box_type_map: std::collections::HashMap<ValueId, String>,
    /// Track StringBox literals: ValueId (NewBox result) -> literal string
    pub(super) string_box_literal: std::collections::HashMap<ValueId, String>,
}

impl LowerCore {
    pub fn new() -> Self {
        Self {
            unsupported: 0,
            covered: 0,
            known_i64: std::collections::HashMap::new(),
            known_f64: std::collections::HashMap::new(),
            known_str: std::collections::HashMap::new(),
            param_index: std::collections::HashMap::new(),
            phi_values: std::collections::HashSet::new(),
            phi_param_index: std::collections::HashMap::new(),
            bool_values: std::collections::HashSet::new(),
            bool_phi_values: std::collections::HashSet::new(),
            float_box_values: std::collections::HashSet::new(),
            handle_values: std::collections::HashSet::new(),
            last_phi_total: 0,
            last_phi_b1: 0,
            last_ret_bool_hint_used: false,
            local_index: std::collections::HashMap::new(),
            next_local: 0,
            box_type_map: std::collections::HashMap::new(),
            string_box_literal: std::collections::HashMap::new(),
        }
    }

    /// Get statistics for the last lowered function
    pub fn last_stats(&self) -> (u64, u64, bool) {
        (
            self.last_phi_total,
            self.last_phi_b1,
            self.last_ret_bool_hint_used,
        )
    }

    /// Walk the MIR function and count supported/unsupported instructions.
    /// In the future, this will build CLIF via Cranelift builders.
    pub fn lower_function(
        &mut self,
        func: &MirFunction,
        builder: &mut dyn IRBuilder,
    ) -> Result<(), String> {
        // Prepare ABI based on MIR signature
        // Reset per-function stats
        self.last_phi_total = 0;
        self.last_phi_b1 = 0;
        self.last_ret_bool_hint_used = false;
        // Build param index map
        self.param_index.clear();
        for (i, v) in func.params.iter().copied().enumerate() {
            self.param_index.insert(v, i);
        }
        // Prepare block mapping (Phase 10.7): deterministic ordering by sorted keys
        let mut bb_ids: Vec<_> = func.blocks.keys().copied().collect();
        bb_ids.sort_by_key(|b| b.0);
        builder.prepare_blocks(bb_ids.len());
        // Pre-seed known_str by scanning all Const(String) ahead of lowering so literal folds work regardless of order
        self.known_str.clear();
        for bb in bb_ids.iter() {
            if let Some(block) = func.blocks.get(bb) {
                for ins in block.instructions.iter() {
                    if let crate::mir::MirInstruction::Const { dst, value } = ins {
                        if let crate::mir::ConstValue::String(s) = value {
                            self.known_str.insert(*dst, s.clone());
                        }
                    }
                }
            }
        }
        self.analyze(func, &bb_ids);
        // Optional: collect PHI targets and ordering per successor for minimal/multi PHI path
        let cfg_now = crate::jit::config::current();
        let enable_phi_min = cfg_now.phi_min;
        // Build successor → phi order and predeclare block params
        let succ_phi_order: std::collections::HashMap<
            crate::mir::BasicBlockId,
            Vec<crate::mir::ValueId>,
        > = self.build_phi_succords(func, &bb_ids, builder, enable_phi_min);
        // Decide ABI: typed or i64-only
        let native_f64 = cfg_now.native_f64;
        let native_bool = cfg_now.native_bool;
        let mut use_typed = false;
        let mut kinds: Vec<super::builder::ParamKind> = Vec::new();
        for mt in func.signature.params.iter() {
            let k = match mt {
                crate::mir::MirType::Float if native_f64 => {
                    use_typed = true;
                    super::builder::ParamKind::F64
                }
                crate::mir::MirType::Bool if native_bool => {
                    use_typed = true;
                    super::builder::ParamKind::B1
                }
                _ => super::builder::ParamKind::I64,
            };
            kinds.push(k);
        }
        let ret_is_f64 =
            native_f64 && matches!(func.signature.return_type, crate::mir::MirType::Float);
        // Hint return bool footing (no-op in current backend; keeps switch point centralized)
        let ret_is_bool = matches!(func.signature.return_type, crate::mir::MirType::Bool);
        if ret_is_bool {
            builder.hint_ret_bool(true);
            // Track how many functions are lowered with boolean return hint (for stats)
            crate::jit::rt::ret_bool_hint_inc(1);
            self.last_ret_bool_hint_used = true;
        }
        let has_ret = !matches!(func.signature.return_type, crate::mir::MirType::Void);
        if std::env::var("NYASH_JIT_TRACE_SIG").ok().as_deref() == Some("1") {
            eprintln!(
                "[SIG-CORE] ret_type={:?} has_ret={} use_typed={} ret_is_f64={}",
                func.signature.return_type, has_ret, use_typed, ret_is_f64
            );
        }
        if use_typed || ret_is_f64 {
            builder.prepare_signature_typed(&kinds, ret_is_f64 && has_ret);
        } else {
            builder.prepare_signature_i64(func.params.len(), has_ret);
        }
        // Pre-scan FloatBox creations across all blocks for arg classification
        self.float_box_values.clear();
        for bb in bb_ids.iter() {
            if let Some(block) = func.blocks.get(bb) {
                for ins in block.instructions.iter() {
                    if let crate::mir::MirInstruction::NewBox { dst, box_type, .. } = ins {
                        if box_type == "FloatBox" {
                            self.float_box_values.insert(*dst);
                        }
                    }
                    if let crate::mir::MirInstruction::Copy { dst, src } = ins {
                        if self.float_box_values.contains(src) {
                            self.float_box_values.insert(*dst);
                        }
                    }
                }
            }
        }

        // Pre-scan to map NewBox origins: ValueId -> box type name; propagate via Copy
        self.box_type_map.clear();
        self.string_box_literal.clear();
        for bb in bb_ids.iter() {
            if let Some(block) = func.blocks.get(bb) {
                for ins in block.instructions.iter() {
                    if let crate::mir::MirInstruction::NewBox { dst, box_type, .. } = ins {
                        self.box_type_map.insert(*dst, box_type.clone());
                    }
                    if let crate::mir::MirInstruction::NewBox {
                        dst,
                        box_type,
                        args,
                    } = ins
                    {
                        if box_type == "StringBox" && args.len() == 1 {
                            let src = args[0];
                            if let Some(s) = self.known_str.get(&src).cloned() {
                                self.string_box_literal.insert(*dst, s);
                            }
                        }
                    }
                    if let crate::mir::MirInstruction::Copy { dst, src } = ins {
                        if let Some(name) = self.box_type_map.get(src).cloned() {
                            self.box_type_map.insert(*dst, name);
                        }
                        if let Some(s) = self.string_box_literal.get(src).cloned() {
                            self.string_box_literal.insert(*dst, s);
                        }
                    }
                }
            }
        }

        builder.begin_function(&func.signature.name);
        // Iterate blocks in the sorted order to keep indices stable
        self.phi_values.clear();
        self.phi_param_index.clear();
        self.float_box_values.clear();
        self.handle_values.clear();
        for (idx, bb_id) in bb_ids.iter().enumerate() {
            let bb = func.blocks.get(bb_id).unwrap();
            builder.switch_to_block(idx);
            // Pre-scan PHIs in this block and ensure block parameters count (multi-PHI)
            if enable_phi_min {
                let mut local_phi_order: Vec<ValueId> = Vec::new();
                // Also detect boolean PHIs: inputs all from boolean-producing values
                for ins in bb.instructions.iter() {
                    if let crate::mir::MirInstruction::Phi { dst, inputs } = ins {
                        local_phi_order.push(*dst);
                        // decide if this phi is boolean
                        if inputs.iter().all(|(_, v)| self.bool_values.contains(v))
                            && !inputs.is_empty()
                        {
                            self.bool_phi_values.insert(*dst);
                        }
                    }
                }
                if !local_phi_order.is_empty() {
                    builder.ensure_block_params_i64(idx, local_phi_order.len());
                    for (i, v) in local_phi_order.into_iter().enumerate() {
                        self.phi_values.insert(v);
                        self.phi_param_index.insert((*bb_id, v), i);
                    }
                }
            }
            for instr in bb.instructions.iter() {
                self.cover_if_supported(instr);
                if let Err(e) = self.try_emit(builder, instr, *bb_id, func) {
                    return Err(e);
                }
                // Track FloatBox creations for later arg classification
                if let crate::mir::MirInstruction::NewBox { dst, box_type, .. } = instr {
                    if box_type == "FloatBox" {
                        self.float_box_values.insert(*dst);
                    }
                }
                if let crate::mir::MirInstruction::Copy { dst, src } = instr {
                    if self.float_box_values.contains(src) {
                        self.float_box_values.insert(*dst);
                    }
                }
            }
            if let Some(term) = &bb.terminator {
                self.cover_if_supported(term);
                // Branch/Jump need block mapping: pass indices
                match term {
                    crate::mir::MirInstruction::Branch {
                        condition,
                        then_bb,
                        else_bb,
                    } => {
                        self.lower_branch_terminator(
                            builder,
                            func,
                            &bb_ids,
                            *bb_id,
                            condition,
                            then_bb,
                            else_bb,
                            &succ_phi_order,
                            enable_phi_min,
                        );
                    }
                    crate::mir::MirInstruction::Jump { target } => {
                        self.lower_jump_terminator(
                            builder,
                            func,
                            &bb_ids,
                            *bb_id,
                            target,
                            &succ_phi_order,
                            enable_phi_min,
                        );
                    }
                    _ => { /* other terminators handled via generic emission below */ }
                }
                // Also allow other terminators to be emitted if needed
                if let Err(e) = self.try_emit(builder, term, *bb_id, func) {
                    return Err(e);
                }
            }
        }
        builder.end_function();
        // Dump CFG/PHI diagnostics
        self.dump_phi_cfg(&succ_phi_order, func, bb_ids.len(), enable_phi_min);
        Ok(())
    }

    // string_len helper moved to core/string_len.rs (no behavior change)

    fn try_emit(
        &mut self,
        b: &mut dyn IRBuilder,
        instr: &MirInstruction,
        cur_bb: crate::mir::BasicBlockId,
        func: &crate::mir::MirFunction,
    ) -> Result<(), String> {
        use crate::mir::MirInstruction as I;
        match instr {
            I::NewBox {
                dst,
                box_type,
                args,
            } => {
                // Materialize StringBox handle at lowering time when literal is known.
                // This enables subsequent BoxCall(len/length) to use a valid runtime handle.
                if box_type == "StringBox" && args.len() == 1 {
                    let src = args[0];
                    // Try from pre-seeded known_str (scanned at function entry)
                    if let Some(s) = self.known_str.get(&src).cloned() {
                        b.emit_string_handle_from_literal(&s);
                        let slot = *self.local_index.entry(*dst).or_insert_with(|| {
                            let id = self.next_local;
                            self.next_local += 1;
                            id
                        });
                        b.store_local_i64(slot);
                        self.handle_values.insert(*dst);
                    }
                } else {
                    // Generic plugin box birth by name via runtime shim: nyash.instance.birth_name_u64x2(lo, hi, len) -> handle
                    // Encode up to 16 bytes of the box type name into two u64 words (little-endian)
                    let name = box_type.as_str();
                    let bytes = name.as_bytes();
                    let take = core::cmp::min(16, bytes.len());
                    let mut lo: u64 = 0;
                    let mut hi: u64 = 0;
                    for i in 0..take.min(8) {
                        lo |= (bytes[i] as u64) << (8 * i as u32);
                    }
                    for i in 8..take {
                        hi |= (bytes[i] as u64) << (8 * (i - 8) as u32);
                    }
                    // Push args and call import
                    b.emit_const_i64(lo as i64);
                    b.emit_const_i64(hi as i64);
                    b.emit_const_i64(bytes.len() as i64);
                    b.emit_host_call(
                        crate::jit::r#extern::birth::SYM_INSTANCE_BIRTH_NAME_U64X2,
                        3,
                        true,
                    );
                    // Store handle to local slot
                    let slot = *self.local_index.entry(*dst).or_insert_with(|| {
                        let id = self.next_local;
                        self.next_local += 1;
                        id
                    });
                    b.store_local_i64(slot);
                    self.handle_values.insert(*dst);
                    // Track type for downstream boxcall routing
                    self.box_type_map.insert(*dst, box_type.clone());
                }
            }
            I::Call {
                dst, func, args, ..
            } => {
                // FunctionBox call shim: emit hostcall nyash_fn_callN(func_h, args...)
                // Push function operand (param or known)
                self.push_value_if_known_or_param(b, func);
                // Push up to 4 args (unknown become iconst 0 via helper)
                for a in args.iter() {
                    self.push_value_if_known_or_param(b, a);
                }
                // Choose symbol by arity
                let argc = args.len();
                let sym = match argc {
                    0 => "nyash_fn_call0",
                    1 => "nyash_fn_call1",
                    2 => "nyash_fn_call2",
                    3 => "nyash_fn_call3",
                    4 => "nyash_fn_call4",
                    5 => "nyash_fn_call5",
                    6 => "nyash_fn_call6",
                    7 => "nyash_fn_call7",
                    _ => "nyash_fn_call8",
                };
                // Emit typed call: all params as I64, returning I64 handle
                // Build param kinds vector: 1 (func) + argc (args)
                let mut params: Vec<crate::jit::lower::builder::ParamKind> = Vec::new();
                params.push(crate::jit::lower::builder::ParamKind::I64);
                for _ in 0..core::cmp::min(argc, 8) {
                    params.push(crate::jit::lower::builder::ParamKind::I64);
                }
                b.emit_host_call_typed(sym, &params, true, false);
                // Persist or discard the return to keep the stack balanced
                if let Some(d) = dst {
                    self.handle_values.insert(*d);
                    let slot = *self.local_index.entry(*d).or_insert_with(|| {
                        let id = self.next_local;
                        self.next_local += 1;
                        id
                    });
                    b.store_local_i64(slot);
                } else {
                    // No destination: spill to scratch local to consume the value
                    let scratch = {
                        let id = self.next_local;
                        self.next_local += 1;
                        id
                    };
                    b.store_local_i64(scratch);
                }
            }
            I::Await { dst, future } => {
                // Push future param index when known; otherwise -1 to trigger legacy search in shim
                if let Some(pidx) = self.param_index.get(future).copied() {
                    b.emit_param_i64(pidx);
                } else {
                    b.emit_const_i64(-1);
                }
                // Call await_h to obtain a handle to the value (0 on timeout)
                b.emit_host_call(crate::jit::r#extern::r#async::SYM_FUTURE_AWAIT_H, 1, true);
                // Store the awaited handle temporarily
                let hslot = {
                    let id = self.next_local;
                    self.next_local += 1;
                    id
                };
                b.store_local_i64(hslot);
                // Build Ok result: ok_h(handle)
                b.load_local_i64(hslot);
                b.emit_host_call(crate::jit::r#extern::result::SYM_RESULT_OK_H, 1, true);
                let ok_slot = {
                    let id = self.next_local;
                    self.next_local += 1;
                    id
                };
                b.store_local_i64(ok_slot);
                // Build Err result: err_h(0) → Timeout
                b.emit_const_i64(0);
                b.emit_host_call(crate::jit::r#extern::result::SYM_RESULT_ERR_H, 1, true);
                let err_slot = {
                    let id = self.next_local;
                    self.next_local += 1;
                    id
                };
                b.store_local_i64(err_slot);
                // Cond: (handle == 0)
                b.load_local_i64(hslot);
                b.emit_const_i64(0);
                b.emit_compare(crate::jit::lower::builder::CmpKind::Eq);
                // Stack for select: cond, then(err), else(ok)
                b.load_local_i64(err_slot);
                b.load_local_i64(ok_slot);
                b.emit_select_i64();
                // Store selected Result handle to destination
                let d = *dst;
                self.handle_values.insert(d);
                let slot = *self.local_index.entry(d).or_insert_with(|| {
                    let id = self.next_local;
                    self.next_local += 1;
                    id
                });
                b.store_local_i64(slot);
            }
            I::Safepoint => {
                // Emit a runtime checkpoint (safepoint + scheduler poll via NyRT/JIT stubs)
                b.emit_host_call(crate::jit::r#extern::runtime::SYM_RT_CHECKPOINT, 0, false);
            }
            I::RefGet {
                dst,
                reference: _,
                field,
            } => {
                // Minimal: env.console をハンドル化（hostcall）
                if field == "console" {
                    // Emit hostcall to create/get ConsoleBox handle
                    // Symbol exported by nyrt: nyash.console.birth_h
                    b.emit_host_call(
                        crate::jit::r#extern::collections::SYM_CONSOLE_BIRTH_H,
                        0,
                        true,
                    );
                } else {
                    // Unknown RefGet: treat as no-op const 0 to avoid strict fail for now
                    b.emit_const_i64(0);
                }
                // Record as covered; do not increment unsupported
                let _ = dst; // keep signature parity
            }
            I::UnaryOp {
                dst: _,
                op,
                operand,
            } => {
                match op {
                    crate::mir::UnaryOp::Neg => {
                        // i64-only minimal: 0 - operand
                        // Try known const or param
                        // push 0
                        b.emit_const_i64(0);
                        // push operand (known/param)
                        self.push_value_if_known_or_param(b, operand);
                        b.emit_binop(BinOpKind::Sub);
                    }
                    _ => {
                        self.unsupported += 1;
                    }
                }
            }
            I::NewBox {
                dst,
                box_type,
                args,
            } => {
                // 最適化は後段へ（現状は汎用・安全な実装に徹する）
                // 通常経路:
                // - 引数なし: 汎用 birth_h（type_idのみ）でハンドル生成
                // - 引数あり: 既存のチェーン（直後の plugin_invoke birth で初期化）を維持（段階的導入）
                if args.is_empty() {
                    // 文字列の型名からインスタンスを生成（グローバルUnifiedRegistry経由）
                    // name → u64x2 パックで渡す
                    let name = box_type.clone();
                    {
                        let name_bytes = name.as_bytes();
                        let mut lo: u64 = 0;
                        let mut hi: u64 = 0;
                        let take = core::cmp::min(16, name_bytes.len());
                        for i in 0..take.min(8) {
                            lo |= (name_bytes[i] as u64) << (8 * i as u32);
                        }
                        for i in 8..take {
                            hi |= (name_bytes[i] as u64) << (8 * (i - 8) as u32);
                        }
                        // Push immediates
                        b.emit_const_i64(lo as i64);
                        b.emit_const_i64(hi as i64);
                        b.emit_const_i64(name_bytes.len() as i64);
                        // Call import (lo, hi, len) -> handle
                        // Use typed hostcall (I64,I64,I64)->I64
                        b.emit_host_call_typed(
                            crate::jit::r#extern::birth::SYM_INSTANCE_BIRTH_NAME_U64X2,
                            &[
                                crate::jit::lower::builder::ParamKind::I64,
                                crate::jit::lower::builder::ParamKind::I64,
                                crate::jit::lower::builder::ParamKind::I64,
                            ],
                            true,
                            false,
                        );
                        self.handle_values.insert(*dst);
                        let slot = *self.local_index.entry(*dst).or_insert_with(|| {
                            let id = self.next_local;
                            self.next_local += 1;
                            id
                        });
                        b.store_local_i64(slot);
                    }
                } else {
                    // 引数あり: 安全なパターンから段階的に birth_i64 に切替
                    // 1) IntegerBox(const i64)
                    if box_type == "IntegerBox" && args.len() == 1 {
                        if let Some(src) = args.get(0) {
                            if let Some(iv) = self.known_i64.get(src).copied() {
                                // 汎用 birth_i64(type_id, argc=1, a1=iv)
                                if let crate::jit::policy::invoke::InvokeDecision::PluginInvoke {
                                    type_id,
                                    ..
                                } = crate::jit::policy::invoke::decide_box_method(
                                    box_type, "birth", 1, true,
                                ) {
                                    b.emit_const_i64(type_id as i64);
                                    b.emit_const_i64(1);
                                    b.emit_const_i64(iv);
                                    b.emit_host_call("nyash.box.birth_i64", 3, true);
                                    self.handle_values.insert(*dst);
                                    let slot = *self.local_index.entry(*dst).or_insert_with(|| {
                                        let id = self.next_local;
                                        self.next_local += 1;
                                        id
                                    });
                                    b.store_local_i64(slot);
                                    // 値伝搬も継続
                                    self.known_i64.insert(*dst, iv);
                                    return Ok(());
                                }
                            }
                        }
                    }
                    // 2) StringBox(const string) → 文字列リテラルから直接ハンドル生成
                    if box_type == "StringBox" && args.len() == 1 {
                        if let Some(src) = args.get(0) {
                            // 探索: 同一関数内で src を定義する Const(String)
                            let mut lit: Option<String> = None;
                            for (_bid, bb) in func.blocks.iter() {
                                for ins in bb.instructions.iter() {
                                    if let crate::mir::MirInstruction::Const { dst: cdst, value } =
                                        ins
                                    {
                                        if cdst == src {
                                            if let crate::mir::ConstValue::String(s) = value {
                                                lit = Some(s.clone());
                                            }
                                            break;
                                        }
                                    }
                                }
                                if lit.is_some() {
                                    break;
                                }
                            }
                            if let Some(s) = lit {
                                b.emit_string_handle_from_literal(&s);
                                self.handle_values.insert(*dst);
                                let slot = *self.local_index.entry(*dst).or_insert_with(|| {
                                    let id = self.next_local;
                                    self.next_local += 1;
                                    id
                                });
                                b.store_local_i64(slot);
                                return Ok(());
                            }
                        }
                    }
                    // 2) 引数がハンドル（StringBox等）で既に存在する場合（最大2引数）
                    if args.len() <= 2 && args.iter().all(|a| self.handle_values.contains(a)) {
                        if let crate::jit::policy::invoke::InvokeDecision::PluginInvoke {
                            type_id,
                            ..
                        } = crate::jit::policy::invoke::decide_box_method(
                            box_type,
                            "birth",
                            args.len(),
                            true,
                        ) {
                            b.emit_const_i64(type_id as i64);
                            b.emit_const_i64(args.len() as i64);
                            // a1, a2 を push（ローカルに保存済みのハンドルをロード）
                            for a in args.iter().take(2) {
                                self.push_value_if_known_or_param(b, a);
                            }
                            b.emit_host_call("nyash.box.birth_i64", 2 + args.len(), true);
                            self.handle_values.insert(*dst);
                            let slot = *self.local_index.entry(*dst).or_insert_with(|| {
                                let id = self.next_local;
                                self.next_local += 1;
                                id
                            });
                            b.store_local_i64(slot);
                            return Ok(());
                        }
                    }
                    // フォールバック: 既存チェーンに委譲（互換）+ 既知値伝搬のみ
                    if box_type == "IntegerBox" {
                        if let Some(src) = args.get(0) {
                            if let Some(iv) = self.known_i64.get(src).copied() {
                                self.known_i64.insert(*dst, iv);
                            }
                        }
                    }
                }
                // Track boxed numeric literals to aid signature checks (FloatBox/IntegerBox)
                if box_type == "FloatBox" {
                    if let Some(src) = args.get(0) {
                        if let Some(fv) = self.known_f64.get(src).copied() {
                            self.known_f64.insert(*dst, fv);
                        } else if let Some(iv) = self.known_i64.get(src).copied() {
                            self.known_f64.insert(*dst, iv as f64);
                        }
                    }
                } else if box_type == "IntegerBox" {
                    if let Some(src) = args.get(0) {
                        if let Some(iv) = self.known_i64.get(src).copied() {
                            self.known_i64.insert(*dst, iv);
                        }
                    }
                }
            }
            I::PluginInvoke {
                dst,
                box_val,
                method,
                args,
                ..
            } => {
                self.lower_plugin_invoke(b, &dst, &box_val, method.as_str(), args, func)?;
            }
            I::ExternCall {
                dst,
                iface_name,
                method_name,
                args,
                ..
            } => {
                self.lower_extern_call(
                    b,
                    &dst,
                    iface_name.as_str(),
                    method_name.as_str(),
                    args,
                    func,
                )?;
            }
            I::Cast {
                dst,
                value,
                target_type,
            } => {
                // Minimal cast footing: materialize source when param/known
                // Bool→Int: rely on producers (compare) and branch/b1 loaders; here we just reuse integer path
                self.push_value_if_known_or_param(b, value);
                // Track known i64 if source known
                if let Some(v) = self.known_i64.get(value).copied() {
                    self.known_i64.insert(*dst, v);
                }
                // Track known f64 for float casts
                if matches!(target_type, crate::mir::MirType::Float) {
                    if let Some(iv) = self.known_i64.get(value).copied() {
                        self.known_f64.insert(*dst, iv as f64);
                    }
                }
            }
            I::Const { dst, value } => match value {
                ConstValue::Integer(i) => {
                    b.emit_const_i64(*i);
                    self.known_i64.insert(*dst, *i);
                }
                ConstValue::Float(f) => {
                    b.emit_const_f64(*f);
                    self.known_f64.insert(*dst, *f);
                }
                ConstValue::Bool(bv) => {
                    let iv = if *bv { 1 } else { 0 };
                    b.emit_const_i64(iv);
                    self.known_i64.insert(*dst, iv);
                    // Mark this value as boolean producer
                    self.bool_values.insert(*dst);
                }
                ConstValue::String(sv) => {
                    self.known_str.insert(*dst, sv.clone());
                }
                ConstValue::Null | ConstValue::Void => {}
            },
            I::Copy { dst, src } => {
                if let Some(v) = self.known_i64.get(src).copied() {
                    self.known_i64.insert(*dst, v);
                }
                if let Some(v) = self.known_f64.get(src).copied() {
                    self.known_f64.insert(*dst, v);
                }
                if let Some(v) = self.known_str.get(src).cloned() {
                    self.known_str.insert(*dst, v);
                }
                // Propagate handle/type knowledge to keep BoxCall routing stable across copies
                if self.handle_values.contains(src) {
                    self.handle_values.insert(*dst);
                }
                if let Some(bt) = self.box_type_map.get(src).cloned() {
                    self.box_type_map.insert(*dst, bt);
                }
                // Propagate boolean classification through Copy
                if self.bool_values.contains(src) {
                    self.bool_values.insert(*dst);
                }
                // If source is a parameter, materialize it on the stack for downstream ops and persist into dst slot
                if let Some(pidx) = self.param_index.get(src).copied() {
                    b.emit_param_i64(pidx);
                    let slot = *self.local_index.entry(*dst).or_insert_with(|| {
                        let id = self.next_local;
                        self.next_local += 1;
                        id
                    });
                    b.ensure_local_i64(slot);
                    b.store_local_i64(slot);
                } else if let Some(src_slot) = self.local_index.get(src).copied() {
                    // If source already has a local slot (e.g., a handle), copy into dst's slot
                    b.load_local_i64(src_slot);
                    let dst_slot = *self.local_index.entry(*dst).or_insert_with(|| {
                        let id = self.next_local;
                        self.next_local += 1;
                        id
                    });
                    b.ensure_local_i64(dst_slot);
                    b.store_local_i64(dst_slot);
                }
            }
            I::BinOp { dst, op, lhs, rhs } => {
                self.lower_binop(b, op, lhs, rhs, dst, func);
            }
            I::Compare { op, lhs, rhs, dst } => {
                self.lower_compare(b, op, lhs, rhs, dst, func);
            }
            I::Jump { .. } => self.lower_jump(b),
            I::Branch { .. } => self.lower_branch(b),
            I::Return { value } => {
                if let Some(v) = value {
                    if std::env::var("NYASH_JIT_TRACE_RET").ok().as_deref() == Some("1") {
                        eprintln!(
                            "[LOWER] Return value={:?} known_i64?={} param?={} local?={}",
                            v,
                            self.known_i64.contains_key(v),
                            self.param_index.contains_key(v),
                            self.local_index.contains_key(v)
                        );
                    }
                    // 1) Prefer known constants first to avoid stale locals overshadowing folded values
                    if let Some(k) = self.known_i64.get(v).copied() {
                        if std::env::var("NYASH_JIT_TRACE_RET").ok().as_deref() == Some("1") {
                            eprintln!("[LOWER] Return known_i64 value for {:?} = {}", v, k);
                        }
                        // Emit the constant and also persist to a stable local slot for this value id,
                        // then reload to ensure a value remains on the stack for emit_return.
                        b.emit_const_i64(k);
                        let rslot = *self.local_index.entry(*v).or_insert_with(|| {
                            let id = self.next_local;
                            self.next_local += 1;
                            id
                        });
                        b.ensure_local_i64(rslot);
                        b.store_local_i64(rslot);
                        b.load_local_i64(rslot);
                    }
                    // 2) Prefer existing locals/params
                    else if self.local_index.get(v).is_some() || self.param_index.get(v).is_some()
                    {
                        self.push_value_if_known_or_param(b, v);
                    } else {
                        // 3) Backward scan and minimal reconstruction for common producers
                        if let Some(bb) = func.blocks.get(&cur_bb) {
                            // Follow Copy chains backwards to original producer where possible
                            let mut want = *v;
                            let mut produced = false;
                            for ins in bb.instructions.iter().rev() {
                                match ins {
                                    crate::mir::MirInstruction::Copy { dst, src }
                                        if dst == &want =>
                                    {
                                        want = *src;
                                        // Try early exit if known/local/param emerges
                                        if self.known_i64.get(&want).is_some() {
                                            b.emit_const_i64(*self.known_i64.get(&want).unwrap());
                                            produced = true;
                                            break;
                                        }
                                        if self.local_index.get(&want).is_some()
                                            || self.param_index.get(&want).is_some()
                                        {
                                            self.push_value_if_known_or_param(b, &want);
                                            produced = true;
                                            break;
                                        }
                                    }
                                    // StringBox.len/length: re-materialize robustly if not saved
                                    crate::mir::MirInstruction::BoxCall {
                                        dst: Some(did),
                                        box_val,
                                        method,
                                        args,
                                        ..
                                    } if did == &want => {
                                        let m = method.as_str();
                                        if m == "len" || m == "length" {
                                            // Prefer param/local handle, else reconstruct literal
                                            if let Some(pidx) =
                                                self.param_index.get(box_val).copied()
                                            {
                                                self.emit_len_with_fallback_param(b, pidx);
                                                produced = true;
                                                break;
                                            } else if let Some(slot) =
                                                self.local_index.get(box_val).copied()
                                            {
                                                self.emit_len_with_fallback_local_handle(b, slot);
                                                produced = true;
                                                break;
                                            } else {
                                                // Try literal reconstruction via known_str map
                                                let mut lit: Option<String> = None;
                                                for (_bid2, bb2) in func.blocks.iter() {
                                                    for ins2 in bb2.instructions.iter() {
                                                        if let crate::mir::MirInstruction::NewBox { dst, box_type, args } = ins2 {
                                                            if dst == box_val && box_type == "StringBox" && args.len() == 1 {
                                                                let src = args[0];
                                                                if let Some(s) = self.known_str.get(&src).cloned() { lit = Some(s); break; }
                                                            }
                                                        }
                                                    }
                                                    if lit.is_some() {
                                                        break;
                                                    }
                                                }
                                                if let Some(s) = lit {
                                                    self.emit_len_with_fallback_literal(b, &s);
                                                    produced = true;
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    // Const producer as last resort
                                    crate::mir::MirInstruction::Const { dst, value: cval }
                                        if dst == &want =>
                                    {
                                        match cval {
                                            crate::mir::ConstValue::Integer(i) => {
                                                b.emit_const_i64(*i);
                                            }
                                            crate::mir::ConstValue::Bool(bv) => {
                                                b.emit_const_i64(if *bv { 1 } else { 0 });
                                            }
                                            crate::mir::ConstValue::Float(f) => {
                                                b.emit_const_f64(*f);
                                            }
                                            _ => {}
                                        }
                                        produced = true;
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                            if !produced {
                                // 4) Final fallback: try pushing as param/local again (no-op if not found)
                                self.push_value_if_known_or_param(b, &want);
                            }
                        }
                    }
                }
                b.emit_return()
            }
            I::Store { value, ptr } => {
                // Minimal lowering: materialize value if known/param and store to a local slot keyed by ptr
                self.push_value_if_known_or_param(b, value);
                let slot = *self.local_index.entry(*ptr).or_insert_with(|| {
                    let id = self.next_local;
                    self.next_local += 1;
                    id
                });
                b.ensure_local_i64(slot);
                b.store_local_i64(slot);
            }
            I::Load { dst, ptr } => {
                // Minimal lowering: load from local slot keyed by ptr, then materialize into dst's own slot
                let src_slot = *self.local_index.entry(*ptr).or_insert_with(|| {
                    let id = self.next_local;
                    self.next_local += 1;
                    id
                });
                b.ensure_local_i64(src_slot);
                b.load_local_i64(src_slot);
                // Persist into dst's slot to make subsequent uses find it via local_index
                let dst_slot = *self.local_index.entry(*dst).or_insert_with(|| {
                    let id = self.next_local;
                    self.next_local += 1;
                    id
                });
                b.ensure_local_i64(dst_slot);
                b.store_local_i64(dst_slot);
            }
            I::Phi { dst, .. } => {
                // PHI をローカルに materialize して後続の Return で安定参照
                let pos = self
                    .phi_param_index
                    .get(&(cur_bb, *dst))
                    .copied()
                    .unwrap_or(0);
                if self.bool_phi_values.contains(dst) {
                    b.push_block_param_b1_at(pos);
                } else {
                    b.push_block_param_i64_at(pos);
                }
                let slot = *self.local_index.entry(*dst).or_insert_with(|| {
                    let id = self.next_local;
                    self.next_local += 1;
                    id
                });
                b.ensure_local_i64(slot);
                b.store_local_i64(slot);
            }
            I::ArrayGet { dst, array, index } => {
                // Prepare receiver + index on stack
                let argc = 2usize;
                if let Some(pidx) = self.param_index.get(array).copied() {
                    b.emit_param_i64(pidx);
                } else {
                    b.emit_const_i64(-1);
                }
                if let Some(iv) = self.known_i64.get(index).copied() {
                    b.emit_const_i64(iv);
                } else {
                    self.push_value_if_known_or_param(b, index);
                }
                // Decide policy
                let decision =
                    crate::jit::policy::invoke::decide_box_method("ArrayBox", "get", argc, true);
                match decision {
                    crate::jit::policy::invoke::InvokeDecision::PluginInvoke {
                        type_id,
                        method_id,
                        box_type,
                        ..
                    } => {
                        b.emit_plugin_invoke(type_id, method_id, argc, true);
                        crate::jit::observe::lower_plugin_invoke(
                            &box_type, "get", type_id, method_id, argc,
                        );
                        // Persist into dst's slot
                        let dslot = *self.local_index.entry(*dst).or_insert_with(|| {
                            let id = self.next_local;
                            self.next_local += 1;
                            id
                        });
                        b.store_local_i64(dslot);
                    }
                    crate::jit::policy::invoke::InvokeDecision::HostCall { symbol, .. } => {
                        crate::jit::observe::lower_hostcall(
                            &symbol,
                            argc,
                            &["Handle", "I64"],
                            "allow",
                            "mapped_symbol",
                        );
                        b.emit_host_call(&symbol, argc, true);
                        let dslot = *self.local_index.entry(*dst).or_insert_with(|| {
                            let id = self.next_local;
                            self.next_local += 1;
                            id
                        });
                        b.store_local_i64(dslot);
                    }
                    _ => {
                        super::core_hostcall::lower_array_get(
                            b,
                            &self.param_index,
                            &self.known_i64,
                            array,
                            index,
                        );
                        let dslot = *self.local_index.entry(*dst).or_insert_with(|| {
                            let id = self.next_local;
                            self.next_local += 1;
                            id
                        });
                        b.store_local_i64(dslot);
                    }
                }
            }
            I::ArraySet {
                array,
                index,
                value,
            } => {
                let argc = 3usize;
                if let Some(pidx) = self.param_index.get(array).copied() {
                    b.emit_param_i64(pidx);
                } else {
                    b.emit_const_i64(-1);
                }
                // GC write barrier hint for mutating array operations (pass receiver handle/index as site id: receiver preferred)
                b.emit_host_call(
                    crate::jit::r#extern::runtime::SYM_GC_BARRIER_WRITE,
                    1,
                    false,
                );
                if let Some(iv) = self.known_i64.get(index).copied() {
                    b.emit_const_i64(iv);
                } else {
                    self.push_value_if_known_or_param(b, index);
                }
                if let Some(vv) = self.known_i64.get(value).copied() {
                    b.emit_const_i64(vv);
                } else {
                    self.push_value_if_known_or_param(b, value);
                }
                let decision =
                    crate::jit::policy::invoke::decide_box_method("ArrayBox", "set", argc, false);
                match decision {
                    crate::jit::policy::invoke::InvokeDecision::PluginInvoke {
                        type_id,
                        method_id,
                        box_type,
                        ..
                    } => {
                        b.emit_plugin_invoke(type_id, method_id, argc, false);
                        crate::jit::observe::lower_plugin_invoke(
                            &box_type, "set", type_id, method_id, argc,
                        );
                    }
                    crate::jit::policy::invoke::InvokeDecision::HostCall { symbol, .. } => {
                        crate::jit::observe::lower_hostcall(
                            &symbol,
                            argc,
                            &["Handle", "I64", "I64"],
                            "allow",
                            "mapped_symbol",
                        );
                        b.emit_host_call(&symbol, argc, false);
                    }
                    _ => super::core_hostcall::lower_array_set(
                        b,
                        &self.param_index,
                        &self.known_i64,
                        array,
                        index,
                        value,
                    ),
                }
            }
            I::BoxCall {
                box_val: array,
                method,
                args,
                dst,
                ..
            } => {
                // Prefer ops_ext; if not handled, fall back to legacy path below
                let trace = std::env::var("NYASH_JIT_TRACE_LOWER").ok().as_deref() == Some("1");
                // Handle ArrayBox.set with handle-valued value for literal strings
                if method == "set"
                    && self
                        .box_type_map
                        .get(&array)
                        .map(|s| s == "ArrayBox")
                        .unwrap_or(false)
                {
                    // Expect args: [index, value]
                    let argc = 3usize;
                    // Receiver handle: prefer param or local slot; else -1 sentinel
                    if let Some(pidx) = self.param_index.get(array).copied() {
                        b.emit_param_i64(pidx);
                    } else if let Some(slot) = self.local_index.get(&array).copied() {
                        b.load_local_i64(slot);
                    } else {
                        b.emit_const_i64(-1);
                    }
                    // Index as i64
                    if let Some(idx_v) = args.get(0) {
                        if let Some(iv) = self.known_i64.get(idx_v).copied() {
                            b.emit_const_i64(iv);
                        } else {
                            self.push_value_if_known_or_param(b, idx_v);
                        }
                    } else {
                        b.emit_const_i64(0);
                    }
                    // Value as handle: for String literal, synthesize a handle; else prefer param/local handle
                    if let Some(val_v) = args.get(1) {
                        let mut emitted_val_handle = false;
                        if let Some(s) = self.known_str.get(val_v).cloned() {
                            b.emit_string_handle_from_literal(&s);
                            emitted_val_handle = true;
                        } else if let Some(slot) = self.local_index.get(val_v).copied() {
                            b.load_local_i64(slot);
                            emitted_val_handle = true;
                        } else if let Some(pidx) = self.param_index.get(val_v).copied() {
                            b.emit_param_i64(pidx);
                            emitted_val_handle = true;
                        }
                        if !emitted_val_handle {
                            b.emit_const_i64(0);
                        }
                    } else {
                        b.emit_const_i64(0);
                    }
                    // Emit handle-handle variant hostcall
                    b.emit_host_call(
                        crate::jit::r#extern::collections::SYM_ARRAY_SET_HH,
                        argc,
                        false,
                    );
                    if trace {
                        eprintln!("[LOWER] BoxCall(ArrayBox.set) → ARRAY_SET_HH");
                    }
                    return Ok(());
                }
                // Early constant fold: StringBox literal length/len (allow disabling via NYASH_JIT_DISABLE_LEN_CONST=1)
                if std::env::var("NYASH_JIT_DISABLE_LEN_CONST").ok().as_deref() != Some("1")
                    && (method == "len" || method == "length")
                    && self
                        .box_type_map
                        .get(&array)
                        .map(|s| s == "StringBox")
                        .unwrap_or(false)
                {
                    if let Some(s) = self.string_box_literal.get(&array).cloned() {
                        let n = s.len() as i64;
                        b.emit_const_i64(n);
                        if let Some(d) = dst {
                            let slot = *self.local_index.entry(*d).or_insert_with(|| {
                                let id = self.next_local;
                                self.next_local += 1;
                                id
                            });
                            b.store_local_i64(slot);
                            self.known_i64.insert(*d, n);
                        }
                        if trace {
                            eprintln!("[LOWER] early const-fold StringBox.{} = {}", method, n);
                        }
                        return Ok(());
                    }
                }
                let handled =
                    self.lower_box_call(func, b, &array, method.as_str(), args, dst.clone())?;
                if trace {
                    eprintln!(
                        "[LOWER] BoxCall recv={:?} method={} handled={} box_type={:?} dst?={}",
                        array,
                        method,
                        handled,
                        self.box_type_map.get(&array),
                        dst.is_some()
                    );
                    if !handled {
                        let bt = self.box_type_map.get(&array).cloned().unwrap_or_default();
                        let is_param = self.param_index.contains_key(&array);
                        let has_local = self.local_index.contains_key(&array);
                        let is_handle = self.handle_values.contains(&array);
                        // Classify up to first 3 args: i(known_i64) s(known_str) p(param) l(local) h(handle) -(unknown)
                        let mut arg_kinds: Vec<&'static str> = Vec::new();
                        for a in args.iter().take(3) {
                            let k = if self.known_i64.contains_key(a) {
                                "i"
                            } else if self.known_str.contains_key(a) {
                                "s"
                            } else if self.param_index.contains_key(a) {
                                "p"
                            } else if self.local_index.contains_key(a) {
                                "l"
                            } else if self.handle_values.contains(a) {
                                "h"
                            } else {
                                "-"
                            };
                            arg_kinds.push(k);
                        }
                        // Policy hint: whether a mapped HostCall exists for (box_type, method)
                        let policy = crate::jit::policy::invoke::decide_box_method(
                            &bt,
                            method.as_str(),
                            1 + args.len(),
                            dst.is_some(),
                        );
                        let policy_str = match policy {
                            crate::jit::policy::invoke::InvokeDecision::HostCall {
                                ref symbol,
                                ..
                            } => format!("hostcall:{}", symbol),
                            crate::jit::policy::invoke::InvokeDecision::PluginInvoke { .. } => {
                                "plugin_invoke".to_string()
                            }
                            crate::jit::policy::invoke::InvokeDecision::Fallback { ref reason } => {
                                format!("fallback:{}", reason)
                            }
                        };
                        eprintln!(
                            "[LOWER] fallback(reason=unhandled) box_type='{}' method='{}' recv[param?{} local?{} handle?{}] args={:?} policy={}",
                            bt, method, is_param, has_local, is_handle, arg_kinds, policy_str
                        );
                    }
                }
                if handled {
                    return Ok(());
                }
            }
            /* legacy BoxCall branch removed (now handled in ops_ext)
                // handled in helper (read-only simple methods)
            } else if matches!(method.as_str(), "sin" | "cos" | "abs" | "min" | "max") {
                super::core_hostcall::lower_math_call(
                    func,
                    b,
                    &self.known_i64,
                    &self.known_f64,
                    &self.float_box_values,
                    method.as_str(),
                    args,
                    dst.clone(),
                );
            } else if false /* moved to ops_ext: NYASH_USE_PLUGIN_BUILTINS */ {
                // StringBox（length/is_empty/charCodeAt）: policy+observe経由に統一
                if matches!(method.as_str(), "length" | "is_empty" | "charCodeAt") {
                    // receiver
                    if let Some(pidx) = self.param_index.get(array).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
                    let mut argc = 1usize;
                    if method.as_str() == "charCodeAt" {
                        if let Some(v) = args.get(0) { self.push_value_if_known_or_param(b, v); } else { b.emit_const_i64(0); }
                        argc = 2;
                    }
                    if method.as_str() == "is_empty" { b.hint_ret_bool(true); }
                    let decision = crate::jit::policy::invoke::decide_box_method("StringBox", method.as_str(), argc, dst.is_some());
                    match decision {
                        crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, method_id, box_type, .. } => {
                            b.emit_plugin_invoke(type_id, method_id, argc, dst.is_some());
                            crate::jit::observe::lower_plugin_invoke(&box_type, method.as_str(), type_id, method_id, argc);
                            return Ok(());
                        }
                        crate::jit::policy::invoke::InvokeDecision::HostCall { symbol, .. } => {
                            crate::jit::observe::lower_hostcall(&symbol, argc, &if argc==1 { ["Handle"][..].to_vec() } else { ["Handle","I64"][..].to_vec() }, "allow", "mapped_symbol");
                            b.emit_host_call(&symbol, argc, dst.is_some());
                            return Ok(());
                        }
                        _ => {}
                    }
                }
                // Integer.get/set specialized when receiver is Integer (avoid Map collision)
                if matches!(method.as_str(), "get" | "set") {
                    let recv_is_int = func.metadata.value_types.get(array).map(|mt| matches!(mt, crate::mir::MirType::Integer)).unwrap_or(false);
                    if recv_is_int {
                        if let Ok(ph) = crate::runtime::plugin_loader_unified::get_global_plugin_host().read() {
                            if let Ok(h) = ph.resolve_method("IntegerBox", method.as_str()) {
                                if let Some(pidx) = self.param_index.get(array).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
                                let mut argc = 1usize;
                                if method.as_str() == "set" {
                                    if let Some(v) = args.get(0) { self.push_value_if_known_or_param(b, v); } else { b.emit_const_i64(0); }
                                    argc = 2;
                                }
                                b.emit_plugin_invoke(h.type_id, h.method_id, argc, dst.is_some());
                                crate::jit::events::emit_lower(
                                    serde_json::json!({
                                        "id": format!("plugin:{}:{}", h.box_type, method.as_str()),
                                        "decision":"allow","reason":"plugin_invoke","argc": argc,
                                        "type_id": h.type_id, "method_id": h.method_id
                                    }),
                                    "plugin","<jit>"
                                );
                                return Ok(());
                            }
                        }
                    }
                }
                match method.as_str() {
                    "len" | "length" => {
                        // Resolve ArrayBox plugin method and emit plugin_invoke (symbolic)
                        if let Ok(ph) = crate::runtime::plugin_loader_unified::get_global_plugin_host().read() {
                            let mname = "length";
                            if let Ok(h) = ph.resolve_method("ArrayBox", mname) {
                                // Receiver
                                if let Some(pidx) = self.param_index.get(array).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
                                let mut argc = 1usize;
                                // length only
                                b.emit_plugin_invoke(h.type_id, h.method_id, argc, dst.is_some());
                                crate::jit::events::emit_lower(
                                    serde_json::json!({
                                        "id": format!("plugin:{}:{}", h.box_type, mname),
                                        "decision":"allow","reason":"plugin_invoke","argc": argc,
                                        "type_id": h.type_id, "method_id": h.method_id
                                    }),
                                    "plugin","<jit>"
                                );
                            }
                        }
                    }
                    // Map: size/get/has (RO) and set (mutating; allowed only when policy.read_only=false)
                    "size" | "get" | "has" | "set" => {
                        if let Ok(ph) = crate::runtime::plugin_loader_unified::get_global_plugin_host().read() {
                            if let Ok(h) = ph.resolve_method("MapBox", method.as_str()) {
                                if method.as_str() == "set" && crate::jit::policy::current().read_only {
                                    // Deny mutating under read-only policy
                                    crate::jit::events::emit_lower(
                                        serde_json::json!({
                                            "id": format!("plugin:{}:{}", "MapBox", "set"),
                                            "decision":"fallback","reason":"policy_denied_mutating"
                                        }),
                                        "plugin","<jit>"
                                    );
                                    // Do not emit plugin call; VM path will handle
                                    return Ok(());
                                }
                                if let Some(pidx) = self.param_index.get(array).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
                                // Insert GC write barrier before mutating Map.set
                                if method.as_str() == "set" {
                                    b.emit_host_call(crate::jit::r#extern::runtime::SYM_GC_BARRIER_WRITE, 1, false);
                                }
                                let mut argc = 1usize;
                                if matches!(method.as_str(), "get" | "has") {
                                    if let Some(v) = args.get(0) { self.push_value_if_known_or_param(b, v); } else { b.emit_const_i64(0); }
                                    argc += 1;
                                } else if method.as_str() == "set" {
                                    if let Some(k) = args.get(0) { self.push_value_if_known_or_param(b, k); } else { b.emit_const_i64(0); }
                                    if let Some(v) = args.get(1) { self.push_value_if_known_or_param(b, v); } else { b.emit_const_i64(0); }
                                    argc += 2;
                                }
                                b.emit_plugin_invoke(h.type_id, h.method_id, argc, dst.is_some());
                                crate::jit::events::emit_lower(
                                    serde_json::json!({
                                        "id": format!("plugin:{}:{}", h.box_type, method.as_str()),
                                        "decision":"allow","reason":"plugin_invoke","argc": argc,
                                        "type_id": h.type_id, "method_id": h.method_id
                                    }),
                                    "plugin","<jit>"
                                );
                            }
                        }
                    }
                    _ => { /* other BoxCalls handled below */ }
                }
            } else if crate::jit::config::current().hostcall {
                match method.as_str() {
                    "len" | "length" => {
                        // Constant fold: if receiver is NewBox(StringBox, Const String), return its length directly
                        if let Some(did) = dst.as_ref() {
                            let mut lit_len: Option<i64> = None;
                            for (_bid, bb) in func.blocks.iter() {
                                for ins in bb.instructions.iter() {
                                    if let crate::mir::MirInstruction::NewBox { dst: ndst, box_type, args } = ins {
                                        if ndst == array && box_type == "StringBox" && args.len() == 1 {
                                            let src = args[0];
                                            if let Some(s) = self.known_str.get(&src) { lit_len = Some(s.len() as i64); break; }
                                            // scan Const directly
                                            for (_b2, bb2) in func.blocks.iter() {
                                                for ins2 in bb2.instructions.iter() {
                                                    if let crate::mir::MirInstruction::Const { dst: cdst, value } = ins2 { if *cdst == src { if let crate::mir::ConstValue::String(sv) = value { lit_len = Some(sv.len() as i64); break; } } }
                                                }
                                                if lit_len.is_some() { break; }
                                            }
                                        }
                                    }
                                }
                                if lit_len.is_some() { break; }
                            }
                            if let Some(n) = lit_len {
                                b.emit_const_i64(n);
                                self.known_i64.insert(*did, n);
                                return Ok(());
                            }
                        }
                        if let Some(pidx) = self.param_index.get(array).copied() {
                            // Param 経路: string.len_h → 0 の場合 any.length_h へフォールバック
                            self.emit_len_with_fallback_param(b, pidx);
                            if let Some(d) = dst.as_ref() {
                                let slot = *self.local_index.entry(*d).or_insert_with(|| { let id = self.next_local; self.next_local += 1; id });
                                b.store_local_i64(slot);
                            }
                        } else {
                            crate::jit::events::emit_lower(
                                serde_json::json!({"id": crate::jit::r#extern::collections::SYM_ANY_LEN_H, "decision":"fallback", "reason":"receiver_not_param", "argc":1, "arg_types":["Handle"]}),
                                "hostcall","<jit>"
                            );
                            // Try local handle (AOT/JIT-AOT) before legacy index fallback
                            if let Some(slot) = self.local_index.get(array).copied() {
                                // ローカルハンドル: string.len_h → any.length_h フォールバック
                                self.emit_len_with_fallback_local_handle(b, slot);
                                if let Some(d) = dst.as_ref() {
                                    let slotd = *self.local_index.entry(*d).or_insert_with(|| { let id = self.next_local; self.next_local += 1; id });
                                    b.store_local_i64(slotd);
                                }
                            } else if self.box_type_map.get(array).map(|s| s == "StringBox").unwrap_or(false) {
                                // Attempt reconstruction for StringBox literal: scan NewBox(StringBox, Const String)
                                let mut lit: Option<String> = None;
                                for (_bid, bb) in func.blocks.iter() {
                                    for ins in bb.instructions.iter() {
                                        if let crate::mir::MirInstruction::NewBox { dst, box_type, args } = ins {
                                            if dst == array && box_type == "StringBox" && args.len() == 1 {
                                                if let Some(src) = args.get(0) {
                                                    if let Some(s) = self.known_str.get(src).cloned() { lit = Some(s); break; }
                                                    // Also scan Const directly
                                                    for (_bid2, bb2) in func.blocks.iter() {
                                                        for ins2 in bb2.instructions.iter() {
                                                            if let crate::mir::MirInstruction::Const { dst: cdst, value } = ins2 { if cdst == src { if let crate::mir::ConstValue::String(sv) = value { lit = Some(sv.clone()); break; } } }
                                                        }
                                                        if lit.is_some() { break; }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if lit.is_some() { break; }
                                }
                                if let Some(s) = lit {
                                    // リテラル復元: string.len_h → any.length_h フォールバック
                                    self.emit_len_with_fallback_literal(b, &s);
                                    if let Some(d) = dst.as_ref() {
                                        let slotd = *self.local_index.entry(*d).or_insert_with(|| { let id = self.next_local; self.next_local += 1; id });
                                        b.store_local_i64(slotd);
                                    }
                                } else {
                                    let arr_idx = -1;
                                    b.emit_const_i64(arr_idx);
                                    b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_LEN, 1, dst.is_some());
                                    if let Some(d) = dst.as_ref() {
                                        let slotd = *self.local_index.entry(*d).or_insert_with(|| { let id = self.next_local; self.next_local += 1; id });
                                        b.store_local_i64(slotd);
                                    }
                                }
                            } else {
                                let arr_idx = -1;
                                b.emit_const_i64(arr_idx);
                                b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_LEN, 1, dst.is_some());
                                if let Some(d) = dst.as_ref() {
                                    let slotd = *self.local_index.entry(*d).or_insert_with(|| { let id = self.next_local; self.next_local += 1; id });
                                    b.store_local_i64(slotd);
                                }
                            }
                        }
                    }
                    // math.* minimal boundary: use registry signature to decide allow/fallback (no actual hostcall yet)
                    "sin" | "cos" | "abs" | "min" | "max" => {
                        use crate::jit::hostcall_registry::{check_signature, ArgKind};
                        // Build symbol and observed arg kinds (f64 if known float, else i64)
                        let sym = format!("nyash.math.{}", method);
                        let mut observed: Vec<ArgKind> = Vec::new();
                        for v in args.iter() {
                            if self.known_f64.contains_key(v) { observed.push(ArgKind::F64); }
                            else { observed.push(ArgKind::I64); }
                        }
                        // Prepare arg_types for event payload
                        // Classify argument kinds using TyEnv when available; fallback to known maps/FloatBox tracking
                        let mut observed_kinds: Vec<crate::jit::hostcall_registry::ArgKind> = Vec::new();
                        for v in args.iter() {
                            let kind = if let Some(mt) = func.metadata.value_types.get(v) {
                                match mt {
                                    crate::mir::MirType::Float => crate::jit::hostcall_registry::ArgKind::F64,
                                    crate::mir::MirType::Integer => crate::jit::hostcall_registry::ArgKind::I64,
                                    crate::mir::MirType::Bool => crate::jit::hostcall_registry::ArgKind::I64, // b1はI64 0/1に正規化
                                    crate::mir::MirType::String | crate::mir::MirType::Box(_) => crate::jit::hostcall_registry::ArgKind::Handle,
                                    _ => {
                                        if self.known_f64.contains_key(v) || self.float_box_values.contains(v) { crate::jit::hostcall_registry::ArgKind::F64 }
                                        else { crate::jit::hostcall_registry::ArgKind::I64 }
                                    }
                                }
                            } else {
                                if self.known_f64.contains_key(v) || self.float_box_values.contains(v) { crate::jit::hostcall_registry::ArgKind::F64 }
                                else { crate::jit::hostcall_registry::ArgKind::I64 }
                            };
                            observed_kinds.push(kind);
                        }
                        let arg_types: Vec<&'static str> = observed_kinds.iter().map(|k| match k { crate::jit::hostcall_registry::ArgKind::I64 => "I64", crate::jit::hostcall_registry::ArgKind::F64 => "F64", crate::jit::hostcall_registry::ArgKind::Handle => "Handle" }).collect();
                        match check_signature(&sym, &observed_kinds) {
                            Ok(()) => {
                                // allow: record decision; execution remains on VM for now (thin bridge)
                                    crate::jit::events::emit_lower(
                                        serde_json::json!({
                                            "id": sym,
                                            "decision": "allow",
                                            "reason": "sig_ok",
                                            "argc": observed.len(),
                                            "arg_types": arg_types
                                        }),
                                        "hostcall","<jit>"
                                    );
                                // If native f64 is enabled, emit a typed hostcall to math extern
                                if crate::jit::config::current().native_f64 {
                                    let (symbol, arity) = match method.as_str() {
                                        "sin" => ("nyash.math.sin_f64", 1),
                                        "cos" => ("nyash.math.cos_f64", 1),
                                        "abs" => ("nyash.math.abs_f64", 1),
                                        "min" => ("nyash.math.min_f64", 2),
                                        "max" => ("nyash.math.max_f64", 2),
                                        _ => ("nyash.math.sin_f64", 1),
                                    };
                                    // Push f64 args from known_f64 or coerce known_i64
                                    for i in 0..arity {
                                        if let Some(v) = args.get(i) {
                                            // Try direct known values
                                            if let Some(fv) = self.known_f64.get(v).copied() { b.emit_const_f64(fv); continue; }
                                            if let Some(iv) = self.known_i64.get(v).copied() { b.emit_const_f64(iv as f64); continue; }
                                            // Try unwrap FloatBox: scan blocks to find NewBox FloatBox { args: [src] } and reuse src const
                                            let mut emitted = false;
                                            'scan: for (_bb_id, bb) in func.blocks.iter() {
                                                for ins in bb.instructions.iter() {
                                                    if let crate::mir::MirInstruction::NewBox { dst, box_type, args: nb_args } = ins {
                                                        if *dst == *v && box_type == "FloatBox" {
                                                            if let Some(srcv) = nb_args.get(0) {
                                                                if let Some(fv) = self.known_f64.get(srcv).copied() { b.emit_const_f64(fv); emitted = true; break 'scan; }
                                                                if let Some(iv) = self.known_i64.get(srcv).copied() { b.emit_const_f64(iv as f64); emitted = true; break 'scan; }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            if !emitted { b.emit_const_f64(0.0); }
                                        } else { b.emit_const_f64(0.0); }
                                    }
                                    let kinds: Vec<super::builder::ParamKind> = (0..arity).map(|_| super::builder::ParamKind::F64).collect();
                                    b.emit_host_call_typed(symbol, &kinds, dst.is_some(), true);
                                }
                            }
                            Err(reason) => {
                                crate::jit::events::emit_lower(
                                    serde_json::json!({
                                        "id": sym,
                                        "decision": "fallback",
                                        "reason": reason,
                                        "argc": observed.len(),
                                        "arg_types": arg_types
                                    }),
                                    "hostcall",
                                    "<jit>"
                                );
                            }
                        }
                        // no-op: VM側で実行される
                    }
                    "isEmpty" | "empty" => {
                        if let Some(pidx) = self.param_index.get(array).copied() {
                            crate::jit::events::emit(
                                "hostcall","<jit>",None,None,
                                serde_json::json!({"id": crate::jit::r#extern::collections::SYM_ANY_IS_EMPTY_H, "decision":"allow", "reason":"sig_ok", "argc":1, "arg_types":["Handle"]})
                            );
                            b.emit_param_i64(pidx);
                            // returns i64 0/1
                            b.emit_host_call(crate::jit::r#extern::collections::SYM_ANY_IS_EMPTY_H, 1, dst.is_some());
                        } else {
                                crate::jit::events::emit_lower(
                                    serde_json::json!({"id": crate::jit::r#extern::collections::SYM_ANY_IS_EMPTY_H, "decision":"fallback", "reason":"receiver_not_param", "argc":1, "arg_types":["Handle"]}),
                                    "hostcall","<jit>"
                                );
                        }
                    }
                    "push" => {
                        // argc=2: (array, value)
                        let argc = 2usize;
                        let val = args.get(0).and_then(|v| self.known_i64.get(v)).copied().unwrap_or(0);
                        if let Some(pidx) = self.param_index.get(array).copied() {
                            // Prepare args
                            b.emit_param_i64(pidx);
                            b.emit_const_i64(val);
                            // Decide policy
                            let decision = crate::jit::policy::invoke::decide_box_method("ArrayBox", "push", argc, false);
                            match decision {
                                crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, method_id, box_type, .. } => {
                                    b.emit_plugin_invoke(type_id, method_id, argc, false);
                                    crate::jit::observe::lower_plugin_invoke(&box_type, "push", type_id, method_id, argc);
                                }
                                crate::jit::policy::invoke::InvokeDecision::HostCall { symbol, .. } => {
                                    crate::jit::observe::lower_hostcall(&symbol, argc, &["Handle","I64"], "allow", "mapped_symbol");
                                    b.emit_host_call(&symbol, argc, false);
                                }
                                _ => {
                                    // Fallback to existing hostcall path
                                    let sym = crate::jit::r#extern::collections::SYM_ARRAY_PUSH_H;
                                    crate::jit::observe::lower_hostcall(sym, argc, &["Handle","I64"], "fallback", "policy_or_unknown");
                                    b.emit_host_call(sym, argc, false);
                                }
                            }
                        } else {
                            // No receiver param index
                            let arr_idx = -1;
                            b.emit_const_i64(arr_idx);
                            b.emit_const_i64(val);
                            let sym = crate::jit::r#extern::collections::SYM_ARRAY_PUSH;
                            crate::jit::observe::lower_hostcall(sym, argc, &["I64","I64"], "fallback", "receiver_not_param");
                            b.emit_host_call(sym, argc, false);
                        }
                    }
                    "size" => {
                        let argc = 1usize;
                        if let Some(pidx) = self.param_index.get(array).copied() {
                            b.emit_param_i64(pidx);
                            let decision = crate::jit::policy::invoke::decide_box_method("MapBox", "size", argc, dst.is_some());
                            match decision {
                                crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, method_id, box_type, .. } => {
                                    b.emit_plugin_invoke(type_id, method_id, argc, dst.is_some());
                                    crate::jit::observe::lower_plugin_invoke(&box_type, "size", type_id, method_id, argc);
                                }
                                crate::jit::policy::invoke::InvokeDecision::HostCall { symbol, .. } => {
                                    crate::jit::observe::lower_hostcall(&symbol, argc, &["Handle"], "allow", "mapped_symbol");
                                    b.emit_host_call(&symbol, argc, dst.is_some());
                                }
                                _ => {
                                    let sym = crate::jit::r#extern::collections::SYM_MAP_SIZE_H;
                                    crate::jit::observe::lower_hostcall(sym, argc, &["Handle"], "fallback", "policy_or_unknown");
                                    b.emit_host_call(sym, argc, dst.is_some());
                                }
                            }
                        } else {
                            let map_idx = -1;
                            b.emit_const_i64(map_idx);
                            let sym = crate::jit::r#extern::collections::SYM_MAP_SIZE;
                            crate::jit::observe::lower_hostcall(sym, argc, &["I64"], "fallback", "receiver_not_param");
                            b.emit_host_call(sym, argc, dst.is_some());
                        }
                    }
                    "get" => {
                        let argc = 2usize;
                        if let Some(pidx) = self.param_index.get(array).copied() {
                            b.emit_param_i64(pidx);
                            if let Some(k) = args.get(0).and_then(|v| self.known_i64.get(v)).copied() { b.emit_const_i64(k); } else if let Some(kvid) = args.get(0) { self.push_value_if_known_or_param(b, kvid); } else { b.emit_const_i64(0); }
                            let decision = crate::jit::policy::invoke::decide_box_method("MapBox", "get", argc, dst.is_some());
                            match decision {
                                crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, method_id, box_type, .. } => {
                                    b.emit_plugin_invoke(type_id, method_id, argc, dst.is_some());
                                    crate::jit::observe::lower_plugin_invoke(&box_type, "get", type_id, method_id, argc);
                                }
                                crate::jit::policy::invoke::InvokeDecision::HostCall { symbol, .. } => {
                                    crate::jit::observe::lower_hostcall(&symbol, argc, &["Handle","I64"], "allow", "mapped_symbol");
                                    b.emit_host_call(&symbol, argc, dst.is_some());
                                }
                                _ => {
                                    let sym = crate::jit::r#extern::collections::SYM_MAP_GET_H;
                                    crate::jit::observe::lower_hostcall(sym, argc, &["Handle","I64"], "fallback", "policy_or_unknown");
                                    b.emit_host_call(sym, argc, dst.is_some());
                                }
                            }
                        } else {
                            let sym = crate::jit::r#extern::collections::SYM_MAP_GET_H;
                            crate::jit::observe::lower_hostcall(sym, argc, &["I64","I64"], "fallback", "receiver_not_param");
                            b.emit_host_call(sym, argc, dst.is_some());
                        }
                    }
                    "set" => {
                        let argc = 3usize;
                        if let Some(pidx) = self.param_index.get(array).copied() {
                            let key = args.get(0).and_then(|v| self.known_i64.get(v)).copied().unwrap_or(0);
                            let val = args.get(1).and_then(|v| self.known_i64.get(v)).copied().unwrap_or(0);
                            b.emit_param_i64(pidx);
                            b.emit_const_i64(key);
                            b.emit_const_i64(val);
                            let decision = crate::jit::policy::invoke::decide_box_method("MapBox", "set", argc, false);
                            match decision {
                                crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, method_id, box_type, .. } => {
                                    b.emit_plugin_invoke(type_id, method_id, argc, false);
                                    crate::jit::observe::lower_plugin_invoke(&box_type, "set", type_id, method_id, argc);
                                }
                                crate::jit::policy::invoke::InvokeDecision::HostCall { symbol, .. } => {
                                    crate::jit::observe::lower_hostcall(&symbol, argc, &["Handle","I64","I64"], "allow", "mapped_symbol");
                                    b.emit_host_call(&symbol, argc, false);
                                }
                                _ => {
                                    let sym = crate::jit::r#extern::collections::SYM_MAP_SET_H;
                                    crate::jit::observe::lower_hostcall(sym, argc, &["Handle","I64","I64"], "fallback", "policy_or_unknown");
                                    b.emit_host_call(sym, argc, false);
                                }
                            }
                        } else {
                            let sym = crate::jit::r#extern::collections::SYM_MAP_SET;
                            crate::jit::observe::lower_hostcall(sym, argc, &["I64","I64","I64"], "fallback", "receiver_not_param");
                            b.emit_host_call(sym, argc, false);
                        }
                    }
                    "charCodeAt" => {
                        // String.charCodeAt(index)
                        if let Some(pidx) = self.param_index.get(array).copied() {
                            let idx = args.get(0).and_then(|v| self.known_i64.get(v)).copied().unwrap_or(0);
                            crate::jit::events::emit_lower(
                                serde_json::json!({"id": crate::jit::r#extern::collections::SYM_STRING_CHARCODE_AT_H, "decision":"allow", "reason":"sig_ok", "argc":2, "arg_types":["Handle","I64"]}),
                                "hostcall","<jit>"
                            );
                            b.emit_param_i64(pidx);
                            b.emit_const_i64(idx);
                            b.emit_host_call(crate::jit::r#extern::collections::SYM_STRING_CHARCODE_AT_H, 2, dst.is_some());
                        } else {
                            crate::jit::events::emit_lower(
                                serde_json::json!({"id": crate::jit::r#extern::collections::SYM_STRING_CHARCODE_AT_H, "decision":"fallback", "reason":"receiver_not_param", "argc":2, "arg_types":["Handle","I64"]}),
                                "hostcall","<jit>"
                            );
                        }
                    }
                    "has" => {
                        let argc = 2usize;
                        if let Some(pidx) = self.param_index.get(array).copied() {
                            let key = args.get(0).and_then(|v| self.known_i64.get(v)).copied().unwrap_or(0);
                            b.emit_param_i64(pidx);
                            b.emit_const_i64(key);
                            let decision = crate::jit::policy::invoke::decide_box_method("MapBox", "has", argc, dst.is_some());
                            match decision {
                                crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, method_id, box_type, .. } => {
                                    b.emit_plugin_invoke(type_id, method_id, argc, dst.is_some());
                                    crate::jit::observe::lower_plugin_invoke(&box_type, "has", type_id, method_id, argc);
                                }
                                crate::jit::policy::invoke::InvokeDecision::HostCall { symbol, .. } => {
                                    crate::jit::observe::lower_hostcall(&symbol, argc, &["Handle","I64"], "allow", "mapped_symbol");
                                    b.emit_host_call(&symbol, argc, dst.is_some());
                                }
                                _ => {
                                    let sym = crate::jit::r#extern::collections::SYM_MAP_HAS_H;
                                    crate::jit::observe::lower_hostcall(sym, argc, &["Handle","I64"], "fallback", "policy_or_unknown");
                                    b.emit_host_call(sym, argc, dst.is_some());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            */
            _ => {}
        }
        Ok(())
    }
}

pub use super::cfg_dot::dump_cfg_dot;
