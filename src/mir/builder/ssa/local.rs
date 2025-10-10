use crate::mir::builder::MirBuilder;
use crate::mir::{ValueId, Callee};

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
    if let Some(bb) = bb_opt {
        if std::env::var("NYASH_LOCAL_SSA_TRACE").ok().as_deref() == Some("1") {
            eprintln!("[local-ssa] ensure bb={:?} kind={:?} v=%{}", bb, kind, v.0);
        }
        let key = (bb, v, kind.tag());
        if let Some(&loc) = builder.local_ssa_map.get(&key) {
            return loc;
        }
        let loc = builder.value_gen.next();
        // Best-effort: errors are propagated by caller; we ignore here to keep helper infallible
        let _ = builder.emit_instruction(crate::mir::MirInstruction::Copy { dst: loc, src: v });
        if std::env::var("NYASH_LOCAL_SSA_TRACE").ok().as_deref() == Some("1") {
            eprintln!("[local-ssa] copy  bb={:?} kind={:?} %{} -> %{}", bb, kind, v.0, loc.0);
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
    if std::env::var("NYASH_LOCAL_SSA_TRACE").ok().as_deref() == Some("1") {
        if let Some(bb) = builder.current_block { eprintln!("[local-ssa] finalize-branch bb={:?} cond=%{}", bb, condition_v.0); }
    }
}

/// Finalize compare operands just before emitting a Compare.
/// Applies in-block materialization to both lhs and rhs.
pub fn finalize_compare(builder: &mut MirBuilder, lhs: &mut ValueId, rhs: &mut ValueId) {
    *lhs = cmp_operand(builder, *lhs);
    *rhs = cmp_operand(builder, *rhs);
    if std::env::var("NYASH_LOCAL_SSA_TRACE").ok().as_deref() == Some("1") {
        if let Some(bb) = builder.current_block { eprintln!("[local-ssa] finalize-compare bb={:?} lhs=%{} rhs=%{}", bb, lhs.0, rhs.0); }
        // Optional varmap mapping lines for compare operands (dev-only)
        crate::mir::builder::observe::varmap::emit_recv_names(builder, *lhs, "cmp-lhs");
        crate::mir::builder::observe::varmap::emit_recv_names(builder, *rhs, "cmp-rhs");
    }
}

/// Finalize field use sites: ensure base and all args are in the current block.
pub fn finalize_field_base_and_args(builder: &mut MirBuilder, base: &mut ValueId, args: &mut Vec<ValueId>) {
    *base = field_base(builder, *base);
    for a in args.iter_mut() { *a = arg(builder, *a); }
    if std::env::var("NYASH_LOCAL_SSA_TRACE").ok().as_deref() == Some("1") {
        if let Some(bb) = builder.current_block { eprintln!("[local-ssa] finalize-field bb={:?} base=%{} argc={}", bb, base.0, args.len()); }
    }
}
