/*!
 * Provider Verify (skeleton)
 *
 * Phase 15.5 受け口: 起動時に最小の必須メソッドを検証するための軽量フック。
 * 既定はOFF。環境変数で warn/strict に切替える。
 *
 * Env:
 * - NYASH_PROVIDER_VERIFY=warn|strict
 * - NYASH_VERIFY_REQUIRED_METHODS="StringBox:length,concat;ArrayBox:push,get"
 *   (optional; merged with nyash.toml definitions when present)
 *
 * nyash.toml (optional; merged when present):
 * - [verify.required_methods]
 *     StringBox = ["length","concat"]
 *     ArrayBox  = ["push","get"]
 *   or
 * - [verify.required_methods.StringBox]
 *     methods = ["length","concat"]
 * - [types.StringBox]
 *     required_methods = ["length","concat"]
 */

use std::collections::HashMap;
// diagnostics are emitted via runtime::diagnostics; direct import unused here

fn parse_required_methods(spec: &str) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();
    for part in spec.split(';') {
        let p = part.trim();
        if p.is_empty() { continue; }
        if let Some((ty, rest)) = p.split_once(':') {
            let methods: Vec<String> = rest
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            if !methods.is_empty() {
                map.insert(ty.trim().to_string(), methods);
            }
        }
    }
    map
}

fn load_required_methods_from_toml() -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    let text = match std::fs::read_to_string("nyash.toml") { Ok(s) => s, Err(_) => return map };
    let doc: toml::Value = match toml::from_str(&text) { Ok(v) => v, Err(_) => return map };

    // Helper: add entry if array-of-strings
    let mut add_arr = |ty: &str, arr: &toml::Value| {
        if let Some(a) = arr.as_array() {
            let mut v: Vec<String> = Vec::new();
            for e in a { if let Some(s) = e.as_str() { let s = s.trim(); if !s.is_empty() { v.push(s.to_string()); } } }
            if !v.is_empty() { map.insert(ty.to_string(), v); }
        }
    };

    // 1) [verify.required_methods]
    if let Some(vrfy) = doc.get("verify") {
        if let Some(req) = vrfy.get("required_methods") {
            if let Some(tbl) = req.as_table() {
                for (k, v) in tbl.iter() {
                    if v.is_array() { add_arr(k, v); continue; }
                    if let Some(t) = v.as_table() { if let Some(m) = t.get("methods") { add_arr(k, m); } }
                }
            }
        }
    }

    // 2) [types.<TypeName>].required_methods
    if let Some(types) = doc.get("types") {
        if let Some(tbl) = types.as_table() {
            for (k, v) in tbl.iter() {
                if let Some(t) = v.as_table() { if let Some(m) = t.get("required_methods") { add_arr(k, m); } }
            }
        }
    }

    map
}

pub fn verify_from_env() -> Result<(), String> {
    let mode = crate::runtime::env_gate_box::string_alias_or("NYASH_PROVIDER_VERIFY","HAKO_PROVIDER_VERIFY","");
    let mode = mode.to_ascii_lowercase();
    if !(mode == "warn" || mode == "strict") { return Ok(()); }

    // Merge: nyash.toml + env override
    let mut req = load_required_methods_from_toml();
    let spec = crate::runtime::env_gate_box::string_alias_or("NYASH_VERIFY_REQUIRED_METHODS","HAKO_VERIFY_REQUIRED_METHODS","");
    if !spec.trim().is_empty() {
        let env_map = parse_required_methods(&spec);
        for (k, v) in env_map { req.entry(k).or_default().extend(v); }
    }
    if req.is_empty() { return Ok(()); }

    let mut failures: Vec<String> = Vec::new();
    for (ty, methods) in req.iter() {
        for m in methods {
            match crate::runtime::method_registry_box::resolve_method_handle(ty, m) {
                Ok(_) => { /* ok */ }
                Err(_e) => failures.push(format!("{}.{}", ty, m)),
            }
        }
    }

    if failures.is_empty() { return Ok(()); }
    if mode == "warn" {
        crate::runtime::diagnostics::trace_event(
            "provider_verify_warn",
            &format!("\"missing\":{}", serde_json::to_string(&failures).unwrap_or("[]".to_string())),
        );
        Ok(())
    } else {
        let msg = format!(
            "Provider verify failed (strict): missing methods: {}",
            failures.join(", ")
        );
        Err(msg)
    }
}
