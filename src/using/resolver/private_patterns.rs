//! [private] section processing: patterns and options for access control

/// Process [private] section: record patterns and options into environment variables
/// for pipeline enforcement
pub fn process_private_section(private_tbl: &toml::Table) {
    // patterns array
    if let Some(arr) = private_tbl.get("patterns").and_then(|v| v.as_array()) {
        let mut pats: Vec<String> = Vec::new();

        for e in arr.iter() {
            if let Some(s) = e.as_str() {
                let t = s.trim();
                if !t.is_empty() {
                    pats.push(t.to_string());
                }
            }
        }

        if !pats.is_empty() {
            std::env::set_var("NYASH_PRIVATE_PATTERNS", pats.join(";"));
        }
    }

    // options subtable
    if let Some(opts) = private_tbl.get("options").and_then(|v| v.as_table()) {
        if let Some(mode) = opts.get("on_violation").and_then(|v| v.as_str()) {
            std::env::set_var("NYASH_PRIVATE_ON_VIOLATION", mode);
        }
        if let Some(diag) = opts
            .get("enable_diagnostics")
            .and_then(|v| v.as_bool())
        {
            std::env::set_var("NYASH_PRIVATE_DIAG", if diag { "1" } else { "0" });
        }
    }
}
