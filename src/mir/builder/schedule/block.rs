use crate::mir::builder::MirBuilder;
use crate::mir::{MirInstruction};

/// BlockScheduleBox — manage physical insertion points within a block.
/// Contract: PHI group → materialize group (Copy/Id) → body (Call etc.)
pub struct BlockScheduleBox;

impl BlockScheduleBox {
    /// Dev-only: verify simple block order invariants.
    /// - PHI group must be at the block head (no PHI after first non-PHI)
    /// - If a Copy immediately precedes a Call-like instruction, prefer that Copy's src
    ///   to be the previously materialized after-PHIs id (best-effort warning only).
    pub fn verify_order(builder: &mut MirBuilder) {
        if std::env::var("NYASH_BLOCK_SCHEDULE_VERIFY").ok().as_deref() != Some("1") {
            return;
        }
        let (f_opt, bb_opt) = (builder.current_function.as_ref(), builder.current_block);
        let (Some(fun), Some(bb_id)) = (f_opt, bb_opt) else { return; };
        let Some(bb) = fun.get_block(bb_id) else { return; };

        // 1) PHI group must be at head
        let mut seen_non_phi = false;
        for (idx, inst) in bb.instructions.iter().enumerate() {
            match inst {
                MirInstruction::Phi { .. } => {
                    if seen_non_phi {
                        eprintln!("[block-schedule][verify] WARN: PHI found after non-PHI at bb={:?} idx={}", bb_id, idx);
                    }
                }
                _ => { seen_non_phi = true; }
            }
        }

        // 2) If a Copy is immediately before a Call-like, prefer it to be derived from after-PHIs copy
        let is_call_like = |mi: &MirInstruction| -> bool {
            matches!(mi, MirInstruction::Call { .. } | MirInstruction::BoxCall { .. })
        };
        for w in bb.instructions.windows(2) {
            if let [MirInstruction::Copy { dst: _, src }, call] = w {
                if is_call_like(call) {
                    // best-effort: src should be one of the after-PHIs materialized ids for this bb
                    let derived_ok = builder
                        .schedule_mat_map
                        .values()
                        .any(|&v| v == *src);
                    if !derived_ok {
                        eprintln!(
                            "[block-schedule][verify] WARN: tail Copy src=%{} is not from after-PHIs in bb={:?}",
                            src.0, bb_id
                        );
                    }
                }
            }
        }
    }
}
