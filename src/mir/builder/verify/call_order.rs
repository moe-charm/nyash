use crate::mir::builder::MirBuilder;

/// CallOrderVerifyBox — wrapper for dev-only schedule checks
pub fn verify_after_call(builder: &mut MirBuilder) {
    crate::mir::builder::schedule::block::BlockScheduleBox::verify_order(builder);
    if std::env::var("NYASH_MAT_TRACE").ok().as_deref() == Some("1") {
        if let (Some(fun), Some(bb_id)) = (builder.current_function.as_ref(), builder.current_block) {
            if let Some(bb) = fun.get_block(bb_id) {
                let n = bb.instructions.len();
                let start = n.saturating_sub(5);
                eprintln!("[mat-trace] bb={:?} last_insts:", bb_id);
                for (i, inst) in bb.instructions.iter().enumerate().skip(start) {
                    eprintln!("  {:04}: {:?}", i, inst);
                }
            }
        }
    }
}
