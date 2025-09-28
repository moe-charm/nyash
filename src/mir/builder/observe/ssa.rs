use super::super::{BasicBlockId, MirBuilder, ValueId};

/// Emit a dev‑only JSONL event for a PHI decision.
/// Computes predecessor meta（type/origin）from the builder’s current maps.
pub(crate) fn emit_phi(builder: &MirBuilder, dst: ValueId, inputs: &Vec<(BasicBlockId, ValueId)>) {
    // Respect env gates in hub; just build meta here.
    let preds: Vec<serde_json::Value> = inputs
        .iter()
        .map(|(bb, v)| {
            let t = builder.value_types.get(v).cloned();
            let o = builder.value_origin_newbox.get(v).cloned();
            serde_json::json!({
                "bb": bb.0,
                "v": v.0,
                "type": t.as_ref().map(|tt| format!("{:?}", tt)).unwrap_or_default(),
                "origin": o.unwrap_or_default(),
            })
        })
        .collect();
    let decided_t = builder
        .value_types
        .get(&dst)
        .cloned()
        .map(|tt| format!("{:?}", tt))
        .unwrap_or_default();
    let decided_o = builder
        .value_origin_newbox
        .get(&dst)
        .cloned()
        .unwrap_or_default();
    let meta = serde_json::json!({
        "dst": dst.0,
        "preds": preds,
        "decided_type": decided_t,
        "decided_origin": decided_o,
    });
    let fn_name = builder
        .current_function
        .as_ref()
        .map(|f| f.signature.name.as_str());
    let region = builder.debug_current_region_id();
    crate::debug::hub::emit("ssa", "phi", fn_name, region.as_deref(), meta);
}
