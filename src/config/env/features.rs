//! Misc feature flags configuration

use crate::config::env_helpers::{env_bool, env_bool_default_true, env_flag};

pub fn verify_allow_no_phi() -> bool {
    env_bool("NYASH_VERIFY_ALLOW_NO_PHI")
}

pub fn verify_edge_copy_strict() -> bool {
    env_flag("NYASH_VERIFY_EDGE_COPY_STRICT")
}

pub fn llvm_use_harness() -> bool {
    match std::env::var("NYASH_LLVM_USE_HARNESS").ok().as_deref() {
        Some("0") | Some("false") => false,
        Some("1") | Some("true") => true,
        _ => {
            #[cfg(not(feature = "llvm-inkwell-legacy"))]
            { true }
            #[cfg(feature = "llvm-inkwell-legacy")]
            { false }
        }
    }
}

pub fn check_contracts() -> bool {
    env_bool_default_true("NYASH_CHECK_CONTRACTS")
}

pub fn opt_debug() -> bool {
    env_flag("NYASH_OPT_DEBUG")
}

pub fn opt_diag() -> bool {
    env_flag("NYASH_OPT_DIAG")
}

pub fn opt_diag_forbid_legacy() -> bool {
    env_flag("NYASH_OPT_DIAG_FORBID_LEGACY")
}

pub fn opt_diag_fail() -> bool {
    env_flag("NYASH_OPT_DIAG_FAIL")
}

pub fn rewrite_debug() -> bool {
    env_flag("NYASH_REWRITE_DEBUG")
}

pub fn rewrite_safepoint() -> bool {
    env_flag("NYASH_REWRITE_SAFEPOINT")
}

pub fn rewrite_future() -> bool {
    env_flag("NYASH_REWRITE_FUTURE")
}

pub fn abi_vtable() -> bool {
    env_flag("NYASH_ABI_VTABLE")
}

pub fn abi_strict() -> bool {
    env_flag("NYASH_ABI_STRICT")
}

pub fn operator_box_compare_adopt() -> bool {
    env_bool("NYASH_OPERATOR_BOX_COMPARE_ADOPT")
}

pub fn operator_box_add_adopt() -> bool {
    env_bool("NYASH_OPERATOR_BOX_ADD_ADOPT")
}

pub fn null_missing_box_enabled() -> bool {
    env_flag("NYASH_NULL_MISSING_BOX")
}

pub fn null_strict() -> bool {
    env_flag("NYASH_NULL_STRICT")
}

pub fn cli_verbose() -> bool {
    env_flag("NYASH_CLI_VERBOSE")
        || env_flag("HAKO_CLI_VERBOSE")
        || env_flag("HAKU_CLI_VERBOSE")
        || env_flag("HRN_CLI_VERBOSE")
}

pub fn cli_quiet() -> bool {
    env_flag("NYASH_QUIET") || env_flag("NYASH_CLI_QUIET")
}

pub fn resolve_trace() -> bool {
    env_flag("NYASH_RESOLVE_TRACE")
}

pub fn resolve_trace_json() -> bool {
    env_flag("NYASH_RESOLVE_TRACE_JSON")
}

pub fn import_trace() -> bool {
    env_flag("NYASH_IMPORT_TRACE")
}

pub fn block_postfix_catch() -> bool {
    env_bool("NYASH_BLOCK_POSTFIX_CATCH")
}

pub fn try_result_mode() -> bool {
    env_bool_default_true("NYASH_TRY_RESULT_MODE")
}

/// Builder parameter guard toggle (default ON)
/// Prevents overwriting function parameter registers during MIR emission.
pub fn builder_param_guard_enabled() -> bool {
    // Accept both NYASH_ and HAKO_ prefixes (alias)
    if env_bool_default_true("NYASH_BUILDER_PARAM_GUARD") { return true; }
    if env_bool_default_true("HAKO_BUILDER_PARAM_GUARD") { return true; }
    false
}

pub fn method_catch() -> bool {
    env_bool("NYASH_METHOD_CATCH")
}

pub fn entry_allow_toplevel_main() -> bool {
    env_bool_default_true("NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN")
}

pub fn entry_prefer_static_main() -> bool {
    env_bool_default_true("NYASH_ENTRY_PREFER_STATIC_MAIN")
}

pub fn expr_postfix_catch() -> bool {
    env_bool("NYASH_EXPR_POSTFIX_CATCH")
}

pub fn emit_trace() -> bool {
    env_flag("NYASH_EMIT_TRACE")
}

pub fn prefer_cfg2() -> bool {
    env_flag("NYASH_PREFER_CFG2")
}

pub fn prefer_cfg() -> bool {
    env_flag("NYASH_PREFER_CFG")
}

pub fn macro_selfhost_pre_expand() -> Option<String> {
    use crate::config::env_helpers::env_opt;
    env_opt("NYASH_MACRO_SELFHOST_PRE_EXPAND")
}
