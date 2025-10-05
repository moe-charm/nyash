/*!
 * Using resolver utilities — static resolution line (SSOT + AST)
 *
 * Separation of concerns:
 * - Static (using-time): Resolve packages/aliases from nyash.toml (SSOT),
 *   strip `using` lines, collect prelude file paths, and (when enabled)
 *   parse/merge them as AST before macro expansion.
 * - Dynamic (runtime): Plugin/extern dispatch only. User instance BoxCall
 *   fallback is disallowed in prod; builder must rewrite obj.method() to
 *   a function call.
 *
 * Modules:
 * - strip: profile-aware resolution (`collect_using_and_strip`,
 *   `resolve_prelude_paths_profiled`) — single entrypoints used by all
 *   runner modes to avoid drift.
 * - seam: seam logging and optional boundary markers (for diagnostics).
 */

pub mod strip;
pub mod seam;
pub mod alias_tools;
pub mod register;
pub mod alias_expand;

// Public re-exports to preserve existing call sites
pub use strip::{
    preexpand_at_local,
    collect_using_and_strip,
    resolve_prelude_paths_profiled,
    parse_preludes_to_asts,
    merge_prelude_asts_with_main,
};

// Re-export registration helpers
pub use register::register_aliases_in_modules_registry;
