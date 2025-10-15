//! UsingAliasExpandBox — helper for local alias recording and head expansion

use std::collections::HashMap;

/// Record a local namespace alias if the using target looks like a namespace (not a file path).
pub fn record_local_namespace_alias(target: &str, alias: &Option<String>, local: &mut HashMap<String, String>) {
    let looks_like_ns = !target.starts_with('"')
        && !target.starts_with('/')
        && !target.contains(".nyash")
        && !target.contains(".hako")
        && !target.contains(std::path::MAIN_SEPARATOR);
    if looks_like_ns {
        if let Some(a) = alias {
            local.insert(a.clone(), target.to_string());
        }
    }
}

/// Expand `Alias.rest` using provided alias maps; returns Some(rewritten) when expanded
pub fn expand_head_alias(target: &str, aliases: &HashMap<String, String>) -> Option<String> {
    if let Some((head, tail)) = target.split_once('.') {
        if let Some(prefix) = aliases.get(head) {
            return Some(format!("{}.{}", prefix, tail));
        }
    }
    None
}

