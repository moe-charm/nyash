use super::{BasicBlock, BasicBlockId};
use crate::mir::{BarrierOp, TypeOpKind, WeakRefOp};
use std::fs;

// Resolve include path using nyash.toml include.roots if present
pub(super) fn resolve_include_path_builder(filename: &str) -> String {
    if filename.starts_with("./") || filename.starts_with("../") {
        return filename.to_string();
    }
    let parts: Vec<&str> = filename.splitn(2, '/').collect();
    if parts.len() == 2 {
        let root = parts[0];
        let rest = parts[1];
        let cfg_path = "nyash.toml";
        if let Ok(toml_str) = fs::read_to_string(cfg_path) {
            if let Ok(toml_val) = toml::from_str::<toml::Value>(&toml_str) {
                if let Some(include) = toml_val.get("include") {
                    if let Some(roots) = include.get("roots").and_then(|v| v.as_table()) {
                        if let Some(root_path) = roots.get(root).and_then(|v| v.as_str()) {
                            let mut base = root_path.to_string();
                            if !base.ends_with('/') && !base.ends_with('\\') {
                                base.push('/');
                            }
                            return format!("{}{}", base, rest);
                        }
                    }
                }
            }
        }
    }
    format!("./{}", filename)
}

// Optional builder debug logging
pub(super) fn builder_debug_enabled() -> bool {
    std::env::var("NYASH_BUILDER_DEBUG").is_ok()
}

pub(super) fn builder_debug_log(msg: &str) {
    if builder_debug_enabled() {
        eprintln!("[BUILDER] {}", msg);
    }
}

// Lightweight helpers moved from builder.rs to reduce file size
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
            Ok(())
        } else {
            Err("No current function".to_string())
        }
    }
}

// Call/Type/WeakRef emission helpers (moved from builder.rs)
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
                    let inferred: Option<super::MirType> = match (bt.as_str(), method.as_str()) {
                        ("StringBox", "length") | ("StringBox", "len") => {
                            Some(super::MirType::Integer)
                        }
                        ("StringBox", "is_empty") => Some(super::MirType::Bool),
                        ("StringBox", "charCodeAt") => Some(super::MirType::Integer),
                        ("StringBox", "substring")
                        | ("StringBox", "concat")
                        | ("StringBox", "replace")
                        | ("StringBox", "trim")
                        | ("StringBox", "toUpper")
                        | ("StringBox", "toLower") => Some(super::MirType::String),
                        ("ArrayBox", "length") => Some(super::MirType::Integer),
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
}
