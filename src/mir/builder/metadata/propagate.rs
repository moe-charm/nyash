//! MetadataPropagationBox — MIR のメタデータ（型/起源）の伝播
//! 仕様不変・小粒。各所のコピペを置換するための薄い関数郡。

use crate::mir::{ValueId};
use crate::mir::builder::MirBuilder;

/// src から dst へ builder 内メタデータ（value_types / value_origin_newbox）を伝播する。
#[inline]
pub fn propagate(builder: &mut MirBuilder, src: ValueId, dst: ValueId) {
    if let Some(t) = builder.value_types.get(&src).cloned() {
        builder.value_types.insert(dst, t);
    }
    if let Some(cls) = builder.origin_get(src).map(|s| s.to_string()) {
        builder.origin_register(dst, cls);
    }
}

