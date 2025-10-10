/// Compatibility shims — delegate to DiagnosticsBox
pub(crate) fn dbg_on() -> bool { crate::runtime::diagnostics::dbg_on() }
pub(super) fn dbg_once(key: &str, msg: &str) { crate::runtime::diagnostics::dbg_once(key, msg) }
