
use crate::mir::{MirFunction, ValueId, MirType, MirInstruction, BasicBlockId};
use std::collections::HashMap;
use crate::ast::ASTNode;
use super::MirBuilder;

// PHI-based return type inference helper
pub(super) fn infer_type_from_phi(
    function: &MirFunction,
    ret_val: ValueId,
    types: &HashMap<ValueId, MirType>,
) -> Option<MirType> {
    for (_bid, bb) in function.blocks.iter() {
        for inst in bb.instructions.iter() {
            if let MirInstruction::Phi { dst, inputs } = inst {
                if *dst == ret_val {
                    let mut it = inputs.iter().filter_map(|(_, v)| types.get(v));
                    if let Some(first) = it.next() {
                        if it.all(|mt| mt == first) {
                            return Some(first.clone());
                        }
                    }
                }
            }
        }
    }
    None
}

// Local helper for if-statement analysis (moved from stmts.rs)
pub(super) fn extract_assigned_var(ast: &ASTNode) -> Option<String> {
    match ast {
        ASTNode::Assignment { target, .. } => {
            if let ASTNode::Variable { name, .. } = target.as_ref() { Some(name.clone()) } else { None }
        }
        ASTNode::Program { statements, .. } => statements.last().and_then(|st| extract_assigned_var(st)),
        ASTNode::If { then_body, else_body, .. } => {
            // Look into nested if: if both sides assign the same variable, propagate that name upward.
            let then_prog = ASTNode::Program { statements: then_body.clone(), span: crate::ast::Span::unknown() };
            let tvar = extract_assigned_var(&then_prog);
            let evar = else_body.as_ref().and_then(|eb| {
                let ep = ASTNode::Program { statements: eb.clone(), span: crate::ast::Span::unknown() };
                extract_assigned_var(&ep)
            });
            match (tvar, evar) {
                (Some(tv), Some(ev)) if tv == ev => Some(tv),
                _ => None,
            }
        }
        _ => None,
    }
}

impl MirBuilder {
    /// Normalize Phi creation for if/else constructs.
    /// This handles variable reassignment patterns and ensures a single exit value.
    pub(super) fn normalize_if_else_phi(
        &mut self,
        then_block: BasicBlockId,
        else_block: BasicBlockId,
        then_value_raw: ValueId,
        else_value_raw: ValueId,
        pre_if_var_map: &HashMap<String, ValueId>,
        then_ast_for_analysis: &ASTNode,
        else_ast_for_analysis: &Option<ASTNode>,
        then_var_map_end: &HashMap<String, ValueId>,
        else_var_map_end_opt: &Option<HashMap<String, ValueId>>,
        pre_then_var_value: Option<ValueId>,
    ) -> Result<ValueId, String> {
        // If only the then-branch assigns a variable (e.g., `if c { x = ... }`) and the else
        // does not assign the same variable, bind that variable to a Phi of (then_value, pre_if_value).
        let assigned_var_then = extract_assigned_var(then_ast_for_analysis);
        let assigned_var_else = else_ast_for_analysis.as_ref().and_then(|a| extract_assigned_var(a));
        let result_val = self.value_gen.next();

        if let Some(var_name) = assigned_var_then.clone() {
            let else_assigns_same = assigned_var_else.as_ref().map(|s| s == &var_name).unwrap_or(false);
            // Resolve branch-end values for the assigned variable
            let then_value_for_var = then_var_map_end.get(&var_name).copied().unwrap_or(then_value_raw);
            let else_value_for_var = if else_assigns_same {
                else_var_map_end_opt.as_ref().and_then(|m| m.get(&var_name).copied()).unwrap_or(else_value_raw)
            } else {
                // Else doesn't assign: use pre-if value if available
                pre_then_var_value.unwrap_or(else_value_raw)
            };
            // Emit Phi for the assigned variable and bind it
            self.emit_instruction(MirInstruction::Phi { dst: result_val, inputs: vec![(then_block, then_value_for_var), (else_block, else_value_for_var)] })?;
            self.variable_map = pre_if_var_map.clone();
            self.variable_map.insert(var_name, result_val);
        } else {
            // No variable assignment pattern detected – just emit Phi for expression result
            self.emit_instruction(MirInstruction::Phi { dst: result_val, inputs: vec![(then_block, then_value_raw), (else_block, else_value_raw)] })?;
            // Merge variable map conservatively to pre-if snapshot (no new bindings)
            self.variable_map = pre_if_var_map.clone();
        }

        Ok(result_val)
    }
}
