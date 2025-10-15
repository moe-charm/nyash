//! Workspace module manifest loading for using resolver

use std::collections::HashMap;
use std::path::Path;

/// Load workspace member manifests and extract namespace->path mappings
///
/// Returns:
/// - Vec<ModuleManifest>: Loaded manifests
/// - HashMap<String, String>: Module name -> version mapping
/// - Vec<(String, String)>: Additional (namespace, path) pairs to append
pub fn load_workspace_members(
    ws_tbl: &toml::Table,
    config_path: &Path,
) -> (
    Vec<crate::config::module_workspace::ModuleManifest>,
    HashMap<String, String>,
    Vec<(String, String)>,
) {
    let mut ws_manifests = Vec::new();
    let mut ws_versions = HashMap::new();
    let mut additional_modules = Vec::new();

    let Some(arr) = ws_tbl.get("members").and_then(|v| v.as_array()) else {
        return (ws_manifests, ws_versions, additional_modules);
    };

    for m in arr.iter() {
        let Some(raw) = m.as_str() else { continue };

        // Accept a manifest path or a directory; expand simple patterns like apps/*/hako_module.toml
        let mut cand_paths: Vec<std::path::PathBuf> = Vec::new();

        if raw.contains('*') || raw.contains('?') {
            // Resolve patterns relative to the config file directory when not absolute
            let expanded = crate::config::module_workspace::expand_members_pattern(raw);
            if let Some(conf_dir) = config_path.parent() {
                for p in expanded.into_iter() {
                    let pp = if p.is_absolute() {
                        p
                    } else {
                        conf_dir.join(p)
                    };
                    cand_paths.push(pp);
                }
            } else {
                cand_paths.extend(expanded);
            }
        } else {
            let p_rel = std::path::Path::new(raw);
            let p = if p_rel.is_absolute() {
                p_rel.to_path_buf()
            } else {
                config_path
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("."))
                    .join(p_rel)
            };

            if p.is_dir() {
                // Prefer hako_module.toml; accept legacy module.toml; also allow module.hako fallback
                cand_paths.push(p.join("hako_module.toml"));
                cand_paths.push(p.join("module.toml"));
                cand_paths.push(p.join("module.hako"));
            } else {
                cand_paths.push(p.to_path_buf());
            }
        }

        for mp in cand_paths.into_iter() {
            if crate::config::env::resolve_trace() && !crate::config::env::cli_quiet() {
                eprintln!("[using] ws cand: {:?}", mp);
            }

            if !mp.exists() {
                continue;
            }

            // Try TOML manifests first
            if let Some(man) = crate::config::module_workspace::parse_module_toml(&mp) {
                let base_ns = man.name.clone();
                if let Some(ver) = man.version.clone() {
                    ws_versions.insert(base_ns.clone(), ver);
                }

                let base_dir = mp.parent().unwrap_or_else(|| std::path::Path::new("."));
                for (key, rel) in man.exports.iter() {
                    let ns = format!("{}.{}", base_ns, key);
                    let sp = base_dir.join(rel);
                    if let Some(sp_s) = sp.to_str() {
                        additional_modules.push((ns, sp_s.to_string()));
                    }
                }

                ws_manifests.push(man);
            } else {
                // Fallback: module.hako (preview) — now wired for resolver
                let pdir = if mp.is_dir() {
                    mp.clone()
                } else {
                    mp.parent()
                        .unwrap_or_else(|| std::path::Path::new("."))
                        .to_path_buf()
                };

                let mh = if mp.file_name().and_then(|s| s.to_str()) == Some("module.hako") {
                    mp.clone()
                } else {
                    pdir.join("module.hako")
                };

                if mh.exists() {
                    if let Some(man) = crate::config::module_workspace::parse_module_hako(&mh) {
                        let base_ns = man.name.clone();
                        if let Some(ver) = man.version.clone() {
                            ws_versions.insert(base_ns.clone(), ver);
                        }

                        let base_dir = mh.parent().unwrap_or_else(|| std::path::Path::new("."));
                        for (key, rel) in man.exports.iter() {
                            let ns = format!("{}.{}", base_ns, key);
                            let sp = base_dir.join(rel);
                            if let Some(sp_s) = sp.to_str() {
                                additional_modules.push((ns, sp_s.to_string()));
                            }
                        }

                        ws_manifests.push(man);
                    }
                }
            }
        }
    }

    (ws_manifests, ws_versions, additional_modules)
}
