//! MIR compiler configuration

pub fn mir_no_phi() -> bool {
    std::env::var("NYASH_MIR_NO_PHI").ok().as_deref() == Some("1")
}

pub fn mir_core13() -> bool {
    match std::env::var("NYASH_MIR_CORE13").ok().as_deref() {
        Some("0") | Some("false") | Some("off") => false,
        _ => true,
    }
}

pub fn mir_core13_pure() -> bool {
    match std::env::var("NYASH_MIR_CORE13_PURE").ok().as_deref() {
        Some("1") | Some("true") | Some("on") => true,
        _ => false,
    }
}

// Retired toggles: Ref/Array legacy normalization is always-on.

pub fn mir_pre_pin_compare_operands() -> bool {
    match std::env::var("NYASH_MIR_PRE_PIN_COMPARE_OPERANDS").ok().as_deref() {
        Some("1") | Some("true") | Some("on") => true,
        _ => false,
    }
}
