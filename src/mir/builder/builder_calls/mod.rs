// Module declaration for builder_calls
// Extracted call-related builders from builder.rs to keep files lean

// Re-export CallTarget for use in parent module
pub use super::calls::call_target::CallTarget;

// Submodules - these implement methods on MirBuilder
mod helpers;
mod special;
mod emit;
mod build;
mod lowering;

// All implementations are in the submodules:
// - helpers.rs: annotate_call_result_from_func_name, parse_type_name_to_mir, extract_string_literal, build_from_expression
// - special.rs: try_handle_math_function, try_handle_env_method, try_handle_me_direct_call
// - emit.rs: emit_unified_call, emit_legacy_call, emit_global_call, emit_method_call, emit_constructor_call
// - build.rs: resolve_call_target, build_function_call, build_method_call
// - lowering.rs: lower_method_as_function, lower_static_method_as_function