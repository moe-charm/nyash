use crate::mir::builder::MirBuilder;
use crate::mir::{ValueId, Callee};
use crate::common::trace_box::TraceBox;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum LocalKind {
    Recv,
    Arg,
    CompareOperand,
    Cond,
    FieldBase,
    Other(u8),
}

impl LocalKind {
    #[inline]
    fn tag(self) -> u8 {
        match self {
            LocalKind::Recv => 0,
            LocalKind::Arg => 1,
            LocalKind::CompareOperand => 2,
            LocalKind::Cond => 4,
            LocalKind::FieldBase => 0, // share recv slot for bases
            LocalKind::Other(k) => k,
        }
    }
}

/// Ensure a value has an in-block definition and cache it per (bb, orig, kind).
/// Always emits a Copy in the current block when not cached.
pub fn ensure(builder: &mut MirBuilder, v: ValueId, kind: LocalKind) -> ValueId {
    let bb_opt = builder.current_block;
    TraceBox::local_ssa(|| format!("[local-ssa] ensure ENTRY bb_opt={:?} kind={:?} v=%{}", bb_opt, kind, v.0));
    if let Some(bb) = bb_opt {
        TraceBox::local_ssa(|| format!("[local-ssa] ensure bb={:?} kind={:?} v=%{}", bb, kind, v.0));
        let key = (bb, v, kind.tag());
        if let Some(&loc) = builder.local_ssa_map.get(&key) {
            return loc;
        }
        // Phase 2.2: Avoid function parameters (v%0-v%N) - never reuse parameter registers
        // Phase 2.P2: Avoid variable_map collision - never reuse existing local variables
        // Phase 2.P2+: Also check value_types (all defined ValueIds, including PHI sources)
        // Phase 2.P2++: Also check local_ssa_map (all cached local SSA copies)
        let mut loc = builder.safe_next_value();
        // Ensure the freshly allocated ValueId never aliases:
        // - the source value (v)
        // - function parameters (fun.params)
        // - existing local variables (variable_map)
        // - any defined values (value_types) - prevents SSA violations
        // - cached local SSA copies (local_ssa_map) - prevents re-allocation conflicts
        let mut attempts = 0;
        if let Some(ref fun) = builder.current_function {
            while loc == v
                || fun.params.contains(&loc)
                || builder.variable_map.values().any(|&vid| vid == loc)
                || builder.value_types.contains_key(&loc)
                || builder.local_ssa_map.values().any(|&vid| vid == loc)
            {
                loc = builder.value_gen.next();
                attempts += 1;
                if attempts > 1000 {
                    panic!(
                        "ValueId allocation loop detected at bb={:?} v=%{} kind={:?} (attempts={})",
                        bb, v.0, kind, attempts
                    );
                }
            }
        } else {
            while loc == v
                || builder.variable_map.values().any(|&vid| vid == loc)
                || builder.value_types.contains_key(&loc)
                || builder.local_ssa_map.values().any(|&vid| vid == loc)
            {
                loc = builder.value_gen.next();
                attempts += 1;
                if attempts > 1000 {
                    panic!(
                        "ValueId allocation loop detected at bb={:?} v=%{} kind={:?} (attempts={})",
                        bb, v.0, kind, attempts
                    );
                }
            }
        }
        // Best-effort: errors are propagated by caller; we log but ignore to keep helper infallible
        TraceBox::local_ssa(|| {
            let fn_name = builder.current_function.as_ref().map(|f| f.signature.name.as_str()).unwrap_or("<none>");
            format!("[local-ssa] BEFORE emit_instruction fn={} current_block={:?} target_bb={:?}", fn_name, builder.current_block, bb)
        });
        if let Err(e) = builder.emit_instruction(crate::mir::MirInstruction::Copy { dst: loc, src: v }) {
            TraceBox::local_ssa(|| format!("[local-ssa] FAILED copy bb={:?} kind={:?} %{} -> %{} error={}", bb, kind, v.0, loc.0, e));
        } else {
            TraceBox::local_ssa(|| format!("[local-ssa] copy  bb={:?} kind={:?} %{} -> %{}", bb, kind, v.0, loc.0));
        }
        if let Some(t) = builder.value_types.get(&v).cloned() {
            builder.value_types.insert(loc, t);
        }
        if let Some(cls) = builder.origin_get(v).map(|s| s.to_string()) {
            builder.origin_register(loc, cls);
        }
        builder.local_ssa_map.insert(key, loc);
        loc
    } else {
        v
    }
}

