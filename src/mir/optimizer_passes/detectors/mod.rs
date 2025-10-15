use crate::mir::{MirFunction, MirInstruction};

#[derive(Debug, Clone, Default)]
pub struct CallPattern {
    pub self_recursive: bool,
    pub tail_call: bool,
}

pub trait PatternDetector {
    fn detect(&self, func: &MirFunction, instr_idx: usize, instr: &MirInstruction) -> Option<CallPattern>;
}

pub struct SelfRecursionDetector;
impl PatternDetector for SelfRecursionDetector {
    fn detect(&self, func: &MirFunction, _idx: usize, instr: &MirInstruction) -> Option<CallPattern> {
        if let MirInstruction::Call { callee: Some(c), .. } = instr {
            if let crate::mir::definitions::Callee::ModuleFunction(name) = c {
                // Normalize: compare base or exact
                let cur = func.signature.name.as_str();
                if name == cur || name.split('/').next() == Some(cur.split('/').next().unwrap_or(cur)) {
                    return Some(CallPattern { self_recursive: true, tail_call: false });
                }
            }
        }
        None
    }
}

pub struct TailCallDetector;
impl PatternDetector for TailCallDetector {
    fn detect(&self, func: &MirFunction, _idx: usize, instr: &MirInstruction) -> Option<CallPattern> {
        // Heuristic: call that is the last non-phi in a block with Return terminator
        for bid in func.block_ids() {
            if let Some(bb) = func.get_block(bid) {
                if !bb.ends_with_return() { continue; }
                if let Some(last) = bb.non_phi_instructions().last() {
                    if std::ptr::eq(last, instr) {
                        if matches!(last, MirInstruction::Call { .. }) {
                            return Some(CallPattern { self_recursive: false, tail_call: true });
                        }
                    }
                }
            }
        }
        None
    }
}

pub struct DetectorRegistry {
    detectors: Vec<Box<dyn PatternDetector + Send + Sync>>,
}

impl Default for DetectorRegistry {
    fn default() -> Self {
        Self { detectors: vec![Box::new(SelfRecursionDetector), Box::new(TailCallDetector)] }
    }
}

impl DetectorRegistry {
    pub fn new() -> Self { Default::default() }
    pub fn detect(&self, func: &MirFunction, instr_idx: usize, instr: &MirInstruction) -> CallPattern {
        let mut out = CallPattern::default();
        for d in &self.detectors {
            if let Some(p) = d.detect(func, instr_idx, instr) {
                if p.self_recursive { out.self_recursive = true; }
                if p.tail_call { out.tail_call = true; }
            }
        }
        out
    }
}

