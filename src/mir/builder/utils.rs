use super::{BasicBlock, BasicBlockId};
use crate::mir::{BarrierOp, TypeOpKind, WeakRefOp};
use std::sync::atomic::{AtomicUsize, Ordering};
// include path resolver removed (using handles modules)

// Optional builder debug logging
pub(super) fn builder_debug_enabled() -> bool {
    std::env::var("NYASH_BUILDER_DEBUG").is_ok()
}

static BUILDER_DEBUG_COUNT: AtomicUsize = AtomicUsize::new(0);

pub(super) fn builder_debug_log(msg: &str) {
    if builder_debug_enabled() {
        // Optional cap: limit the number of builder debug lines to avoid flooding the terminal.
        // Set via env: NYASH_BUILDER_DEBUG_LIMIT=<N> (default: unlimited)
        if let Ok(cap_s) = std::env::var("NYASH_BUILDER_DEBUG_LIMIT") {
            if let Ok(cap) = cap_s.parse::<usize>() {
                let n = BUILDER_DEBUG_COUNT.fetch_add(1, Ordering::Relaxed);
                if n >= cap { return; }
            }
        }
        eprintln!("[BUILDER] {}", msg);
    }
}

impl super::MirBuilder {
    #[inline]
    fn coerce_string_like_receiver_if_ambiguous(
        &mut self,
        recv: super::ValueId,
        method: &str,
        inferred_cls: &str,
    ) -> (super::ValueId, String) {
        let is_string_like = matches!(
            method,
            "length" | "len" | "substring" | "indexOf" | "lastIndexOf"
        );
        if !is_string_like {
            return (recv, inferred_cls.to_string());
        }
        let is_string_ty = self
            .value_types
            .get(&recv)
            .map(|t| matches!(t, super::MirType::String))
            .unwrap_or(false);
        let is_string_origin = self
            .origin_get(recv)
            .map(|s| s == "StringBox")
            .unwrap_or(false);
        // Only coerce for ambiguous/non-string non-core receivers (Instance/Parser/Debug/File/Unknown)
        let is_ambiguous = matches!(
            inferred_cls,
            "UnknownBox" | "InstanceBox" | "ParserBox" | "DebugBox" | "FileBox"
        );
        if is_ambiguous && !(is_string_ty || is_string_origin) {
            // Emit: tmp = "" + recv  (stringify via concat; VM fast-path guarantees string result)
            let empty = crate::mir::builder::emission::constant::emit_string(self, "");
            let tmp = self.value_gen.next();
            let _ = self.emit_instruction(super::MirInstruction::BinOp {
                dst: tmp,
                op: crate::mir::BinaryOp::Add,
                lhs: empty,
                rhs: recv,
            });
            self.value_types.insert(tmp, super::MirType::String);
            return (tmp, "StringBox".to_string());
        }
        (recv, inferred_cls.to_string())
    }
    // ---- LocalSSA convenience (readability helpers) ----
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn local_recv(&mut self, v: super::ValueId) -> super::ValueId { super::ssa::local::recv(self, v) }
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn local_arg(&mut self, v: super::ValueId) -> super::ValueId { super::ssa::local::arg(self, v) }
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn local_field_base(&mut self, v: super::ValueId) -> super::ValueId { super::ssa::local::field_base(self, v) }
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn local_cond(&mut self, v: super::ValueId) -> super::ValueId { super::ssa::local::cond(self, v) }
    /// Ensure a basic block exists in the current function
    pub(crate) fn ensure_block_exists(&mut self, block_id: BasicBlockId) -> Result<(), String> {
        if let Some(ref mut function) = self.current_function {
            if !function.blocks.contains_key(&block_id) {
                let block = BasicBlock::new(block_id);
                function.add_block(block);
            }
            Ok(())
        } else {
            Err("No current function".to_string())
        }
    }

