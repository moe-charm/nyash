//! TypeAnnotationBox — MIR 値への型注釈（仕様不変の最小）

use crate::mir::{MirType, ValueId};
use crate::mir::builder::MirBuilder;

/// 直接的に MirType を設定する（仕様不変）。
#[inline]
pub fn set_type(builder: &mut MirBuilder, dst: ValueId, ty: MirType) {
    builder.value_types.insert(dst, ty);
}

/// 関数名から既知の戻り型を注釈する（最小ハードコード）。
/// 例: "StringBox.str/0" → MirType::String
#[inline]
pub fn annotate_from_function(builder: &mut MirBuilder, dst: ValueId, func_name: &str) {
    if let Some(ty) = infer_return_type(func_name) {
        builder.value_types.insert(dst, ty);
    }
}

fn infer_return_type(func_name: &str) -> Option<MirType> {
    // Very small whitelist; 仕様不変（既知の戻り型のみ）
    // Normalize forms like "JsonNode.str/0" or "StringBox.length/0" if needed
    if func_name.ends_with(".str/0") { return Some(MirType::String); }
    if func_name.ends_with(".length/0") { return Some(MirType::Integer); }
    if func_name.ends_with(".size/0") { return Some(MirType::Integer); }
    if func_name.ends_with(".len/0") { return Some(MirType::Integer); }
    if func_name.ends_with(".substring/2") { return Some(MirType::String); }
    if func_name.ends_with(".esc_json/0") { return Some(MirType::String); }
    if func_name.ends_with(".indexOf/1") { return Some(MirType::Integer); }
    if func_name.ends_with(".lastIndexOf/1") { return Some(MirType::Integer); }
    if func_name.ends_with(".is_digit_char/1") { return Some(MirType::Bool); }
    if func_name.ends_with(".is_hex_digit_char/1") { return Some(MirType::Bool); }
    if func_name.ends_with(".is_alpha_char/1") { return Some(MirType::Bool); }
    if func_name.ends_with("MapBox.has/1") { return Some(MirType::Bool); }
    // Fallback: none (変更なし)
    None
}
