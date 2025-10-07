//! Using/namespace system configuration

pub fn enable_using() -> bool {
    std::env::var("NYASH_USING").ok().as_deref() != Some("0")
}

pub fn using_ast_enabled() -> bool {
    std::env::var("NYASH_USING_AST").ok().as_deref() == Some("1")
}

pub fn using_profile() -> String {
    std::env::var("NYASH_USING_PROFILE").unwrap_or_else(|_| "dev".to_string())
}

pub fn using_is_prod() -> bool {
    using_profile().as_str() == "prod" || using_profile().as_str() == "production"
}

pub fn using_is_dev() -> bool {
    let p = using_profile();
    p.as_str() == "dev" || p.as_str() == "development"
}

pub fn using_is_ci() -> bool {
    using_profile().as_str() == "ci"
}

pub fn using_strict() -> bool {
    std::env::var("NYASH_USING_STRICT").ok().as_deref() == Some("1")
}

pub fn using_namespace_alias() -> bool {
    std::env::var("NYASH_USING_NAMESPACE_ALIAS").ok().as_deref() == Some("1")
}

pub fn allow_using_file() -> bool {
    std::env::var("NYASH_ALLOW_USING_FILE").ok().as_deref() == Some("1")
}

pub fn ns_policy_module_first() -> bool {
    std::env::var("NYASH_NS_POLICY").ok().as_deref() == Some("module-first")
}
