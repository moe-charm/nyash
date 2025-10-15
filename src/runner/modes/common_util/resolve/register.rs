//! Registration helpers for using/modules aliases
/// Register collected aliases into the modules registry.
/// Each alias is mapped to its canonical path as a StringBox value.
pub fn register_aliases_in_modules_registry(alias_pairs: &[(String, String)]) {
    if alias_pairs.is_empty() { return; }
    for (alias, canon) in alias_pairs.iter() {
        let sb = nyash_rust::box_trait::StringBox::new(canon.clone());
        nyash_rust::runtime::modules_registry::set(alias.clone(), Box::new(sb));
    }
}