#[inline]
pub fn recv(builder: &mut MirBuilder, v: ValueId) -> ValueId { ensure(builder, v, LocalKind::Recv) }

#[inline]
pub fn arg(builder: &mut MirBuilder, v: ValueId) -> ValueId { ensure(builder, v, LocalKind::Arg) }

#[inline]
pub fn cond(builder: &mut MirBuilder, v: ValueId) -> ValueId { ensure(builder, v, LocalKind::Cond) }

#[inline]
pub fn field_base(builder: &mut MirBuilder, v: ValueId) -> ValueId { ensure(builder, v, LocalKind::FieldBase) }

#[inline]
pub fn cmp_operand(builder: &mut MirBuilder, v: ValueId) -> ValueId { ensure(builder, v, LocalKind::CompareOperand) }

/// Finalize a callee+args just before emitting a Call instruction:
/// - If Method: ensure receiver is in the current block
/// - All args: ensure in the current block
pub fn finalize_callee_and_args(builder: &mut MirBuilder, callee: &mut Callee, args: &mut Vec<ValueId>) {
    if let Callee::Method { receiver: Some(mut r), box_name, method, certainty } = callee.clone() {
        // Dev-stability: inside ParserBox.* methods, force receiver to `me` param id
        // so we never depend on fragile tail copies for `me`.
        if box_name == "ParserBox" {
            if let Some(fun) = builder.current_function.as_ref() {
                if fun.signature.name.starts_with("ParserBox.") {
                    if let Some(first_param) = fun.params.first() {
                        r = *first_param;
                    }
                }
            }
        }
        // FIX: Materialize receiver in current block (was missing!)
        // This ensures literal.method() has the receiver const emitted.
        r = recv(builder, r);
        *callee = Callee::Method { box_name, method, receiver: Some(r), certainty };
    }
    for a in args.iter_mut() {
        *a = arg(builder, *a);
    }
}

/// Finalize only the args (legacy Call paths)
pub fn finalize_args(builder: &mut MirBuilder, args: &mut Vec<ValueId>) {
    for a in args.iter_mut() {
        *a = arg(builder, *a);
    }
}

/// Finalize a single branch condition just before emitting a Branch.
/// Ensures the condition has a definition in the current block.
pub fn finalize_branch_cond(builder: &mut MirBuilder, condition_v: &mut ValueId) {
    *condition_v = cond(builder, *condition_v);
    if let Some(bb) = builder.current_block {
        TraceBox::local_ssa(|| format!("[local-ssa] finalize-branch bb={:?} cond=%{}", bb, condition_v.0));
    }
}

/// Finalize compare operands just before emitting a Compare.
/// Applies in-block materialization to both lhs and rhs.
pub fn finalize_compare(builder: &mut MirBuilder, lhs: &mut ValueId, rhs: &mut ValueId) {
    *lhs = cmp_operand(builder, *lhs);
    *rhs = cmp_operand(builder, *rhs);
    if let Some(bb) = builder.current_block {
        TraceBox::local_ssa(|| format!("[local-ssa] finalize-compare bb={:?} lhs=%{} rhs=%{}", bb, lhs.0, rhs.0));
        // Optional varmap mapping lines for compare operands (dev-only)
        crate::mir::builder::observe::varmap::emit_recv_names(builder, *lhs, "cmp-lhs");
        crate::mir::builder::observe::varmap::emit_recv_names(builder, *rhs, "cmp-rhs");
    }
}

/// Finalize field use sites: ensure base and all args are in the current block.
pub fn finalize_field_base_and_args(builder: &mut MirBuilder, base: &mut ValueId, args: &mut Vec<ValueId>) {
    *base = field_base(builder, *base);
    for a in args.iter_mut() { *a = arg(builder, *a); }
    if let Some(bb) = builder.current_block {
        TraceBox::local_ssa(|| format!("[local-ssa] finalize-field bb={:?} base=%{} argc={}", bb, base.0, args.len()));
    }
}
