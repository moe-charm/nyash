//! [modules] section processing: overrides, aliases, and options

use std::collections::HashMap;

/// Process [modules.overrides] section
pub fn process_overrides(
    overrides_tbl: &toml::Table,
    pending_modules: &mut Vec<(String, String)>,
) {
    let mut v = crate::common::using_core::flatten_modules_table(overrides_tbl);
    pending_modules.append(&mut v);
}

/// Process [modules.aliases] section (DEPRECATED)
pub fn process_aliases(
    aliases_tbl: &toml::Table,
    aliases: &mut HashMap<String, String>,
) {
    if !aliases_tbl.is_empty() && !crate::config::env::cli_quiet() {
        eprintln!("[deprecate] [modules.aliases] is deprecated; use [modules.overrides] instead");
    }

    for (k, v) in aliases_tbl.iter() {
        if let Some(target) = v.as_str() {
            aliases.insert(k.to_string(), target.to_string());
        }
    }
}

/// Process [modules.options] section: discovery and exclude settings
pub fn process_options(options_tbl: &toml::Table) {
    // enable_discovery
    if let Some(b) = options_tbl
        .get("enable_discovery")
        .and_then(|v| v.as_bool())
    {
        std::env::set_var("NYASH_DISCOVER_MODULES", if b { "1" } else { "0" });
    }

    // roots (array of strings)
    if let Some(arr) = options_tbl.get("roots").and_then(|v| v.as_array()) {
        let mut roots: Vec<String> = Vec::new();
        for e in arr.iter() {
            if let Some(s) = e.as_str() {
                let s = s.trim();
                if !s.is_empty() {
                    roots.push(s.to_string());
                }
            }
        }
        if !roots.is_empty() {
            std::env::set_var("NYASH_DISCOVER_ROOTS", roots.join(":"));
        }
    }

    // exclude array（ヒューリスティック: キーワードでON）
    if let Some(arr) = options_tbl.get("exclude").and_then(|v| v.as_array()) {
        let mut ex_archive = None;
        let mut ex_uscore = None;
        let mut ex_test = None;
        let mut ex_example = None;

        for e in arr.iter() {
            if let Some(s) = e.as_str() {
                let t = s.to_ascii_lowercase();
                if t.contains("archive") {
                    ex_archive = Some(true);
                }
                if t.contains("_*/") || t.contains("/_*") {
                    ex_uscore = Some(true);
                }
                if t.contains("test_") {
                    ex_test = Some(true);
                }
                if t.contains("example_") {
                    ex_example = Some(true);
                }
            }
        }

        if let Some(b) = ex_archive {
            std::env::set_var("NYASH_DISCOVER_EXCLUDE_ARCHIVE", if b { "1" } else { "0" });
        }
        if let Some(b) = ex_uscore {
            std::env::set_var(
                "NYASH_DISCOVER_EXCLUDE_UNDERSCORE_DIRS",
                if b { "1" } else { "0" },
            );
        }
        if let Some(b) = ex_test {
            std::env::set_var("NYASH_DISCOVER_EXCLUDE_TEST_PREFIX", if b { "1" } else { "0" });
        }
        if let Some(b) = ex_example {
            std::env::set_var(
                "NYASH_DISCOVER_EXCLUDE_EXAMPLE_PREFIX",
                if b { "1" } else { "0" },
            );
        }
    }

    // boolean 個別指定も許可（exclude_*）
    if let Some(b) = options_tbl
        .get("exclude_archive")
        .and_then(|v| v.as_bool())
    {
        std::env::set_var("NYASH_DISCOVER_EXCLUDE_ARCHIVE", if b { "1" } else { "0" });
    }
    if let Some(b) = options_tbl
        .get("exclude_underscore_dirs")
        .and_then(|v| v.as_bool())
    {
        std::env::set_var(
            "NYASH_DISCOVER_EXCLUDE_UNDERSCORE_DIRS",
            if b { "1" } else { "0" },
        );
    }
    if let Some(b) = options_tbl
        .get("exclude_test_prefix")
        .and_then(|v| v.as_bool())
    {
        std::env::set_var("NYASH_DISCOVER_EXCLUDE_TEST_PREFIX", if b { "1" } else { "0" });
    }
    if let Some(b) = options_tbl
        .get("exclude_example_prefix")
        .and_then(|v| v.as_bool())
    {
        std::env::set_var(
            "NYASH_DISCOVER_EXCLUDE_EXAMPLE_PREFIX",
            if b { "1" } else { "0" },
        );
    }
}
