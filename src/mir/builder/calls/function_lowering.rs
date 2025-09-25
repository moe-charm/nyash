/*!
 * Function Lowering Utilities
 *
 * Helpers for lowering box methods and static methods to MIR functions
 * Manages the complex state transitions during function lowering
 */

use crate::ast::ASTNode;
use crate::mir::{Effect, EffectMask, FunctionSignature, MirType};
use super::special_handlers::contains_value_return;

/// Prepare function signature for a box method
/// Includes 'me' parameter as first parameter
pub fn prepare_method_signature(
    func_name: String,
    box_name: &str,
    params: &[String],
    body: &[ASTNode],
) -> FunctionSignature {
    let mut param_types = Vec::new();

    // First parameter is always 'me' (the box instance)
    param_types.push(MirType::Box(box_name.to_string()));

    // Additional parameters (type unknown initially)
    for _ in params {
        param_types.push(MirType::Unknown);
    }

    // Determine return type based on body analysis
    let returns_value = contains_value_return(body);
    let ret_ty = if returns_value {
        MirType::Unknown  // Will be inferred later
    } else {
        MirType::Void
    };

    FunctionSignature {
        name: func_name,
        params: param_types,
        return_type: ret_ty,
        effects: EffectMask::READ.add(Effect::ReadHeap),
    }
}

/// Prepare function signature for a static method
/// No 'me' parameter needed
pub fn prepare_static_method_signature(
    func_name: String,
    params: &[String],
    body: &[ASTNode],
) -> FunctionSignature {
    let mut param_types = Vec::new();

    // Parameters (type unknown initially)
    for _ in params {
        param_types.push(MirType::Unknown);
    }

    // Determine return type based on body analysis
    let returns_value = contains_value_return(body);
    let ret_ty = if returns_value {
        MirType::Unknown  // Will be inferred later
    } else {
        MirType::Void
    };

    FunctionSignature {
        name: func_name,
        params: param_types,
        return_type: ret_ty,
        effects: EffectMask::READ.add(Effect::ReadHeap),
    }
}

/// Generate canonical method name for MIR function
/// E.g., "StringBox.upper/0" for StringBox's upper method with 0 args
pub fn generate_method_function_name(
    box_name: &str,
    method_name: &str,
    arity: usize,
) -> String {
    format!("{}.{}/{}", box_name, method_name, arity)
}

/// Generate canonical static method name for MIR function
/// E.g., "Main.main/0" for static box Main's main method
pub fn generate_static_method_function_name(
    static_box_name: &str,
    method_name: &str,
    arity: usize,
) -> String {
    format!("{}.{}/{}", static_box_name, method_name, arity)
}

/// Check if a function needs termination with void return
pub fn needs_void_termination(returns_value: bool, is_terminated: bool) -> bool {
    !returns_value && !is_terminated
}

/// Create parameter mapping for method lowering
/// Returns (parameter_names, includes_me)
pub fn create_method_parameter_mapping(
    box_name: &str,
    params: &[String],
) -> (Vec<(String, MirType)>, bool) {
    let mut param_mapping = Vec::new();

    // Add 'me' parameter
    param_mapping.push(("me".to_string(), MirType::Box(box_name.to_string())));

    // Add regular parameters
    for p in params {
        param_mapping.push((p.clone(), MirType::Unknown));
    }

    (param_mapping, true)
}

/// Create parameter mapping for static method lowering
pub fn create_static_parameter_mapping(
    params: &[String],
) -> Vec<(String, MirType)> {
    params.iter()
        .map(|p| (p.clone(), MirType::Unknown))
        .collect()
}

/// Wrap statements in a Program node for consistent processing
pub fn wrap_in_program(statements: Vec<ASTNode>) -> ASTNode {
    ASTNode::Program {
        statements,
        span: crate::ast::Span::unknown(),
    }
}

/// Check if method name suggests it returns a value
pub fn method_likely_returns_value(method_name: &str) -> bool {
    // Heuristic: methods that likely return values
    method_name.starts_with("get") ||
    method_name.starts_with("is") ||
    method_name.starts_with("has") ||
    method_name.starts_with("to") ||
    matches!(method_name,
        "length" | "size" | "count" |
        "upper" | "lower" | "trim" |
        "add" | "sub" | "mul" | "div" |
        "min" | "max" | "abs"
    )
}