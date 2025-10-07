//! Call resolution policy helpers (shared across VM/builder where possible).

/// Return the base name (without trailing "/arity") of a fully qualified name.
pub fn base_without_arity(name: &str) -> &str {
    name.rsplit_once('/')
        .map(|(b, _)| b)
        .unwrap_or(name)
}

/// Detect immediate self-cycle between a candidate pick and the requested name.
/// Compares base names without arity.
pub fn is_immediate_cycle(pick: &str, name: &str) -> bool {
    base_without_arity(pick) == base_without_arity(name)
}

/// Build tail-match candidates for a method/arity.
///
/// - `keys`: available function keys (fully-qualified: "Class.method/Arity")
/// - `class_or_alias`: the class qualifier (or alias) from the requested name
/// - `method_only`: method name without arity ("birth", "run_min", ...)
/// - `arity`: expected arity from the callsite
/// - `wide`: when true, accept any class prefix; when false, require prefix to
///   match either `<class_or_alias>.` or `<class_or_alias>_` (alias-alias form)
pub fn tail_candidates<'a, I>(
    keys: I,
    class_or_alias: &str,
    method_only: &str,
    arity: usize,
    wide: bool,
) -> Vec<String>
where
    I: IntoIterator<Item = &'a String>,
{
    let tail = format!(".{}{}", method_only, format!("/{}", arity));
    let prefix1 = format!("{}.", class_or_alias);
    let prefix2 = format!("{}_", class_or_alias);
    let mut res: Vec<String> = Vec::new();
    for k in keys.into_iter() {
        if k.ends_with(&tail) {
            if wide || k.starts_with(&prefix1) || k.starts_with(&prefix2) {
                res.push(k.clone());
            }
        }
    }
    res.sort();
    res
}
