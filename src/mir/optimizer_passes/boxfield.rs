use crate::mir::optimizer::MirOptimizer;
use crate::mir::optimizer_stats::OptimizationStats;
use crate::mir::{MirInstruction as I, MirModule};

/// Optimize BoxField operations (scaffolding)
pub fn optimize_boxfield_operations(
    opt: &mut MirOptimizer,
    module: &mut MirModule,
) -> OptimizationStats {
    let mut stats = OptimizationStats::new();
    for (func_name, function) in &mut module.functions {
        if opt.debug_enabled() {
            println!("  📦 BoxField optimization in function: {}", func_name);
        }
        for (_bb_id, block) in &mut function.blocks {
            let mut changed = 0usize;
            let mut out: Vec<I> = Vec::with_capacity(block.instructions.len());
            let mut i = 0usize;
            while i < block.instructions.len() {
                // Look ahead for simple store-followed-by-load on same box/index
                if i + 1 < block.instructions.len() {
                    match (&block.instructions[i], &block.instructions[i + 1]) {
                        (
                            I::BoxCall {
                                box_val: b1,
                                method: m1,
                                args: a1,
                                ..
                            },
                            I::BoxCall {
                                dst: Some(dst2),
                                box_val: b2,
                                method: m2,
                                args: a2,
                                ..
                            },
                        ) if (m1 == "set" || m1 == "setField")
                            && (m2 == "get" || m2 == "getField") =>
                        {
                            // set(arg0=index/key, arg1=value), then get(arg0=index/key)
                            if b1 == b2 && a1.len() == 2 && a2.len() == 1 && a1[0] == a2[0] {
                                // Replace the second with Copy from just-stored value
                                let src_val = a1[1];
                                out.push(block.instructions[i].clone());
                                out.push(I::Copy {
                                    dst: *dst2,
                                    src: src_val,
                                });
                                changed += 1;
                                i += 2;
                                continue;
                            }
                        }
                        _ => {}
                    }
                }
                out.push(block.instructions[i].clone());
                i += 1;
            }
            if changed > 0 {
                block.instructions = out;
                stats.boxfield_optimizations += changed;
            }
        }
    }
    stats
}
