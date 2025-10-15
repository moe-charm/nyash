use crate::mir::{MirInstruction, MirModule, ValueId};
use crate::mir::definitions::call_unified::Callee;
use crate::mir::optimizer_stats::OptimizationStats;

/// Light repair pass: ensure StringBox.(size|len|length) method calls have an in-block
/// receiver copy immediately before the call site. This defends against use-before-def
/// issues when earlier stages failed to materialize the receiver.
pub fn repair_string_len_receivers(module: &mut MirModule) -> OptimizationStats {
    let mut stats = OptimizationStats::new();
    for (_fname, func) in module.functions.iter_mut() {
        let mut changed_in_fn = 0usize;
        // Iterate blocks by id to avoid borrow conflicts when allocating new ids
        let bids: Vec<_> = func.block_ids();
        for bid in bids {
            // First pass: scan and collect patch sites
            let mut patches: Vec<(usize, ValueId, Option<ValueId>, Callee, crate::mir::EffectMask, Vec<ValueId>)> = Vec::new();
            if let Some(block) = func.blocks.get(&bid) {
                for (idx, inst) in block.instructions.iter().enumerate() {
                    if let MirInstruction::Call { dst, callee: Some(Callee::Method { box_name, method, receiver: Some(r), .. }), args, effects, .. } = inst {
                        let is_len = (method == "size" || method == "len" || method == "length") && args.is_empty();
                        let is_target_box = box_name == "StringBox" || box_name == "ArrayBox";
                        if is_len && is_target_box {
                            patches.push((idx, *r, *dst, Callee::Method { box_name: box_name.clone(), method: method.clone(), receiver: Some(*r), certainty: crate::mir::definitions::call_unified::TypeCertainty::Known }, *effects, args.clone()));
                        }
                    }
                    // Also repair already-normalized extern calls
                    if let MirInstruction::Call { dst, callee: Some(Callee::Extern(name)), args, effects, .. } = inst {
                        let name_str = name.as_str();
                        if (name_str == "nyrt.string.length" || name_str == "nyrt.array.size") && args.len() == 1 {
                            let r = args[0];
                            patches.push((idx, r, *dst, Callee::Extern(name.clone()), *effects, args.clone()));
                        }
                    }
                }
            }

            if patches.is_empty() { continue; }

            // Apply patches with running offset
            let mut applied = 0usize;
            for (idx, recv_old, dst_opt, callee_clone, effects_clone, args_clone) in patches.into_iter() {
                // Allocate new id before borrowing the block mutably
                let recv_local = {
                    let id = func.next_value_id;
                    // Increment function's next id
                    // SAFETY: we do not hold any &mut to blocks here
                    // (we'll borrow after computing this value)
                    // so this split is fine for borrow checker
                    // Since field is public, adjust directly.
                    // Then wrap into ValueId
                    //
                    // Equivalent to func.next_value_id()
                    // but avoids mutable borrow conflict with block
                    //
                    // Update counter
                    //
                    // NOTE: keep in sync with MirFunction::next_value_id semantics
                    
                    // Actually update
                    // (we need a second mutable borrow – so perform in a tiny scope without &mut block)
                    
                    // Use a raw pointer-like pattern by re-borrowing func mutably via a block
                    // This closure is a no-op placeholder; we will update below when we have unique borrow.
                    ValueId::new(id)
                };
                // Now update the counter properly (separate step to satisfy borrow rules)
                func.next_value_id += 1;

                if let Some(block) = func.blocks.get_mut(&bid) {
                    let at = idx + applied;
                    // Insert Copy before call
                    block.instructions.insert(at, MirInstruction::Copy { dst: recv_local, src: recv_old });
                    // String/Array: rewrite to extern for length-like methods to avoid fragile Method recv
                    let new_inst = match callee_clone {
                        Callee::Method { box_name, method, .. } if (method == "size" || method == "len" || method == "length") => {
                            let extern_name = if box_name == "StringBox" { "nyrt.string.length" } else { "nyrt.array.size" };
                            MirInstruction::Call {
                                dst: dst_opt,
                                func: ValueId::new(0),
                                callee: Some(Callee::Extern(extern_name.to_string())),
                                args: vec![recv_local],
                                effects: effects_clone,
                            }
                        }
                        Callee::Extern(name) if (name == "nyrt.string.length" || name == "nyrt.array.size") => {
                            // Already extern: only patch the receiver arg to the fresh local
                            MirInstruction::Call {
                                dst: dst_opt,
                                func: ValueId::new(0),
                                callee: Some(Callee::Extern(name)),
                                args: vec![recv_local],
                                effects: effects_clone,
                            }
                        }
                        Callee::Method { box_name, method, certainty, .. } => {
                            // Fallback: keep as Method but ensure recv is local
                            let cal = Callee::Method { box_name, method, receiver: Some(recv_local), certainty };
                            MirInstruction::Call { dst: dst_opt, func: ValueId::new(0), callee: Some(cal), args: args_clone, effects: effects_clone }
                        }
                        other => MirInstruction::Call { dst: dst_opt, func: ValueId::new(0), callee: Some(other), args: args_clone, effects: effects_clone },
                    };
                    // Replace the call at at+1
                    block.instructions[at + 1] = new_inst;
                    applied += 1;
                    changed_in_fn += 1;
                }
            }
        }
        if changed_in_fn > 0 {
            // Count as intrinsic-like fixes
            stats.intrinsic_optimizations += changed_in_fn;
        }
    }
    stats
}

