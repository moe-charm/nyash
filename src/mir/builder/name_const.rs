use crate::mir::ValueId;
use crate::mir::builder::MirBuilder;

/// Emit a string Const (function name const) and return its ValueId.
/// Behavior-preserving wrapper around Const emission with String value.
pub fn make_name_const_result(b: &mut MirBuilder, name: &str) -> Result<ValueId, String> {
    // Delegate to ConstantEmissionBox to keep Const emission centralized
    let dst = crate::mir::builder::emission::constant::emit_string(b, name.to_string());
    Ok(dst)
}
