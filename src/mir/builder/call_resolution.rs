/*!
 * Call Resolution Utilities - Type-safe function call helpers
 *
 * ChatGPT5 Pro Design: Stateless helpers for compile-time function resolution
 * These utilities can be used across different parts of the compiler pipeline
 */

/// Check if function name is a built-in global function
/// These functions are resolved at compile-time to Callee::Global
pub fn is_builtin_function(name: &str) -> bool {
    matches!(
        name,
        // Core runtime functions
        "print" | "error" | "panic" | "exit" | "now" |
        // Type operation functions
        "isType" | "asType" |
        // Math functions (may be expanded)
        "abs" | "min" | "max"
    )
}

/// Check if function name is an external/host function
/// These functions are resolved to Callee::Extern and handled by runtime
pub fn is_extern_function(name: &str) -> bool {
    name.starts_with("nyash.") // Host functions use nyash.* namespace
}

/// Get suggested resolution for unresolved function names
/// Provides helpful error messages for common mistakes
pub fn suggest_resolution(name: &str) -> String {
    match name {
        "print" | "error" | "panic" | "exit" => {
            format!("Consider using ::{}() for global function or check if you're in a box with a {} method", name, name)
        }
        name if name.starts_with("str") || name.starts_with("string") => {
            "Consider using StringBox methods or string.* functions".to_string()
        }
        name if name.starts_with("array") || name.starts_with("arr") => {
            "Consider using ArrayBox methods or array.* functions".to_string()
        }
        _ => {
            format!("Function '{}' not found. Check spelling or add explicit scope qualifier", name)
        }
    }
}

/// Check if a method name is commonly shadowed by global functions
/// Used for generating warnings about potential self-recursion
pub fn is_commonly_shadowed_method(method: &str) -> bool {
    matches!(
        method,
        "print" | "error" | "log" | "panic" | // Console methods
        "length" | "size" | "count" |         // Container methods
        "toString" | "valueOf" |              // Conversion methods
        "equals" | "compare"                  // Comparison methods
    )
}

/// Generate warning message for potential self-recursion
pub fn generate_self_recursion_warning(box_name: &str, method: &str) -> String {
    format!(
        "Warning: Potential self-recursion detected in {}.{}(). \
         Consider using ::{}() for global function or {}.{}() for explicit self-call.",
        box_name, method, method, box_name, method
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_function_detection() {
        assert!(is_builtin_function("print"));
        assert!(is_builtin_function("error"));
        assert!(is_builtin_function("panic"));
        assert!(is_builtin_function("isType"));
        assert!(!is_builtin_function("custom_function"));
        assert!(!is_builtin_function("nyash.console.log"));
    }

    #[test]
    fn test_extern_function_detection() {
        assert!(is_extern_function("nyash.console.log"));
        assert!(is_extern_function("nyash.fs.read"));
        assert!(!is_extern_function("print"));
        assert!(!is_extern_function("custom_function"));
    }

    #[test]
    fn test_shadowed_method_detection() {
        assert!(is_commonly_shadowed_method("print"));
        assert!(is_commonly_shadowed_method("length"));
        assert!(is_commonly_shadowed_method("toString"));
        assert!(!is_commonly_shadowed_method("custom_method"));
    }

    #[test]
    fn test_warning_generation() {
        let warning = generate_self_recursion_warning("ConsoleStd", "print");
        assert!(warning.contains("ConsoleStd.print()"));
        assert!(warning.contains("::print()"));
        assert!(warning.contains("self-recursion"));
    }
}