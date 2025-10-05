// CallResolver — VM wrapper around shared MIR resolver with optional trace

fn maybe_trace(raw: &str, argc: usize, pick: Option<&str>) {
    if std::env::var("NYASH_VM_RESOLVE_TRACE").ok().as_deref() == Some("1") {
        let picked = pick.unwrap_or("");
        eprintln!("{{\"resolve\":{{\"raw\":\"{}\",\"argc\":{},\"pick\":\"{}\"}}}}", raw, argc, picked);
    }
}

/// Resolve a ModuleFunction name from a raw name and argc against a set of known function keys.
/// Strategy:
/// 1) Exact match
/// 2) If no arity suffix, append `/argc` and try again
/// 3) Tail-based fallback: match `.method/arity` and same class prefix (`Class.method/arity`)
/// Wrapper over shared core with VM trace
pub fn resolve_module_function_collect(keys_iter: impl IntoIterator<Item = String>, raw_name: &str, argc: usize) -> Option<String> {
    let pick = crate::mir::resolve::call_resolver_core::resolve_module_function(keys_iter, raw_name, argc);
    maybe_trace(raw_name, argc, pick.as_deref());
    pick
}

#[cfg(test)]
mod tests {
    use super::resolve_module_function_collect;

    #[test]
    fn exact_match() {
        let keys = vec![
            "Main.main/0".to_string(),
            "JsonNode.create_object/0".to_string(),
        ];
        let r = resolve_module_function_collect(keys.clone().into_iter(), "JsonNode.create_object/0", 0);
        assert_eq!(r.as_deref(), Some("JsonNode.create_object/0"));
    }

    #[test]
    fn append_arity_when_missing() {
        let keys = vec![
            "Helper.make/2".to_string(),
        ];
        let r = resolve_module_function_collect(keys.clone().into_iter(), "Helper.make", 2);
        assert_eq!(r.as_deref(), Some("Helper.make/2"));
    }

    #[test]
    fn tail_based_fallback() {
        let keys = vec![
            "JsonNode.create_object/0".to_string(),
            "JsonNode.parse/1".to_string(),
        ];
        let r = resolve_module_function_collect(keys.clone().into_iter(), "JsonNode.create_object", 0);
        assert_eq!(r.as_deref(), Some("JsonNode.create_object/0"));
        let r2 = resolve_module_function_collect(keys.clone().into_iter(), "JsonNode.parse", 1);
        assert_eq!(r2.as_deref(), Some("JsonNode.parse/1"));
    }
}
