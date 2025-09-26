use super::{BasicBlock, BasicBlockId};
use crate::mir::{BarrierOp, TypeOpKind, WeakRefOp};
// include path resolver removed (using handles modules)

// Optional builder debug logging
pub(super) fn builder_debug_enabled() -> bool {
    std::env::var("NYASH_BUILDER_DEBUG").is_ok()
}

pub(super) fn builder_debug_log(msg: &str) {
    if builder_debug_enabled() {
        eprintln!("[BUILDER] {}", msg);
    }
}

impl super::MirBuilder {
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
            // Entry materialization for pinned slots only when not suppressed.
            // This provides block-local defs in single-predecessor flows without touching user vars.
            if !self.suppress_pin_entry_copy_next {
                let names: Vec<String> = self.variable_map.keys().cloned().collect();
                for name in names {
                    if !name.starts_with("__pin$") { continue; }
                    if let Some(&src) = self.variable_map.get(&name) {
                        let dst = self.value_gen.next();
                        self.emit_instruction(super::MirInstruction::Copy { dst, src })?;
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
    ) -> Result<(), String> {
        // Check environment variable for unified call usage
        let use_unified = std::env::var("NYASH_MIR_UNIFIED_CALL")
            .unwrap_or_else(|_| "0".to_string()) != "0";

        if use_unified {
            // Use unified call emission for BoxCall
            // First, try to determine the box type
            let mut box_type: Option<String> = self.value_origin_newbox.get(&box_val).cloned();
            if box_type.is_none() {
                if let Some(t) = self.value_types.get(&box_val) {
                    match t {
                        super::MirType::String => box_type = Some("StringBox".to_string()),
                        super::MirType::Box(name) => box_type = Some(name.clone()),
                        _ => {}
                    }
                }
            }

            // Use emit_unified_call with Method target
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
        if crate::config::env::mir_core13_pure() {
            return Ok(box_val);
        }
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
        if crate::config::env::mir_core13_pure() {
            return Ok(weak_ref);
        }
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
        self.variable_map.insert(slot_name, dst);
        Ok(dst)
    }

    /// Ensure a value has a local definition in the current block by inserting a Copy.
    pub(crate) fn materialize_local(&mut self, v: super::ValueId) -> Result<super::ValueId, String> {
        let dst = self.value_gen.next();
        self.emit_instruction(super::MirInstruction::Copy { dst, src: v })?;
        Ok(dst)
    }

    /// Ensure a value is safe to use in the current block by slotifying (pinning) it.
    /// Currently correctness-first: always pin to get a block-local def and PHI participation.
    pub(crate) fn ensure_slotified_for_use(&mut self, v: super::ValueId, hint: &str) -> Result<super::ValueId, String> {
        self.pin_to_slot(v, hint)
    }
}
