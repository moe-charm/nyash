// ---- Unified semantics shims (handle-based) ----
// Exported as: nyash.semantics.add_hh(i64 lhs_handle, i64 rhs_handle) -> i64 (NyashBox handle)
#[export_name = "nyash.semantics.add_hh"]
pub extern "C" fn nyash_semantics_add_hh_export(lhs_h: i64, rhs_h: i64) -> i64 {
    use nyash_rust::runtime::host_handles as handles;
    use nyash_rust::{
        box_trait::{IntegerBox, StringBox},
        runtime::semantics,
    };
    if lhs_h <= 0 || rhs_h <= 0 {
        return 0;
    }
    let lhs = if let Some(obj) = handles::get(lhs_h as u64) {
        obj
    } else {
        return 0;
    };
    let rhs = if let Some(obj) = handles::get(rhs_h as u64) {
        obj
    } else {
        return 0;
    };
    let ls_opt = semantics::coerce_to_string(lhs.as_ref());
    let rs_opt = semantics::coerce_to_string(rhs.as_ref());
    if ls_opt.is_some() || rs_opt.is_some() {
        let ls = ls_opt.unwrap_or_else(|| lhs.to_string_box().value);
        let rs = rs_opt.unwrap_or_else(|| rhs.to_string_box().value);
        let s = format!("{}{}", ls, rs);
        let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> =
            std::sync::Arc::new(StringBox::new(s));
        return handles::to_handle_arc(arc) as i64;
    }
    if let (Some(li), Some(ri)) = (
        semantics::coerce_to_i64(lhs.as_ref()),
        semantics::coerce_to_i64(rhs.as_ref()),
    ) {
        let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> =
            std::sync::Arc::new(IntegerBox::new(li + ri));
        return handles::to_handle_arc(arc) as i64;
    }
    // Fallback: stringify both and concat to preserve total order
    let ls = lhs.to_string_box().value;
    let rs = rhs.to_string_box().value;
    let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> =
        std::sync::Arc::new(StringBox::new(format!("{}{}", ls, rs)));
    handles::to_handle_arc(arc) as i64
}
