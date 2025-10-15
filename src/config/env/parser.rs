//! Parser configuration

pub fn parser_flow_enabled() -> bool {
    std::env::var("NYASH_PARSER_FLOW").ok().as_deref() != Some("0")
}

pub fn parser_stage3() -> bool {
    std::env::var("NYASH_PARSER_STAGE3").ok().as_deref() == Some("1")
}

pub fn loopform_normalize() -> bool {
    std::env::var("NYASH_LOOPFORM_NORMALIZE").ok().as_deref() == Some("1")
}
