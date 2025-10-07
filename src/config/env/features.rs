//! Misc feature flags configuration

pub fn verify_allow_no_phi() -> bool {
    match std::env::var("NYASH_VERIFY_ALLOW_NO_PHI").ok().as_deref() {
        Some("1") | Some("true") => true,
        _ => false,
    }
}

pub fn verify_edge_copy_strict() -> bool {
    std::env::var("NYASH_VERIFY_EDGE_COPY_STRICT").ok().as_deref() == Some("1")
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
    match std::env::var("NYASH_CHECK_CONTRACTS").ok().as_deref() {
        Some("0") | Some("false") | Some("off") => false,
        _ => true,
    }
}

pub fn opt_debug() -> bool {
    std::env::var("NYASH_OPT_DEBUG").ok().as_deref() == Some("1")
}

pub fn opt_diag() -> bool {
    std::env::var("NYASH_OPT_DIAG").ok().as_deref() == Some("1")
}

pub fn opt_diag_forbid_legacy() -> bool {
    std::env::var("NYASH_OPT_DIAG_FORBID_LEGACY").ok().as_deref() == Some("1")
}

pub fn opt_diag_fail() -> bool {
    std::env::var("NYASH_OPT_DIAG_FAIL").ok().as_deref() == Some("1")
}

pub fn rewrite_debug() -> bool {
    std::env::var("NYASH_REWRITE_DEBUG").ok().as_deref() == Some("1")
}

pub fn rewrite_safepoint() -> bool {
    std::env::var("NYASH_REWRITE_SAFEPOINT").ok().as_deref() == Some("1")
}

pub fn rewrite_future() -> bool {
    std::env::var("NYASH_REWRITE_FUTURE").ok().as_deref() == Some("1")
}

pub fn abi_vtable() -> bool {
    std::env::var("NYASH_ABI_VTABLE").ok().as_deref() == Some("1")
}

pub fn abi_strict() -> bool {
    std::env::var("NYASH_ABI_STRICT").ok().as_deref() == Some("1")
}

pub fn operator_box_compare_adopt() -> bool {
    match std::env::var("NYASH_OPERATOR_BOX_COMPARE_ADOPT").ok().as_deref() {
        Some("1") | Some("true") | Some("on") => true,
        _ => false,
    }
}

pub fn operator_box_add_adopt() -> bool {
    match std::env::var("NYASH_OPERATOR_BOX_ADD_ADOPT").ok().as_deref() {
        Some("1") | Some("true") | Some("on") => true,
        _ => false,
    }
}

pub fn null_missing_box_enabled() -> bool {
    std::env::var("NYASH_NULL_MISSING_BOX").ok().as_deref() == Some("1")
}

pub fn null_strict() -> bool {
    std::env::var("NYASH_NULL_STRICT").ok().as_deref() == Some("1")
}

pub fn cli_verbose() -> bool {
    std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1")
        || std::env::var("HAKO_CLI_VERBOSE").ok().as_deref() == Some("1")
        || std::env::var("HAKU_CLI_VERBOSE").ok().as_deref() == Some("1")
        || std::env::var("HRN_CLI_VERBOSE").ok().as_deref() == Some("1")
}

pub fn cli_quiet() -> bool {
    std::env::var("NYASH_QUIET").ok().as_deref() == Some("1")
        || std::env::var("NYASH_CLI_QUIET").ok().as_deref() == Some("1")
}

pub fn resolve_trace() -> bool {
    std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1")
}

pub fn resolve_trace_json() -> bool {
    std::env::var("NYASH_RESOLVE_TRACE_JSON").ok().as_deref() == Some("1")
}

pub fn import_trace() -> bool {
    std::env::var("NYASH_IMPORT_TRACE").ok().as_deref() == Some("1")
}

pub fn block_postfix_catch() -> bool {
    match std::env::var("NYASH_BLOCK_POSTFIX_CATCH").ok().as_deref() {
        Some("1") | Some("true") | Some("on") => true,
        _ => false,
    }
}

pub fn try_result_mode() -> bool {
    match std::env::var("NYASH_TRY_RESULT_MODE").ok().as_deref() {
        Some("0") | Some("false") | Some("off") => false,
        _ => true,
    }
}

pub fn method_catch() -> bool {
    match std::env::var("NYASH_METHOD_CATCH").ok().as_deref() {
        Some("1") | Some("true") | Some("on") => true,
        _ => false,
    }
}

pub fn entry_allow_toplevel_main() -> bool {
    match std::env::var("NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN").ok().as_deref() {
        Some("0") | Some("false") | Some("off") => false,
        _ => true,
    }
}

pub fn entry_prefer_static_main() -> bool {
    match std::env::var("NYASH_ENTRY_PREFER_STATIC_MAIN").ok().as_deref() {
        Some("0") | Some("false") | Some("off") => false,
        _ => true,
    }
}

pub fn expr_postfix_catch() -> bool {
    match std::env::var("NYASH_EXPR_POSTFIX_CATCH").ok().as_deref() {
        Some("1") | Some("true") | Some("on") => true,
        _ => false,
    }
}

pub fn emit_trace() -> bool {
    std::env::var("NYASH_EMIT_TRACE").ok().as_deref() == Some("1")
}

pub fn prefer_cfg2() -> bool {
    std::env::var("NYASH_PREFER_CFG2").ok().as_deref() == Some("1")
}

pub fn prefer_cfg() -> bool {
    std::env::var("NYASH_PREFER_CFG").ok().as_deref() == Some("1")
}

pub fn macro_selfhost_pre_expand() -> Option<String> {
    std::env::var("NYASH_MACRO_SELFHOST_PRE_EXPAND").ok()
}
