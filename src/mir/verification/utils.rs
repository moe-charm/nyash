use crate::mir::{function::MirFunction, BasicBlockId, ValueId};
use std::collections::{HashMap, HashSet};

pub fn compute_predecessors(function: &MirFunction) -> HashMap<BasicBlockId, Vec<BasicBlockId>> {
    let mut preds: HashMap<BasicBlockId, Vec<BasicBlockId>> = HashMap::new();
    for (bid, block) in &function.blocks {
        for succ in &block.successors {
            preds.entry(*succ).or_default().push(*bid);
        }
    }
    preds
}

pub fn compute_def_blocks(function: &MirFunction) -> HashMap<ValueId, BasicBlockId> {
    let mut def_block: HashMap<ValueId, BasicBlockId> = HashMap::new();
    for (bid, block) in &function.blocks {
        for inst in block.all_instructions() {
            if let Some(dst) = inst.dst_value() { def_block.insert(dst, *bid); }
        }
    }
    def_block
}

pub fn compute_dominators(function: &MirFunction) -> HashMap<BasicBlockId, HashSet<BasicBlockId>> {
    let all_blocks: HashSet<BasicBlockId> = function.blocks.keys().copied().collect();
    let preds = compute_predecessors(function);
    let mut dom: HashMap<BasicBlockId, HashSet<BasicBlockId>> = HashMap::new();

    for &b in function.blocks.keys() {
        if b == function.entry_block {
            let mut set = HashSet::new();
            set.insert(b);
            dom.insert(b, set);
        } else {
            dom.insert(b, all_blocks.clone());
        }
    }

    let mut changed = true;
    while changed {
        changed = false;
        for &b in function.blocks.keys() {
            if b == function.entry_block { continue; }
            let mut new_set = all_blocks.clone();
            if let Some(p_list) = preds.get(&b) {
                for p in p_list {
                    if let Some(p_dom) = dom.get(p) {
                        new_set = new_set.intersection(p_dom).copied().collect();
                    }
                }
            }
            new_set.insert(b);
            let cur = dom.get(&b).unwrap();
            if &new_set != cur { dom.insert(b, new_set); changed = true; }
        }
    }
    dom
}

pub fn compute_reachable_blocks(function: &MirFunction) -> HashSet<BasicBlockId> {
    let mut reachable = HashSet::new();
    let mut worklist = vec![function.entry_block];
    while let Some(current) = worklist.pop() {
        if reachable.insert(current) {
            if let Some(block) = function.blocks.get(&current) {
                for successor in &block.successors {
                    if !reachable.contains(successor) {
                        worklist.push(*successor);
                    }
                }
                for instruction in &block.instructions {
                    if let crate::mir::MirInstruction::Catch { handler_bb, .. } = instruction {
                        if !reachable.contains(handler_bb) { worklist.push(*handler_bb); }
                    }
                }
                if let Some(ref terminator) = block.terminator {
                    if let crate::mir::MirInstruction::Catch { handler_bb, .. } = terminator {
                        if !reachable.contains(handler_bb) { worklist.push(*handler_bb); }
                    }
                }
            }
        }
    }
    reachable
}

