//! Centralized helpers to access core Box type IDs.
//! Note: IDs may be configured via hako.toml; these helpers query the runtime registry.

use once_cell::sync::OnceCell;

fn resolve(name: &str, fallback: u32, cell: &OnceCell<u32>) -> u32 {
    *cell.get_or_init(|| {
        crate::runtime::type_registry::builtin_type_id(name).unwrap_or(fallback)
    })
}

static MAP_ID: OnceCell<u32> = OnceCell::new();
static ARRAY_ID: OnceCell<u32> = OnceCell::new();
static STRING_ID: OnceCell<u32> = OnceCell::new();

#[inline]
pub fn map() -> u32 {
    resolve("MapBox", 11, &MAP_ID)
}

#[inline]
pub fn array() -> u32 {
    resolve("ArrayBox", 12, &ARRAY_ID)
}

#[inline]
pub fn string() -> u32 {
    resolve("StringBox", 13, &STRING_ID)
}

/// Resolve a builtin TypeBox identifier by name (using helpers where possible).
pub fn by_name(type_name: &str) -> Option<u32> {
    match type_name {
        "MapBox" => Some(map()),
        "ArrayBox" => Some(array()),
        "StringBox" => Some(string()),
        _ => crate::runtime::type_registry::builtin_type_id(type_name),
    }
}
