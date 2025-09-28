use crate::mir::builder::MirBuilder;

/// RewriteGateBox — centralize Known rewrite gating policy.
/// Rules (behavior-preserving):
/// - Only user-defined boxes対象
/// - StringBox の {length,len,substring,indexOf,lastIndexOf} は除外
/// - (Box,method,arity) が instance_method_index に存在する
pub fn should_rewrite(builder: &MirBuilder, cls: &str, method: &str, arity: usize) -> bool {
    if !builder.user_defined_boxes.contains(cls) {
        if std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1") {
            eprintln!("[rewrite.gate] skip: not userbox cls={}", cls);
        }
        return false;
    }
    if cls == "StringBox" && matches!(method, "length" | "len" | "substring" | "indexOf" | "lastIndexOf") {
        if std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1") {
            eprintln!("[rewrite.gate] skip: string-api {} on StringBox", method);
        }
        return false;
    }
    let ok = crate::mir::builder::indexes::instance::exists(builder, cls, method, arity);
    if std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1") {
        eprintln!("[rewrite.gate] {} cls={} m={}/{}", if ok {"hit"} else {"miss"}, cls, method, arity);
    }
    ok
}
