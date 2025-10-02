use crate::mir::{MirType, ValueId};
use crate::mir::definitions::call_unified::TypeCertainty;
use std::collections::HashMap;

/// ReceiverInferenceBox — decide class name and certainty for a method receiver.
/// Priority: explicit hint → origin (NewBox) → value_types → UnknownBox.
/// Normalizes primitives (String → StringBox) and applies narrow safe heuristics
/// to avoid accidental binding (e.g., ParserBox vs String methods).
pub fn infer_receiver<F>(
    box_hint: Option<&str>,
    method: &str,
    receiver: ValueId,
    origin_lookup: F,
    value_types: &HashMap<ValueId, MirType>,
) -> (String, TypeCertainty)
where
    F: Fn(ValueId) -> Option<String>,
{
    // 1) Prefer NewBox origin first（Known が最優先: 起源が明確なら型汚染の影響を受けにくい）
    if let Some(name) = origin_lookup(receiver) {
        // If the origin is a user instance but the method is a well-known string API,
        // prefer StringBox to avoid mis-binding (behavioral no-op for typical code where
        // these APIs target strings). This prevents routing to InstanceBox.* for
        // substring/indexOf/lastIndexOf/length/len which is unsupported.
        if name == "InstanceBox" && matches!(method,
            "length" | "len" | "substring" | "indexOf" | "lastIndexOf"
        ) {
            return ("StringBox".to_string(), TypeCertainty::Union);
        }
        return (name, TypeCertainty::Known);
    }

    // 2) 次に MIR types（原始型は正規化）。String は StringBox へ。
    if let Some(t) = value_types.get(&receiver) {
        match t {
            MirType::Box(b) => return (b.clone(), TypeCertainty::Union),
            MirType::String => return ("StringBox".to_string(), TypeCertainty::Union),
            _ => {}
        }
    }

    // 3) Hint (soft). If string-like method and hint == ParserBox, correct to StringBox.
    if let Some(h) = box_hint {
        let corrected = if h == "ParserBox" && matches!(method,
            "length" | "len" | "substring" | "indexOf" | "lastIndexOf"
        ) { "StringBox" } else { h };
        return (corrected.to_string(), TypeCertainty::Union);
    }

    // 4) Fallback Unknown (heuristic for common string APIs)
    let mut name = "UnknownBox".to_string();
    let certainty = TypeCertainty::Union;

    // Heuristic safety: if later logic saw ParserBox but the type said String,
    // callers may pass (Some("ParserBox"), recv_ty=String). Cover the common string APIs.
    if matches!(method, "length" | "len" | "substring" | "indexOf" | "lastIndexOf") {
        name = "StringBox".to_string();
    }

    (name, certainty)
}
