/*!
 * WASM Lowering Passes
 *
 * Backend-specific MIR transformations for WASM execution.
 *
 * Note: This module adapts upstream ideas to the unified MirCall design.
 * It provides optional helpers (not wired by default) that can be used by
 * the WASM codegen to simplify handling of BoxCall-heavy MIR.
 */

pub mod boxcall_elimination;

pub use boxcall_elimination::BoxCallEliminator;

