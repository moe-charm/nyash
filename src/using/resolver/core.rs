use crate::using::errors::UsingError;
use crate::using::policy::UsingPolicy;
use crate::using::spec::UsingPackage;
use std::collections::HashMap;

/// Populate using context vectors from configuration file (hako.toml preferred; compat nyash.toml/hakorune.toml).
/// Keeps behavior aligned with existing runner pipeline:
///  - Adds [using.paths] entries to `using_paths`
///  - Flattens [modules] into (name, path) pairs appended to `pending_modules`
///  - Reads optional [aliases] table (k -> v)
pub fn populate_from_toml(
    using_paths: &mut Vec<String>,
    pending_modules: &mut Vec<(String, String)>,
    aliases: &mut HashMap<String, String>,
    packages: &mut HashMap<String, UsingPackage>,
) -> Result<UsingPolicy, UsingError> {
    let mut policy = UsingPolicy::default();

    // Locate configuration file
    let chosen = super::config_locator::locate_config_file();
    let path = chosen
        .as_ref()
        .map(|p| p.as_path())
        .unwrap_or_else(|| std::path::Path::new("nyash.toml"));

    // Emit trace information
    super::config_locator::trace_config_location(path);

    // Early return if file doesn't exist
    if !path.exists() {
        return Ok(policy);
    }

    // Load and parse TOML
    let doc = super::toml_parser::load_and_parse_toml(path)?;

    // [modules] table flatten: supports nested namespaces (a.b.c = "path")
    if let Some(mods) = doc.get("modules").and_then(|v| v.as_table()) {
        // main table: no direct flatten (use workspace/overrides for clarity)
        // [modules.workspace] — load module.toml manifests and export public entries
        if let Some(ws_tbl) = mods.get("workspace").and_then(|v| v.as_table()) {
            let (ws_manifests, ws_versions, mut ws_modules) =
                super::workspace_loader::load_workspace_members(ws_tbl, path);

            // Append workspace modules
            pending_modules.append(&mut ws_modules);

            // Analyze dependency graph (cycle detection + version checks)
            super::dep_graph_analyzer::analyze_dependencies(&ws_manifests, &ws_versions)?;
        }
        // [modules.overrides] takes precedence (append; later wins in map usage)
        if let Some(ovr) = mods.get("overrides").and_then(|v| v.as_table()) {
            super::modules_processor::process_overrides(ovr, pending_modules);
        }

        // [modules.aliases] (DEPRECATED)
        if let Some(alias_tbl) = mods.get("aliases").and_then(|v| v.as_table()) {
            super::modules_processor::process_aliases(alias_tbl, aliases);
        }

        // [modules.options] → discovery/env オプション
        if let Some(opts_tbl) = mods.get("options").and_then(|v| v.as_table()) {
            super::modules_processor::process_options(opts_tbl);
        }

        // Namespace conflict detection across accumulated pending_modules (warn/strict)
        super::conflict_detector::detect_conflicts_from_modules(pending_modules)?;
    }

    // [using] section processing
    if let Some(using_tbl) = doc.get("using").and_then(|v| v.as_table()) {
        // paths
        if let Some(paths_arr) = using_tbl.get("paths").and_then(|v| v.as_array()) {
            super::using_processor::process_paths(paths_arr, using_paths, &mut policy);
        }

        // aliases
        if let Some(alias_tbl) = using_tbl.get("aliases").and_then(|v| v.as_table()) {
            super::using_processor::process_aliases(alias_tbl, aliases);
        }

        // named packages
        super::using_processor::process_packages(using_tbl, packages);
    }

    // legacy top-level [aliases] also accepted (migration)
    if let Some(alias_tbl) = doc.get("aliases").and_then(|v| v.as_table()) {
        super::using_processor::process_legacy_aliases(alias_tbl, aliases);
    }

    // [private] section processing
    if let Some(priv_tbl) = doc.get("private").and_then(|v| v.as_table()) {
        super::private_patterns::process_private_section(priv_tbl);
    }

    // Directory-as-Namespace fallback (optional; env-gated)
    super::dir_namespace_discovery::discover_and_append_if_enabled(pending_modules)?;

    // Final duplicate detection across all accumulated entries (always on; strict -> error)
    super::conflict_detector::detect_conflicts_from_modules(pending_modules)?;
    Ok(policy)
}


#[cfg(test)]
mod tests {

    #[test]
    fn e2e_workspace_modules_exports() {
        let dir = std::env::temp_dir().join(format!("nyash_workspace_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
        std::fs::create_dir_all(&dir).unwrap();
        // Layout: apps/selfhost/vm/module.toml + boxes/mini_vm_entry.hako
        let mod_dir = dir.join("apps/selfhost/vm");
        std::fs::create_dir_all(mod_dir.join("boxes")).unwrap();
        std::fs::write(mod_dir.join("boxes/mini_vm_entry.hako"), "// stub").unwrap();
        let module_toml = r#"[module]
name = "selfhost.vm"
version = "1.0.0"

[exports]
entry = "boxes/mini_vm_entry.hako"
"#;
        std::fs::write(mod_dir.join("module.toml"), module_toml).unwrap();
        // Write hako.toml with workspace members
        let toml_text = r#"[modules.workspace]
members = ["apps/selfhost/vm/module.toml"]
"#;
        std::fs::write(dir.join("hako.toml"), toml_text).unwrap();
        std::env::set_var("NYASH_USING_TEST_FORCE_ENV_ROOT", "1");
        std::env::set_var("NYASH_ROOT", dir.to_str().unwrap());
        let mut using_paths = Vec::new();
        let mut pending_modules = Vec::new();
        let mut aliases = std::collections::HashMap::new();
        let mut packages = std::collections::HashMap::new();
        let _policy = super::populate_from_toml(&mut using_paths, &mut pending_modules, &mut aliases, &mut packages).unwrap();
        pending_modules.sort();
        assert!(pending_modules.iter().any(|(ns, p)| ns == "selfhost.vm.entry" && p.ends_with("apps/selfhost/vm/boxes/mini_vm_entry.hako")));
    }

    use super::*;
    use std::fs;
    #[test]
    fn e2e_flatten_modules_from_runner_env_root() {
        // Create temp dir
        let dir = std::env::temp_dir().join(format!("nyash_using_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
        fs::create_dir_all(&dir).unwrap();
        // Write nyash.toml with nested modules
        let toml_text = r#"
[modules.a.b]
c = "x/path"
d = "y/path"
[modules.e]
f = "z/path"
"#;
        fs::write(dir.join("nyash.toml"), toml_text).unwrap();
        // Force resolver to use env root
        std::env::set_var("NYASH_USING_TEST_FORCE_ENV_ROOT", "1");
        std::env::set_var("NYASH_ROOT", dir.to_str().unwrap());
        let mut using_paths = Vec::new();
        let mut pending_modules = Vec::new();
        let mut aliases = std::collections::HashMap::new();
        let mut packages = std::collections::HashMap::new();
        let _policy = super::populate_from_toml(&mut using_paths, &mut pending_modules, &mut aliases, &mut packages).unwrap();
        pending_modules.sort();
        assert_eq!(pending_modules, vec![
            ("a.b.c".into(), "x/path".into()),
            ("a.b.d".into(), "y/path".into()),
            ("e.f".into(), "z/path".into()),
        ]);
    }
}
