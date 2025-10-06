//! UsingNamespaceResolverBox — shared helpers for alias/namespaces
//!
//! Provides helpers to:
//! - Expand head alias (Alias.rest -> canonical)
//! - Accept pure namespace alias when [modules] has children under the prefix

use std::collections::HashMap;

pub fn has_children_under(prefix: &str, modules: &[(String, String)]) -> bool {
    let pref = format!("{}.", prefix);
    modules.iter().any(|(ns, _)| ns.starts_with(&pref))
}

pub fn accept_namespace_alias_if_modules_have_children(
    target: &str,
    alias: &Option<String>,
    modules: &[(String, String)],
    seen_aliases: &mut HashMap<String, (String, usize)>,
    alias_pairs: &mut Vec<(String, String)>,
    line_no: usize,
    verbose: bool,
) -> bool {
    if target.starts_with('"') || target.starts_with('/') || target.contains(".nyash") || target.contains(".hako") || target.contains(std::path::MAIN_SEPARATOR) {
        return false;
    }
    if !has_children_under(target, modules) { return false; }
    if let Some(a) = alias {
        if !seen_aliases.contains_key(a) {
            seen_aliases.insert(a.clone(), (target.to_string(), line_no));
            alias_pairs.push((a.clone(), target.to_string()));
            if verbose && !crate::config::env::cli_quiet() { eprintln!("[using] dev: alias '{}' for prefix '{}'", a, target); }
        }
    }
    true
}
