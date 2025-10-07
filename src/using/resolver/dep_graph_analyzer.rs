//! Dependency graph analysis: cycle detection and version checking

use crate::using::errors::UsingError;
use std::collections::HashMap;

/// Analyze dependency graph: detect cycles and check version compatibility
///
/// Performs two checks:
/// 1. Cycle detection - detects circular dependencies between modules
/// 2. Version checking - validates dependencies are present and compatible
///
/// In strict mode (NYASH_USING_CHECKS_STRICT=1), returns error on first violation
/// Otherwise, emits warnings and continues
pub fn analyze_dependencies(
    manifests: &[crate::config::module_workspace::ModuleManifest],
    versions: &HashMap<String, String>,
) -> Result<(), UsingError> {
    // Cycle detection (warn/strict)
    let g = crate::config::module_workspace::build_dep_graph(manifests);
    let cycles = crate::config::module_workspace::detect_cycles_from_graph(&g);

    if !cycles.is_empty() {
        for cyc in cycles.iter() {
            eprintln!("{}", crate::common::diagnostics::modules_error::cycle(cyc));
            if !crate::config::env::cli_quiet() {
                eprintln!("[deps] cycle: {}", cyc.join(" -> "));
            }
        }

        let strict = std::env::var("NYASH_USING_CHECKS_STRICT")
            .ok()
            .as_deref()
            == Some("1");

        if strict {
            let joined = cycles
                .get(0)
                .map(|c| c.join(" -> "))
                .unwrap_or_else(|| "cycle".to_string());
            return Err(UsingError::Cycle(joined));
        }
    }

    // Missing dependency/version checks (warn/strict)
    for man in manifests.iter() {
        let base_ns = man.name.clone();

        for (dep, req) in man.dependencies.iter() {
            if !versions.contains_key(dep) {
                eprintln!(
                    "{}",
                    crate::common::diagnostics::modules_error::missing_dep(&base_ns, dep, req)
                );
                if !crate::config::env::cli_quiet() {
                    eprintln!("[deps] missing: {} -> {} {}", base_ns, dep, req);
                }

                if std::env::var("NYASH_USING_CHECKS_STRICT")
                    .ok()
                    .as_deref()
                    == Some("1")
                {
                    return Err(UsingError::MissingDep {
                        module: base_ns.clone(),
                        dep: dep.clone(),
                        req: req.clone(),
                    });
                }
            } else if let Some(cur) = versions.get(dep) {
                // Version compatibility check (simplified: only caret requirements)
                if req.starts_with('^') {
                    let want_major = req.trim_start_matches('^').split('.').next().unwrap_or("");
                    let have_major = cur.split('.').next().unwrap_or("");

                    if want_major != have_major {
                        eprintln!(
                            "{}",
                            crate::common::diagnostics::modules_error::missing_dep(
                                &base_ns, dep, req
                            )
                        );
                        if !crate::config::env::cli_quiet() {
                            eprintln!(
                                "[deps] version-mismatch: {} requires {} {} but found {}",
                                base_ns, dep, req, cur
                            );
                        }

                        if std::env::var("NYASH_USING_CHECKS_STRICT")
                            .ok()
                            .as_deref()
                            == Some("1")
                        {
                            return Err(UsingError::MissingDep {
                                module: base_ns.clone(),
                                dep: dep.clone(),
                                req: req.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
