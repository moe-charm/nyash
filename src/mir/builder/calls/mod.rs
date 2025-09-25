/*!
 * Call System Module Organization
 *
 * Refactored from monolithic builder_calls.rs (879 lines)
 * Split into focused modules following Single Responsibility Principle
 */

// Core types
pub mod call_target;
pub use call_target::CallTarget;

// Resolution system
pub mod method_resolution;

// External calls
pub mod extern_calls;

// Special handlers
pub mod special_handlers;

// Function lowering
pub mod function_lowering;

// Unified call system
pub mod call_unified;

// Re-export commonly used items
pub use method_resolution::{
    resolve_call_target,
    is_builtin_function,
    is_extern_function,
    has_method,
};

pub use extern_calls::{
    get_env_method_spec,
    parse_extern_name,
    is_env_interface,
    compute_extern_effects,
};

pub use special_handlers::{
    is_math_function,
    is_typeop_method,
    extract_string_literal,
    parse_type_name_to_mir,
    contains_value_return,
    make_function_name_with_arity,
};

pub use function_lowering::{
    prepare_method_signature,
    prepare_static_method_signature,
    generate_method_function_name,
    generate_static_method_function_name,
    wrap_in_program,
};

pub use call_unified::{
    is_unified_call_enabled,
    convert_target_to_callee,
    compute_call_effects,
    create_call_flags,
    create_mir_call,
    validate_call_args,
};