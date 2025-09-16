pub fn dump_cfg_dot(
    func: &crate::mir::MirFunction,
    path: &str,
    phi_min: bool,
) -> std::io::Result<()> {
    let mut out = String::new();
    out.push_str(&format!("digraph \"{}\" {{\n", func.signature.name));
    out.push_str("  node [shape=box, fontsize=10];\n");
    // Derive simple bool sets: compare dsts are bool; phi of all-bool inputs are bool
    let mut bool_values: std::collections::HashSet<crate::mir::ValueId> =
        std::collections::HashSet::new();
    for (_bb_id, bb) in func.blocks.iter() {
        for ins in bb.instructions.iter() {
            if let crate::mir::MirInstruction::Compare { dst, .. } = ins {
                bool_values.insert(*dst);
            }
        }
    }
    let mut bool_phi: std::collections::HashSet<crate::mir::ValueId> =
        std::collections::HashSet::new();
    if phi_min {
        for (_bb_id, bb) in func.blocks.iter() {
            for ins in bb.instructions.iter() {
                if let crate::mir::MirInstruction::Phi { dst, inputs } = ins {
                    if !inputs.is_empty() && inputs.iter().all(|(_, v)| bool_values.contains(v)) {
                        bool_phi.insert(*dst);
                    }
                }
            }
        }
    }
    // Sort blocks for deterministic output
    let mut bb_ids: Vec<_> = func.blocks.keys().copied().collect();
    bb_ids.sort_by_key(|b| b.0);
    // Emit nodes with labels
    for bb_id in bb_ids.iter() {
        let bb = func.blocks.get(bb_id).unwrap();
        let phi_count = bb
            .instructions
            .iter()
            .filter(|ins| matches!(ins, crate::mir::MirInstruction::Phi { .. }))
            .count();
        let phi_b1_count = bb
            .instructions
            .iter()
            .filter(|ins| match ins {
                crate::mir::MirInstruction::Phi { dst, .. } => bool_phi.contains(dst),
                _ => false,
            })
            .count();
        let mut label = format!("bb{}", bb_id.0);
        if phi_min && phi_count > 0 {
            if phi_b1_count > 0 {
                label = format!("{}\\nphi:{} (b1:{})", label, phi_count, phi_b1_count);
            } else {
                label = format!("{}\\nphi:{}", label, phi_count);
            }
        }
        if *bb_id == func.entry_block {
            label = format!("{}\\nENTRY", label);
        }
        out.push_str(&format!("  n{} [label=\"{}\"];\n", bb_id.0, label));
    }
    // Emit edges based on terminators
    for bb_id in bb_ids.iter() {
        let bb = func.blocks.get(bb_id).unwrap();
        if let Some(term) = &bb.terminator {
            match term {
                crate::mir::MirInstruction::Jump { target } => {
                    out.push_str(&format!("  n{} -> n{};\n", bb_id.0, target.0));
                }
                crate::mir::MirInstruction::Branch {
                    then_bb, else_bb, ..
                } => {
                    // Branch condition is boolean (b1)
                    out.push_str(&format!(
                        "  n{} -> n{} [label=\"then cond:b1\"];\n",
                        bb_id.0, then_bb.0
                    ));
                    out.push_str(&format!(
                        "  n{} -> n{} [label=\"else cond:b1\"];\n",
                        bb_id.0, else_bb.0
                    ));
                }
                _ => {}
            }
        }
    }
    out.push_str("}\n");
    std::fs::write(path, out)
}
