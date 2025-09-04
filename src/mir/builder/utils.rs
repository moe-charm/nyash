use std::fs;
use super::{BasicBlock, BasicBlockId};

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

// PHI-based return type inference helper
pub(super) fn infer_type_from_phi(
    function: &super::MirFunction,
    ret_val: super::ValueId,
    types: &std::collections::HashMap<super::ValueId, super::MirType>,
) -> Option<super::MirType> {
    for (_bid, bb) in function.blocks.iter() {
        for inst in bb.instructions.iter() {
            if let super::MirInstruction::Phi { dst, inputs } = inst {
                if *dst == ret_val {
                    let mut it = inputs.iter().filter_map(|(_, v)| types.get(v));
                    if let Some(first) = it.next() {
                        if it.all(|mt| mt == first) {
                            return Some(first.clone());
                        }
                    }
                }
            }
        }
    }
    None
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
