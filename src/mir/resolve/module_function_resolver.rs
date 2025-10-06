//! ModuleFunctionResolverBox — strict resolver with optional tail fallback
//!
//! Policy:
//! - Dotted names (Class.method[/N]) resolve strictly by exact match (with arity).
//! - If arity missing, append `/argc` and retry.
//! - Optional tail fallback is allowed only when explicitly enabled by caller.
//! - Bare global names (no dot) delegate to the shared core resolver.

/// Resolve a function name against a set of known function keys.
/// `raw_name` may be "Class.method" or "Class.method/N" or bare.
pub fn resolve_strict<I: IntoIterator<Item = String>>(
    keys_iter: I,
    raw_name: &str,
    argc: usize,
    allow_tail: bool,
) -> Option<String> {
    let keys: Vec<String> = keys_iter.into_iter().collect();
    let dotted = raw_name.contains('.');
    if dotted {
        // Canonicalize to include arity if missing
        let want = if raw_name.contains('/') {
            raw_name.to_string()
        } else {
            format!("{}/{}", raw_name, argc)
        };
        if keys.iter().any(|k| k == &want) {
            return Some(want);
        }
        if !allow_tail { return None; }
        // Tail fallback: only candidates with matching class prefix or alias_alias
        if let Some((class, method)) = raw_name.split_once('.') {
            let tail = format!(".{}{}", method, format!("/{}", argc));
            let mut cands: Vec<String> = keys
                .iter()
                .filter(|k| k.ends_with(&tail)
                    && (k.starts_with(&format!("{}.", class)) || k.starts_with(&format!("{}_", class))))
                .cloned()
                .collect();
            if !cands.is_empty() {
                cands.sort();
                return cands.into_iter().next();
            }
            // Alias_Alias.method/arity
            let alias_alias = format!("{}_{}.{}{}", class, class, method, format!("/{}", argc));
            if keys.iter().any(|k| k == &alias_alias) {
                return Some(alias_alias);
            }
        }
        None
    } else {
        // Delegate to core for legacy bare names
        crate::mir::resolve::call_resolver_core::resolve_module_function(keys.into_iter(), raw_name, argc)
    }
}

