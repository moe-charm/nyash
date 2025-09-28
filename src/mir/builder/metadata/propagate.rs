//! MetadataPropagationBox — MIR のメタデータ（型/起源）の伝播
//! 仕様不変・小粒。各所のコピペを置換するための薄い関数郡。

use crate::mir::{MirType, ValueId};
use crate::mir::builder::MirBuilder;

/// src から dst へ builder 内メタデータ（value_types / value_origin_newbox）を伝播する。
#[inline]
pub fn propagate(builder: &mut MirBuilder, src: ValueId, dst: ValueId) {
    if let Some(t) = builder.value_types.get(&src).cloned() {
        builder.value_types.insert(dst, t);
    }
    if let Some(cls) = builder.value_origin_newbox.get(&src).cloned() {
        builder.value_origin_newbox.insert(dst, cls);
    }
}

/// dst に型注釈を明示的に設定し、必要ならば起源情報を消去/維持する。
/// 現状は型のみ設定（挙動不変）。
#[inline]
pub fn propagate_with_override(builder: &mut MirBuilder, dst: ValueId, ty: MirType) {
    builder.value_types.insert(dst, ty);
}

