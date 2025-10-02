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

/// Wrap statements in a Program node for consistent processing
pub fn wrap_in_program(statements: Vec<ASTNode>) -> ASTNode {
    ASTNode::Program {
        statements,
        span: crate::ast::Span::unknown(),
    }
}

