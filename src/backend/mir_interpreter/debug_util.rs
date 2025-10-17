use crate::backend::abi_util;
use crate::backend::mir_interpreter::VMValue;
use crate::box_core::NyashBox;

/// Debug-only argument formatter for VM call traces.
///
/// Behavior:
/// - kind: abi_util::tag_of_vm
/// - preview: to_string_box().value, truncated to max_len
/// - Always gated by callers (NYASH_VM_CALL_ARG_TRACE / HAKO_DEBUG_MODULE_FN_ARGS)
pub fn format_arg_debug(v: &VMValue, max_len: usize) -> (String, String) {
    let kind = abi_util::tag_of_vm(v).to_string();
    let mut s = match v {
        VMValue::BoxRef(bx) => bx.to_string_box().value,
        VMValue::Future(f) => f.to_string_box().value,
        VMValue::String(s) => s.clone(),
        _ => v.to_string(),
    };
    if s.len() > max_len { s.truncate(max_len); }
    (kind, s)
}
