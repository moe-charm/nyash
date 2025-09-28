use crate::mir::builder::MirBuilder;
use crate::mir::definitions::call_unified::TypeCertainty;

/// ResolveTraceBox — dev-only tracing helpers (forward to observe::resolve)
pub fn emit_try_method(builder: &mut MirBuilder, recv_cls: &str, method: &str, arity: usize, candidates: &[String]) {
    let meta = serde_json::json!({
        "recv_cls": recv_cls,
        "method": method,
        "arity": arity,
        "candidates": candidates,
    });
    super::super::observe::resolve::emit_try(builder, meta);
}

pub fn emit_choose_unified(builder: &mut MirBuilder, recv_cls: &str, method: &str, arity: usize, chosen: &str, certainty: &TypeCertainty) {
    let meta = serde_json::json!({
        "recv_cls": recv_cls,
        "method": method,
        "arity": arity,
        "chosen": chosen,
        "certainty": format!("{:?}", certainty),
        "reason": "unified",
    });
    super::super::observe::resolve::emit_choose(builder, meta);
}

