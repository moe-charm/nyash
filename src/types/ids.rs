//! Centralized helpers to access core Box type IDs.
//! Note: IDs may be configured via hako.toml; these helpers query the runtime registry.

#[inline]
pub fn map() -> u32 {
    crate::runtime::type_registry::builtin_type_id("MapBox").unwrap_or(11)
}

#[inline]
pub fn array() -> u32 {
    crate::runtime::type_registry::builtin_type_id("ArrayBox").unwrap_or(12)
}

#[inline]
pub fn string() -> u32 {
    crate::runtime::type_registry::builtin_type_id("StringBox").unwrap_or(13)
}

