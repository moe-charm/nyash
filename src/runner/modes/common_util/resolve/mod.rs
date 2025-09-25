/*!
 * Using resolver utilities (split)
 * - strip: remove `using` lines, inline modules, register aliases/modules
 * - seam: seam logging and optional brace-fix at join points
 */

pub mod strip;
pub mod seam;

// Public re-exports to preserve existing call sites
pub use strip::{strip_using_and_register, preexpand_at_local, collect_using_and_strip, resolve_prelude_paths_profiled};
