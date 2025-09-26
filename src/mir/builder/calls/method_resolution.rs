/*!
 * Method Resolution System
 *
 * Type-safe function and method resolution at compile-time
 * ChatGPT5 Pro design for preventing runtime string-based resolution
 */

use crate::mir::{Callee, ValueId};
use std::collections::HashMap;

/// Resolve function call target to type-safe Callee
/// Implements the core logic of compile-time function resolution
pub fn resolve_call_target(
    name: &str,
    current_static_box: &Option<String>,
    variable_map: &HashMap<String, ValueId>,
) -> Result<Callee, String> {
    // 1. Check for built-in/global functions first
    if is_builtin_function(name) {
        return Ok(Callee::Global(name.to_string()));
    }

    // 2. Check for static box method in current context
    if let Some(box_name) = current_static_box {
        if has_method(box_name, name) {
            // Warn about potential self-recursion
            if is_commonly_shadowed_method(name) {
                eprintln!("{}", generate_self_recursion_warning(box_name, name));
            }

            return Ok(Callee::Method {
                box_name: box_name.clone(),
                method: name.to_string(),
                receiver: None, // Static method call
            });
        }
    }

    // 3. Check for local variable containing function value
    if let Some(&value_id) = variable_map.get(name) {
        return Ok(Callee::Value(value_id));
    }

    // 4. Check for external/host functions
    if is_extern_function(name) {
        return Ok(Callee::Extern(name.to_string()));
    }

    // 5. Do not assume bare `name()` refers to current static box.
    //    Leave it unresolved so caller can try static_method_index fallback
    //    or report a clear unresolved error.

    // 6. Resolution failed - prevent runtime string-based resolution
    Err(format!("Unresolved function: '{}'. {}", name, suggest_resolution(name)))
}

/// Check if function name is a built-in global function
pub fn is_builtin_function(name: &str) -> bool {
    matches!(name,
        "print" | "error" | "panic" | "exit" | "now" |
        "gc_collect" | "gc_stats" |
        // Math functions (handled specially)
        "sin" | "cos" | "abs" | "min" | "max"
    )
}

/// Check if function name is an external/host function
pub fn is_extern_function(name: &str) -> bool {
    name.starts_with("nyash.") ||
    name.starts_with("env.") ||
    name.starts_with("system.")
}

/// Check if method is commonly shadowed (for warning generation)
pub fn is_commonly_shadowed_method(name: &str) -> bool {
    matches!(name, "print" | "log" | "error" | "toString")
}

/// Generate warning about potential self-recursion
pub fn generate_self_recursion_warning(box_name: &str, method: &str) -> String {
    format!(
        "[Warning] Calling '{}' in static box '{}' context. \
         This resolves to '{}.{}' which may cause self-recursion if called from within the same method.",
        method, box_name, box_name, method
    )
}

/// Suggest resolution for unresolved function
pub fn suggest_resolution(name: &str) -> String {
    match name {
        n if n.starts_with("console") => {
            "Did you mean 'env.console.log' or 'print'?".to_string()
        }
        "log" | "println" => {
            "Did you mean 'print' or 'env.console.log'?".to_string()
        }
        n if n.contains('.') => {
            "Qualified names should use 'env.' prefix for external calls.".to_string()
        }
        _ => {
            "Check function name or ensure it's in scope.".to_string()
        }
    }
}

/// Check if current static box has the specified method
/// TODO: Replace with proper method registry lookup
pub fn has_method(box_name: &str, method: &str) -> bool {
    match box_name {
        "ConsoleStd" => matches!(method, "print" | "println" | "log"),
        "StringBox" => matches!(method, "upper" | "lower" | "length" | "concat" | "slice"),
        "IntegerBox" => matches!(method, "add" | "sub" | "mul" | "div"),
        "ArrayBox" => matches!(method, "push" | "pop" | "get" | "set" | "length"),
        "MapBox" => matches!(method, "get" | "set" | "has" | "delete"),
        "MathBox" => matches!(method, "sin" | "cos" | "abs" | "min" | "max"),
        _ => false, // Conservative: assume no method unless explicitly known
    }
}
