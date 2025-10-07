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

/// Normalize a call name to fully qualified "Class.method/Arity".
/// - If already fully qualified, return as-is.
/// - If "Class.method" without arity, append "/Arity".
/// - Otherwise (only method name), return Err.
pub fn normalize(raw_name: &str, argc: usize) -> Result<String, String> {
    if is_fully_qualified(raw_name) { return Ok(raw_name.to_string()); }
    if raw_name.contains('.') { return Ok(format!("{}/{}", raw_name, argc)); }
    Err(format!("CallNameResolver: class name required: {}", raw_name))
}

/// Parse a fully qualified name into (Class, method, arity).
pub fn parse(full: &str) -> Result<(String, String, usize), String> {
    if let Some((class_method, arity_str)) = full.rsplit_once('/') {
        if let Some((class, method)) = class_method.split_once('.') {
            if let Ok(arity) = arity_str.parse::<usize>() {
                return Ok((class.to_string(), method.to_string(), arity));
            }
        }
    }
    Err(format!("Invalid call name: {}", full))
}

/// Return true if name is fully qualified (contains both '.' and '/').
pub fn is_fully_qualified(name: &str) -> bool { name.contains('.') && name.contains('/') }


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn policy_and_core_tail_resolution_align() {
        // Available keys
        let keys = vec![
            "A.m/2".to_string(),
            "B.m/2".to_string(),
            "Alias_Alias.m/2".to_string(),
            "C.n/1".to_string(),
        ];
        // Core resolver
        let pick = resolve_module_function(keys.clone(), "A.m", 2).unwrap();
        // Policy candidates (class_or_alias="A") must include A.m/2 first
        let cands = crate::common::call_policy::tail_candidates(keys.iter(), "A", "m", 2, false);
        assert_eq!(pick, cands[0]);

        // Alias_Alias heuristic
        let pick2 = resolve_module_function(keys.clone(), "Alias.m", 2).unwrap();
        assert_eq!(pick2, "Alias_Alias.m/2");
        let cands2 = crate::common::call_policy::tail_candidates(keys.iter(), "Alias", "m", 2, false);
        assert!(cands2.first().map(|s| s.as_str()) == Some("Alias_Alias.m/2"));
    }
}
