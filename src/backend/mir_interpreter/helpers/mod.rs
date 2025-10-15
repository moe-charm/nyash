/*!
 * MirInterpreter helper functions split into focused modules.
 */

mod eval;
mod guards;
mod materialize;
mod trace;
mod utils;
pub mod lifecycle_contracts_box;

// Re-export nothing (all functions are pub(in crate::backend::mir_interpreter) on MirInterpreter impl blocks)
