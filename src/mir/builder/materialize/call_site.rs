use crate::mir::builder::MirBuilder;
use crate::mir::definitions::call_unified::Callee;
use crate::mir::ValueId;

/// MaterializeBox — unify finalization of call operands and receiver copies.
pub fn finalize_call_site(builder: &mut MirBuilder, callee: &mut Callee, args: &mut Vec<ValueId>) {
    // Dev trace (short): dump receiver/args id + type/origin when enabled
    let mat_trace = std::env::var("NYASH_MAT_TRACE").ok().as_deref() == Some("1");
    if mat_trace {
        match callee {
            Callee::Method { box_name, method, receiver, .. } => {
                // Receiver info
                let (rid, rty, rorig) = receiver
                    .and_then(|r| {
                        let ty = builder.value_types.get(&r).cloned();
                        let orig = builder.value_origin_newbox.get(&r).cloned();
                        Some((r, ty, orig))
                    })
                    .unwrap_or((ValueId(u32::MAX), None, None));
                // Args brief
                let mut parts: Vec<String> = Vec::with_capacity(args.len());
                for a in args.iter() {
                    let aty = builder
                        .value_types
                        .get(a)
                        .map(|t| format!("{:?}", t))
                        .unwrap_or_else(|| "?".into());
                    parts.push(format!("%{}:{}", a.0, aty));
                }
                eprintln!(
                    "[mat-trace] call recv=%{} ty={:?} orig={} -> {}.{}({})",
                    rid.0,
                    rty,
                    rorig.as_deref().unwrap_or("-"),
                    box_name,
                    method,
                    parts.join(", ")
                );
            }
            Callee::Global(name) => {
                eprintln!("[mat-trace] call global {}(..)", name);
            }
            _ => {}
        }
    }
    // LocalSSA for receiver and args
    crate::mir::builder::ssa::local::finalize_callee_and_args(builder, callee, args);
    // Keep receiver as is after LocalSSA finalize.
    // The earlier LocalSSA (recv) already materializes a stable in-block id.
    // Avoid late rebind (r -> r2) that can go missing due to block mutations.
}
