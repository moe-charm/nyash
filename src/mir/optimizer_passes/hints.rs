use std::collections::HashMap;
use crate::mir::{MirFunction, MirInstruction};
use super::hints_config::{OptimizationConfigBox, OptimizationType};
use super::detectors::{DetectorRegistry, CallPattern};

#[derive(Debug, Clone, Default)]
pub struct CallHints {
    pub self_recursive: bool,
    pub tail_call: bool,
    pub pure: bool,
    pub escape_none: bool,
}

pub type HintsMap = HashMap<(String, usize), CallHints>;

pub struct HintsBuilderBox {
    pub config: OptimizationConfigBox,
    pub detectors: DetectorRegistry,
}

impl Default for HintsBuilderBox {
    fn default() -> Self {
        Self { config: OptimizationConfigBox::from_env(), detectors: DetectorRegistry::new() }
    }
}

impl HintsBuilderBox {
    pub fn build_hints(&self, function: &MirFunction) -> HintsMap {
        let mut map: HintsMap = HashMap::new();
        let name = function.signature.name.clone();
        let mut idx: usize = 0;
        for bid in function.block_ids() {
            if let Some(bb) = function.get_block(bid) {
                for inst in bb.non_phi_instructions() {
                    if let MirInstruction::Call { .. } = inst {
                        let CallPattern { self_recursive, tail_call } = self.detectors.detect(function, idx, inst);
                        let mut hints = CallHints::default();
                        if self.config.is_enabled(OptimizationType::SelfRecursion) && self_recursive { hints.self_recursive = true; }
                        if self.config.is_enabled(OptimizationType::TailCall) && tail_call { hints.tail_call = true; }
                        map.insert((name.clone(), idx), hints);
                    }
                    idx += 1;
                }
            }
        }
        map
    }

    pub fn to_json(&self, hints: &HintsMap) -> String {
        let mut v = Vec::new();
        for ((fn_name, idx), h) in hints.iter() {
            v.push(serde_json::json!({
                "function": fn_name,
                "call_index": idx,
                "self_recursive": h.self_recursive,
                "tail_call": h.tail_call,
                "pure": h.pure,
                "escape_none": h.escape_none,
            }));
        }
        serde_json::to_string(&v).unwrap_or_else(|_| "[]".to_string())
    }
}

