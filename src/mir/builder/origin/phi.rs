use super::super::{MirBuilder, MirType, ValueId, BasicBlockId};

/// Lightweight propagation at PHI when all inputs agree（型/起源）。
/// 仕様は不変: 一致時のみ dst にコピーする（不一致/未知は何もしない）。
pub(crate) fn propagate_phi_meta(
    builder: &mut MirBuilder,
    dst: ValueId,
    inputs: &Vec<(BasicBlockId, ValueId)>,
) {
    // Type一致のときだけコピー
    let mut common_ty: Option<MirType> = None;
    let mut ty_agree = true;
    for (_bb, v) in inputs.iter() {
        if let Some(t) = builder.value_types.get(v).cloned() {
            match &common_ty {
                None => common_ty = Some(t),
                Some(ct) => {
                    if ct != &t { ty_agree = false; break; }
                }
            }
        } else { ty_agree = false; break; }
    }
    if ty_agree {
        if let Some(ct) = common_ty { builder.value_types.insert(dst, ct); }
    }
    // Origin一致のときだけコピー
    let mut common_cls: Option<String> = None;
    let mut cls_agree = true;
    for (_bb, v) in inputs.iter() {
        if let Some(c) = builder.value_origin_newbox.get(v).cloned() {
            match &common_cls {
                None => common_cls = Some(c),
                Some(cc) => { if cc != &c { cls_agree = false; break; } }
            }
        } else { cls_agree = false; break; }
    }
    if cls_agree { if let Some(cc) = common_cls { builder.value_origin_newbox.insert(dst, cc); } }
}
