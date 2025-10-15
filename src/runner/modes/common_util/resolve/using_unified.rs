//! Unified using/alias resolution for all runner modes
//!
//! Consolidates the duplicated using resolution logic from vm.rs, llvm.rs,
//! vm_fallback.rs, and common.rs into a single reusable function.

use std::collections::{HashMap, HashSet};

/// Result of using resolution and prelude loading
pub struct UsingResolveResult {
    /// Cleaned code with `using` lines stripped
    pub cleaned_code: String,
    /// Parsed prelude ASTs (empty if AST merge disabled)
    pub prelude_asts: Vec<nyash_rust::ast::ASTNode>,
    /// Collected alias names from `using ... as Alias`
    pub alias_names: HashSet<String>,
}

/// Options for using resolution
#[derive(Default)]
pub struct UsingResolveOptions {
    /// Skip AST merge without error (for quiet pipe mode)
    pub allow_skip_ast_merge: bool,
    /// Collect alias names into result (needed for vm.rs)
    pub collect_alias_names: bool,
}

/// Unified using/alias resolution with prelude loading
///
/// This function consolidates the common pattern found in vm.rs, llvm.rs,
/// vm_fallback.rs, and common.rs:
/// 1. Call resolve_prelude_paths_profiled() to strip using lines
/// 2. Register aliases in modules registry
/// 3. Optionally parse and merge prelude ASTs
/// 4. Handle alias renaming with collision guards
///
/// Returns error message on failure (for consistent error handling)
pub fn resolve_using_with_preludes(
    runner: &crate::runner::NyashRunner,
    code: &str,
    filename: &str,
    options: UsingResolveOptions,
) -> Result<UsingResolveResult, String> {
    let use_ast = crate::config::env::using_ast_enabled();
    let mut alias_names = HashSet::new();
    let mut prelude_asts = Vec::new();

    // Step 1: Resolve using paths and strip using lines
    let (cleaned_code, paths, alias_pairs) =
        super::resolve_prelude_paths_profiled(runner, code, filename)
            .map_err(|e| format!("using resolution error: {}", e))?;
    if crate::config::env::resolve_trace() && !crate::config::env::cli_quiet() {
        eprintln!("[using/trace] preludes: {} alias_pairs: {}", paths.len(), alias_pairs.len());
        if !paths.is_empty() {
            for p in paths.iter() { eprintln!("[using/trace] prelude: {}", p); }
        }
    }
    // Self-check: modules resolved (alias_pairs>0) but no paths collected → likely AST merge or file-using gate
    if alias_pairs.len() > 0 && paths.is_empty() {
        if !crate::config::env::cli_quiet() {
            eprintln!("[using/selfcheck] modules resolved but no prelude paths collected (alias_pairs={}, paths=0)", alias_pairs.len());
            let ast_on = crate::config::env::using_ast_enabled();
            let allow_file = crate::config::env::allow_using_file();
            if !ast_on {
                eprintln!("[using/selfcheck] AST merge is disabled. Enable with HAKO_USING=full or NYASH_USING_AST=1");
            }
            if !allow_file {
                eprintln!("[using/selfcheck] File-using is disabled. Enable with HAKO_USING=full or HAKO_ALLOW_USING_FILE=1");
            }
        }
    }

    // Step 2: Register aliases in modules registry (always done)
    super::register_aliases_in_modules_registry(&alias_pairs);

    // Step 3: Collect alias names if requested
    if options.collect_alias_names {
        for (alias, _canon) in alias_pairs.iter() {
            alias_names.insert(alias.clone());
        }
    }

    // Step 4: Handle AST prelude merge if enabled
    if !paths.is_empty() {
        if !use_ast {
            if !options.allow_skip_ast_merge {
                return Err(
                    "AST prelude merge is disabled. Enable NYASH_USING_AST=1 or remove 'using' lines."
                        .to_string(),
                );
            }
            // Quiet pipe mode: skip AST merge without error
        } else {
            // Parse preludes to ASTs
            let parsed = super::parse_preludes_to_asts(runner, &paths)
                .map_err(|e| format!("prelude parsing error: {}", e))?;
            if crate::config::env::resolve_trace() && !crate::config::env::cli_quiet() {
                eprintln!("[using/trace] parsed preludes: {}", parsed.len());
            }

            // Build alias map: canonical_path -> alias
            let mut alias_map: HashMap<String, String> = HashMap::new();
            for (alias, canon) in alias_pairs {
                alias_map.insert(canon, alias);
            }

            // Rename ASTs with collision guards
            let mut used_prefixed: HashSet<String> = HashSet::new();
            for (path, ast) in parsed {
                let canon = std::fs::canonicalize(&path)
                    .ok()
                    .map(|pb| pb.to_string_lossy().to_string())
                    .unwrap_or(path.clone());

                let renamed_ast = if let Some(alias) = alias_map.get(&canon) {
                    super::alias_tools::rename_with_collision_guard(
                        &ast,
                        alias,
                        &mut used_prefixed,
                        &canon,
                    )
                    .map_err(|e| format!("alias renaming error: {}", e))?
                } else {
                    ast
                };

                prelude_asts.push(renamed_ast);
            }
        }
    }

    Ok(UsingResolveResult {
        cleaned_code,
        prelude_asts,
        alias_names,
    })
}