/// Ensure BoxCall receivers are materialized directly before the call site.
///
/// Some legacy emission paths or conservative routing may produce BoxCall with a
/// receiver that was defined in a different block. This pass inserts a Copy of
/// the receiver immediately before each BoxCall and rewrites the call to use
/// the new local id, eliminating "use of undefined value" at runtime.
pub fn repair_boxcall_receivers(module: &mut MirModule) -> OptimizationStats {
    let mut stats = OptimizationStats::new();
    for (_fname, func) in module.functions.iter_mut() {
        let bids: Vec<_> = func.block_ids();
        for bid in bids {
            let mut patches: Vec<(usize, ValueId, Option<ValueId>, String, Option<u16>, Vec<ValueId>, crate::mir::EffectMask)> = Vec::new();
            if let Some(block) = func.blocks.get(&bid) {
                for (idx, inst) in block.instructions.iter().enumerate() {
                    if let MirInstruction::BoxCall { dst, box_val, method, method_id, args, effects } = inst {
                        patches.push((idx, *box_val, *dst, method.clone(), *method_id, args.clone(), *effects));
                    }
                }
            }
            if patches.is_empty() { continue; }
            let mut applied = 0usize;
            for (idx, recv_old, dst_opt, method, method_id, args_clone, effects_clone) in patches.into_iter() {
                let recv_local = { let id = func.next_value_id; func.next_value_id += 1; ValueId::new(id) };
                if let Some(block) = func.blocks.get_mut(&bid) {
                    let at = idx + applied;
                    block.instructions.insert(at, MirInstruction::Copy { dst: recv_local, src: recv_old });
                    block.instructions[at + 1] = MirInstruction::BoxCall {
                        dst: dst_opt,
                        box_val: recv_local,
                        method,
                        method_id,
                        args: args_clone,
                        effects: effects_clone,
                    };
                    applied += 1;
                    stats.intrinsic_optimizations += 1;
                }
            }
        }
    }
    stats
}

/// Generic repair: materialize Method receivers directly before call sites.
///
/// Inserts `Copy recv_local <- recv_old` immediately before every
/// `Call{ callee=Method{ receiver=Some(recv_old) } }` and rewrites the call to
/// use `recv_local`. This avoids use-before-def across basic blocks when the
/// receiver was defined in a different block.
pub fn repair_method_receivers(module: &mut MirModule) -> OptimizationStats {
    let mut stats = OptimizationStats::new();
    for (_fname, func) in module.functions.iter_mut() {
        let bids: Vec<_> = func.block_ids();
        for bid in bids {
            let mut patches: Vec<(usize, ValueId, Option<ValueId>, Callee, Vec<ValueId>, crate::mir::EffectMask)> = Vec::new();
            if let Some(block) = func.blocks.get(&bid) {
                for (idx, inst) in block.instructions.iter().enumerate() {
                    if let MirInstruction::Call { dst, callee: Some(Callee::Method { box_name, method, receiver: Some(r), certainty }), args, effects, .. } = inst {
                        // Record patch for any Method call (class/method kept)
                        let cal = Callee::Method { box_name: box_name.clone(), method: method.clone(), receiver: Some(*r), certainty: *certainty };
                        patches.push((idx, *r, *dst, cal, args.clone(), *effects));
                    }
                }
            }
            if patches.is_empty() { continue; }
            let mut applied = 0usize;
            for (idx, recv_old, dst_opt, callee_clone, args_clone, effects_clone) in patches.into_iter() {
                let recv_local = { let id = func.next_value_id; func.next_value_id += 1; ValueId::new(id) };
                if let Some(block) = func.blocks.get_mut(&bid) {
                    let at = idx + applied;
                    block.instructions.insert(at, MirInstruction::Copy { dst: recv_local, src: recv_old });
                    let new_call = match callee_clone {
                        Callee::Method { box_name, method, certainty, .. } => {
                            Callee::Method { box_name, method, receiver: Some(recv_local), certainty }
                        }
                        other => other,
                    };
                    block.instructions[at + 1] = MirInstruction::Call {
                        dst: dst_opt,
                        func: ValueId::new(0),
                        callee: Some(new_call),
                        args: args_clone,
                        effects: effects_clone,
                    };
                    applied += 1;
                    stats.intrinsic_optimizations += 1;
                }
            }
        }
    }
    stats
}