    /// Start a new basic block and set as current
    pub(crate) fn start_new_block(&mut self, block_id: BasicBlockId) -> Result<(), String> {
        if let Some(ref mut function) = self.current_function {
            function.add_block(BasicBlock::new(block_id));
            self.current_block = Some(block_id);
            // Local SSA cache is per-block; clear on block switch
            self.local_ssa_map.clear();
            // BlockSchedule materialize cache is per-block as well
            self.schedule_mat_map.clear();
            // Entry materialization for pinned slots only when not suppressed.
            // This provides block-local defs in single-predecessor flows without touching user vars.
            if !self.suppress_pin_entry_copy_next {
                // First pass: copy all pin slots and remember old->new mapping
                let names: Vec<String> = self.variable_map.keys().cloned().collect();
                for name in names.iter() {
                    if !name.starts_with("__pin$") { continue; }
                    if let Some(&src) = self.variable_map.get(name) {
                        let dst = self.value_gen.next();
                        self.emit_instruction(super::MirInstruction::Copy { dst, src })?;
                        crate::mir::builder::metadata::propagate::propagate(self, src, dst);
                        self.variable_map.insert(name.clone(), dst);
                    }
                }
            }
            // Reset suppression flag after use (one-shot)
            self.suppress_pin_entry_copy_next = false;
            Ok(())
        } else {
            Err("No current function".to_string())
        }
    }
}

