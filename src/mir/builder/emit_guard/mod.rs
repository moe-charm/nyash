use crate::mir::builder::MirBuilder;
use crate::mir::definitions::call_unified::Callee;
use crate::mir::ValueId;

// Design note — EmitGuard (Box boundary for callsite safety)
//
// Responsibility
// - Provide a single, mandatory place to "materialize" Method receiver and Call arguments
//   using LocalSSA before a Call is emitted.
// - Downstream normalizers (e.g., normalize_* for string/array length) MUST NOT re‑materialize
//   the receiver/args. They must consume the ids as‑is to avoid generating a fresh, undefined
//   ValueId that would not have a Copy in the current block.
//
// Rules
// - finalize_call_operands MUST be called exactly once, immediately before any
//   MirInstruction::Call emission in unified paths (emit_unified_call).
// - Call normalizers MUST NOT call LocalSSA; they must reuse the already
//   materialized receiver/args provided by EmitGuard.
// - verify_after_call MAY be called right after emission (dev only) to assert block invariants.
//
// Rationale
// - Centralizing LocalSSA at the call boundary removes scattered, duplicate materialization and
//   prevents subtle use‑before‑def bugs (e.g., receiver ids created after a block switch).
// - Keeping normalization pure (no new ids) makes the pipeline easier to reason about.

/// Finalize call operands (receiver/args) using LocalSSA; thin wrapper to centralize usage.
/// Includes optional trace logging (NYASH_MAT_TRACE=1) for debugging.
pub fn finalize_call_operands(builder: &mut MirBuilder, callee: &mut Callee, args: &mut Vec<ValueId>) {
    // Dev trace (short): dump receiver/args id + type/origin when enabled
    let mat_trace = std::env::var("NYASH_MAT_TRACE").ok().as_deref() == Some("1");
    if mat_trace {
        match callee {
            Callee::Method { box_name, method, receiver, .. } => {
                // Receiver info
                let (rid, rty, rorig) = receiver
                    .and_then(|r| {
                        let ty = builder.value_types.get(&r).cloned();
                        let orig = builder.origin_get(r).map(|s| s.to_string());
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

    crate::mir::builder::ssa::local::finalize_callee_and_args(builder, callee, args);
}

/// Verify block schedule invariants after emitting a call (dev-only WARNs inside).
pub fn verify_after_call(builder: &mut MirBuilder) {
    crate::mir::builder::verify::call_order::verify_after_call(builder);
}
