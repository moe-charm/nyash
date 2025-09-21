/*!
 * Macro child mode (split modules)
 */

mod transforms;
mod entry;

pub use entry::run_macro_child;
pub use transforms::normalize_core_pass;

