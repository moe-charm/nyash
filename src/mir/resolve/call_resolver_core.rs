/// Resolve a ModuleFunction name from a raw name and argc against a set of keys.
/// Strategy (ordered):
/// 1) Exact match
/// 2) If no `/arity`, append and try again
/// 3) Tail-based fallback: `.method/arity` with `Class.` or `Class_` prefix
/// 4) Heuristic: `Alias_Alias.method/arity`
/// 5) Final: any unique key that ends with `.method/arity`
pub fn resolve_module_function(
    keys_iter: impl IntoIterator<Item = String>,
    raw_name: &str,
    argc: usize,
) -> Option<String> {
    let keys: Vec<String> = keys_iter.into_iter().collect();
    // 1) Exact
    if keys.iter().any(|k| k == raw_name) {
        return Some(raw_name.to_string());
    }
    // 2) Append arity when missing
    if !raw_name.contains('/') {
        let want = format!("{}/{}", raw_name, argc);
        if keys.iter().any(|k| k == &want) {
            return Some(want);
        }
    }
    // 3..5) Tail-based
    if let Some((class_or_alias, method)) = raw_name.split_once('.') {
        let want_tail = format!(".{}{}", method, format!("/{}", argc));
        let mut cands: Vec<String> = keys
            .iter()
            .filter(|k| k.ends_with(&want_tail)
                && (k.starts_with(&format!("{}.", class_or_alias))
                    || k.starts_with(&format!("{}_", class_or_alias))))
            .cloned()
            .collect();
        if !cands.is_empty() {
            cands.sort();
            return cands.into_iter().next();
        }
        // Heuristic: alias-prefixed static box pattern — Alias_Alias.method/arity
        let alias_alias = format!("{}_{}.{}{}", class_or_alias, class_or_alias, method, format!("/{}", argc));
        if keys.iter().any(|k| k == &alias_alias) {
            return Some(alias_alias);
        }
        // Final: any key that ends with .method/arity
        let any_tail = format!(".{}{}", method, format!("/{}", argc));
        if let Some(pick) = keys.iter().find(|k| k.ends_with(&any_tail)).cloned() {
            return Some(pick);
        }
    }
    None
}

