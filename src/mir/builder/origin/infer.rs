use super::super::{MirBuilder, MirType, ValueId};

/// Annotate the origin of `me`/receiver with a Known class when分かる範囲のみ。
/// - 優先: current_static_box（静的ボックスの文脈）
/// - 次点: 現在の関数名のプレフィックス（"Class.method/Arity"）
/// - それ以外: 付与せず（挙動不変）
pub(crate) fn annotate_me_origin(builder: &mut MirBuilder, me_id: ValueId) {
    let mut cls: Option<String> = None;
    if let Some(c) = builder.current_static_box.clone() {
        if !c.is_empty() { cls = Some(c); }
    }
    if cls.is_none() {
        if let Some(ref fun) = builder.current_function {
            if let Some(dot) = fun.signature.name.find('.') {
                let c = fun.signature.name[..dot].to_string();
                if !c.is_empty() { cls = Some(c); }
            }
        }
    }
    if let Some(c) = cls {
        // Record both origin class and a Box type hint for downstream passes（観測用）。
        builder.value_origin_newbox.insert(me_id, c.clone());
        builder.value_types.insert(me_id, MirType::Box(c));
    }
}