impl super::MirBuilder {
    /// Emit a Box method call or plugin call (unified BoxCall)
    pub(super) fn emit_box_or_plugin_call(
        &mut self,
        dst: Option<super::ValueId>,
        box_val: super::ValueId,
        method: String,
        method_id: Option<u16>,
        args: Vec<super::ValueId>,
        effects: super::EffectMask,
        // When true, do not bounce back into unified-call route from here.
        // This is used by unified-call's router-guard fallback to avoid infinite recursion
        // (unified -> boxcall -> unified -> ...).
        force_legacy: bool,
    ) -> Result<(), String> {
        if method == "birth" {
            let recv_local = self.local_recv(box_val);
            let mut argv: Vec<super::ValueId> = args.into_iter().map(|a| self.local_arg(a)).collect();
            let (cls, _c) = crate::mir::builder::infer::receiver::infer_receiver(
                None,
                &method,
                recv_local,
                |vid| self.origin_get(vid).map(|s| s.to_string()),
                &self.value_types,
            );
            let arity = argv.len();
            let fname = crate::mir::builder::calls::function_lowering::generate_method_function_name(&cls, &method, arity);
            let name_val = crate::mir::builder::name_const::make_name_const_result(self, &fname)?;
            let mut call_args: Vec<super::ValueId> = Vec::with_capacity(1 + arity);
            call_args.push(recv_local);
            call_args.extend(argv.drain(..));
            let out = dst.unwrap_or_else(|| self.value_gen.next());
            self.emit_instruction(super::MirInstruction::Call {
                dst: Some(out),
                func: name_val,
                callee: Some(crate::mir::definitions::Callee::ModuleFunction(fname.clone())),
                args: call_args,
                effects,
            })?;
            self.annotate_call_result_from_func_name(out, &fname);
            return Ok(());
        }

        // Ensure receiver has a definition in the current block to avoid undefined use across
        // block boundaries (LoopForm/header, if-joins, etc.).
        // LocalSSA: ensure receiver has an in-block definition (kind=0 = recv)
        let box_val = self.local_recv(box_val);
        // LocalSSA: ensure args are materialized in current block
        let args: Vec<super::ValueId> = args.into_iter().map(|a| self.local_arg(a)).collect();
        // Check environment variable for unified call usage, with safe overrides for core/user boxes
        let use_unified_env = super::calls::call_unified::is_unified_call_enabled();
        // First, infer the receiver class consistently with unified path
        let (mut inferred_cls, _certainty) = crate::mir::builder::infer::receiver::infer_receiver(
            None,
            &method,
            box_val,
            |vid| self.origin_get(vid).map(|s| s.to_string()),
            &self.value_types,
        );
        let mut box_val = box_val;
        // Coerce ambiguous receiver for string-like APIs (Instance/Parser/Debug/File/Unknown)
        let (coerced, coerced_cls) = self.coerce_string_like_receiver_if_ambiguous(box_val, &method, &inferred_cls);
        box_val = coerced;
        inferred_cls = coerced_cls;
        let box_type: Option<String> = Some(inferred_cls.clone());
        // Route decision is centralized in RouterPolicyBox（仕様不変）。
        let bx_name = box_type.clone().unwrap_or_else(|| "UnknownBox".to_string());
        let route = crate::mir::builder::router::policy::choose_route(
            &bx_name,
            &method,
            crate::mir::definitions::call_unified::TypeCertainty::Union,
            args.len(),
        );
        if super::utils::builder_debug_enabled() || std::env::var("NYASH_LOCAL_SSA_TRACE").ok().as_deref() == Some("1") {
            if matches!(method.as_str(), "parse" | "substring" | "has_errors" | "length") {
                eprintln!(
                    "[boxcall-decision] method={} bb={:?} recv=%{} class_hint={:?} prefer_legacy={}",
                    method,
                    self.current_block,
                    box_val.0,
                    box_type,
                    matches!(route, crate::mir::builder::router::policy::Route::BoxCall)
                );
            }
        }
        if !force_legacy && use_unified_env && matches!(route, crate::mir::builder::router::policy::Route::Unified) {
            let target = super::builder_calls::CallTarget::Method {
                box_type,
                method: method.clone(),
                receiver: box_val,
            };
            return self.emit_unified_call(dst, target, args);
        }

        // Legacy implementation
        self.emit_instruction(super::MirInstruction::BoxCall {
            dst,
            box_val,
            method: method.clone(),
            method_id,
            args,
            effects,
        })?;
        if let Some(d) = dst {
            let mut recv_box: Option<String> = self.origin_get(box_val).map(|s| s.to_string());
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
                    // Phase 15.5: Unified plugin-based type resolution
                    // Former core boxes (StringBox, ArrayBox, MapBox) now use plugin_method_sigs only
                    // No special hardcoded inference - all boxes treated uniformly
                }
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub(super) fn emit_type_check(
        &mut self,
        value: super::ValueId,
        expected_type: String,
    ) -> Result<super::ValueId, String> {
        let dst = self.value_gen.next();
        self.emit_instruction(super::MirInstruction::TypeOp {
            dst,
            op: TypeOpKind::Check,
            value,
            ty: super::MirType::Box(expected_type),
        })?;
        Ok(dst)
    }

    #[allow(dead_code)]
    pub(super) fn emit_cast(
        &mut self,
        value: super::ValueId,
        target_type: super::MirType,
    ) -> Result<super::ValueId, String> {
        let dst = self.value_gen.next();
        self.emit_instruction(super::MirInstruction::TypeOp {
            dst,
            op: TypeOpKind::Cast,
            value,
            ty: target_type.clone(),
        })?;
        Ok(dst)
    }

    #[allow(dead_code)]
    pub(super) fn emit_weak_new(
        &mut self,
        box_val: super::ValueId,
    ) -> Result<super::ValueId, String> {
        // Core‑13 pure mode removed; keep WeakRef emission available.
        let dst = self.value_gen.next();
        self.emit_instruction(super::MirInstruction::WeakRef {
            dst,
            op: WeakRefOp::New,
            value: box_val,
        })?;
        Ok(dst)
    }

    #[allow(dead_code)]
    pub(super) fn emit_weak_load(
        &mut self,
        weak_ref: super::ValueId,
    ) -> Result<super::ValueId, String> {
        // Core‑13 pure mode removed; keep WeakRef emission available.
        let dst = self.value_gen.next();
        self.emit_instruction(super::MirInstruction::WeakRef {
            dst,
            op: WeakRefOp::Load,
            value: weak_ref,
        })?;
        Ok(dst)
    }

    #[allow(dead_code)]
    pub(super) fn emit_barrier_read(&mut self, ptr: super::ValueId) -> Result<(), String> {
        self.emit_instruction(super::MirInstruction::Barrier {
            op: BarrierOp::Read,
            ptr,
        })
    }

    #[allow(dead_code)]
    pub(super) fn emit_barrier_write(&mut self, ptr: super::ValueId) -> Result<(), String> {
        self.emit_instruction(super::MirInstruction::Barrier {
            op: BarrierOp::Write,
            ptr,
        })
    }

    /// Pin a block-crossing ephemeral value into a pseudo local slot and register it in variable_map
    /// so it participates in PHI merges across branches/blocks. Safe default for correctness-first.
    pub(crate) fn pin_to_slot(&mut self, v: super::ValueId, hint: &str) -> Result<super::ValueId, String> {
        self.temp_slot_counter = self.temp_slot_counter.wrapping_add(1);
        let slot_name = format!("__pin${}${}", self.temp_slot_counter, hint);
        let dst = self.value_gen.next();
        self.emit_instruction(super::MirInstruction::Copy { dst, src: v })?;
        if super::utils::builder_debug_enabled() || std::env::var("NYASH_PIN_TRACE").ok().as_deref() == Some("1") {
            super::utils::builder_debug_log(&format!("pin slot={} src={} dst={}", slot_name, v.0, dst.0));
        }
        // Propagate lightweight metadata so downstream resolution/type inference remains stable
        crate::mir::builder::metadata::propagate::propagate(self, v, dst);
        self.variable_map.insert(slot_name, dst);
        Ok(dst)
    }

    /// Ensure a value has a local definition in the current block by inserting a Copy.
    pub(crate) fn materialize_local(&mut self, v: super::ValueId) -> Result<super::ValueId, String> {
        let dst = self.value_gen.next();
        self.emit_instruction(super::MirInstruction::Copy { dst, src: v })?;
        // Propagate metadata (type/origin) from source to the new local copy
        crate::mir::builder::metadata::propagate::propagate(self, v, dst);
        Ok(dst)
    }

    /// Insert a Copy immediately after PHI nodes in the current block (position-stable).
    /// 📦 Kept for future use: SSA transformation optimizations requiring precise instruction ordering
    pub(crate) fn insert_copy_after_phis(&mut self, dst: super::ValueId, src: super::ValueId) -> Result<(), String> {
        if let (Some(ref mut function), Some(bb)) = (&mut self.current_function, self.current_block) {
            if let Some(block) = function.get_block_mut(bb) {
                // Propagate effects on the block
                block.insert_instruction_after_phis(super::MirInstruction::Copy { dst, src });
                // Lightweight metadata propagation (unified)
                crate::mir::builder::metadata::propagate::propagate(self, src, dst);
                return Ok(());
            }
        }
        Err("No current function/block to insert copy".to_string())
    }

    /// Ensure a value is safe to use in the current block by slotifying (pinning) it.
    /// Currently correctness-first: always pin to get a block-local def and PHI participation.
    /// 📦 Kept for future use: memory management and slot allocation strategies
    pub(crate) fn ensure_slotified_for_use(&mut self, v: super::ValueId, hint: &str) -> Result<super::ValueId, String> {
        self.pin_to_slot(v, hint)
    }

    /// Local SSA: ensure a value has a definition in the current block and cache it per-block.
    /// kind: 0 = recv (reserved for args in future)
    pub(crate) fn local_ssa_ensure(&mut self, v: super::ValueId, kind: u8) -> super::ValueId {
        use super::ssa::local::{ensure, LocalKind};
        let lk = match kind {
            0 => LocalKind::Recv,
            1 => LocalKind::Arg,
            2 => LocalKind::CompareOperand,
            4 => LocalKind::Cond,
            x => LocalKind::Other(x),
        };
        ensure(self, v, lk)
    }
}
