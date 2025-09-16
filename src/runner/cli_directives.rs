/*!
 * CLI Directives Scanner — early source comments to env plumbing
 *
 * Supports lightweight, file-scoped directives placed in the first lines
 * of a Nyash source file. Current directives:
 *  - // @env KEY=VALUE         → export KEY=VALUE into process env
 *  - // @plugin-builtins      → NYASH_USE_PLUGIN_BUILTINS=1
 *  - // @jit-debug            → enable common JIT debug flags (no-op if JIT unused)
 *  - // @jit-strict           → strict JIT flags (no VM fallback) for experiments
 *
 * Also runs the "fields-at-top" lint delegated to pipeline::lint_fields_top.
 */

pub(super) fn apply_cli_directives_from_source(
    code: &str,
    strict_fields: bool,
    verbose: bool,
) -> Result<(), String> {
    // Scan only the header area (up to the first non-comment content line)
    for (i, line) in code.lines().take(128).enumerate() {
        let l = line.trim();
        if !(l.starts_with("//") || l.starts_with("#!") || l.is_empty()) {
            if i > 0 {
                break;
            }
        }
        if let Some(rest) = l.strip_prefix("//") {
            let rest = rest.trim();
            if let Some(dir) = rest.strip_prefix("@env ") {
                if let Some((k, v)) = dir.split_once('=') {
                    let key = k.trim();
                    let val = v.trim();
                    if !key.is_empty() {
                        std::env::set_var(key, val);
                    }
                }
            } else if rest == "@plugin-builtins" {
                std::env::set_var("NYASH_USE_PLUGIN_BUILTINS", "1");
            } else if rest == "@jit-debug" {
                // Safe even if JIT is disabled elsewhere; treated as no-op flags
                std::env::set_var("NYASH_JIT_EXEC", "1");
                std::env::set_var("NYASH_JIT_THRESHOLD", "1");
                std::env::set_var("NYASH_JIT_EVENTS", "1");
                std::env::set_var("NYASH_JIT_EVENTS_COMPILE", "1");
                std::env::set_var("NYASH_JIT_EVENTS_RUNTIME", "1");
                std::env::set_var("NYASH_JIT_SHIM_TRACE", "1");
            } else if rest == "@jit-strict" {
                std::env::set_var("NYASH_JIT_STRICT", "1");
                std::env::set_var("NYASH_JIT_ARGS_HANDLE_ONLY", "1");
                if std::env::var("NYASH_JIT_ONLY").ok().is_none() {
                    std::env::set_var("NYASH_JIT_ONLY", "1");
                }
            }
        }
    }

    // Lint: enforce fields at top-of-box (delegated)
    super::pipeline::lint_fields_top(code, strict_fields, verbose)
}
