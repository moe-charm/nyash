/*!
 * Call Resolution Utilities - Legacy re-exports
 *
 * ⚠️ DEPRECATED: This module re-exports from calls::method_resolution
 * New code should import directly from calls::method_resolution
 *
 * ChatGPT5 Pro Design: Maintaining backward compatibility during refactoring
 */

// Re-export canonical implementations from method_resolution
pub use super::calls::method_resolution::{
    is_builtin_function,
    is_extern_function,
    is_commonly_shadowed_method,
    suggest_resolution,
    generate_self_recursion_warning,
};

// Tests are maintained in calls::method_resolution module