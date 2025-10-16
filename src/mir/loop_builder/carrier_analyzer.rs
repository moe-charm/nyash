/*!
 * Loop Carrier Variable Analyzer
 *
 * Identifies true loop-carried variables using intersection analysis:
 *   loop_carried = (variables in preheader) ∩ (variables assigned in body)
 *
 * This box encapsulates the core logic for PHI node generation in loops,
 * ensuring only variables that carry values across iterations receive PHI nodes.
 */

use crate::ast::ASTNode;
use crate::mir::ValueId;
use std::collections::{HashMap, HashSet};

/// Box for analyzing loop-carried variables.
///
/// Implements the "Everything is Box" principle by providing a single-responsibility,
/// pure-function interface for loop carrier analysis.
pub struct LoopCarrierAnalyzerBox;

impl LoopCarrierAnalyzerBox {
    /// Analyze which variables are true loop-carried variables.
    ///
    /// # Algorithm
    /// 1. Collect all variables assigned within the loop body
    /// 2. Filter to only those defined in the preheader (intersection)
    /// 3. Return the intersection as a HashSet
    ///
    /// # Parameters
    /// - `preheader_vars`: Variable map from the preheader block (before loop entry)
    /// - `body`: AST nodes representing the loop body
    ///
    /// # Returns
    /// HashSet of variable names that are true loop-carried variables
    ///
    /// # Example
    /// ```rust,ignore
    /// let preheader_vars = HashMap::from([
    ///     ("i".to_string(), ValueId(0)),
    ///     ("acc".to_string(), ValueId(1)),
    ///     ("s".to_string(), ValueId(2)),  // parameter, not assigned in loop
    /// ]);
    /// let body = vec![/* AST nodes */];
    /// let carriers = LoopCarrierAnalyzerBox::analyze(&preheader_vars, &body);
    /// // carriers = {"i", "acc"}  (not "s" or "ch")
    /// ```
    pub fn analyze(
        preheader_vars: &HashMap<String, ValueId>,
        body: &[ASTNode],
    ) -> HashSet<String> {
        // Step 1: Collect all variables assigned in loop body
        let assigned_vars = Self::collect_assigned_vars(body);

        // Step 2: Compute intersection (preheader ∩ assigned)
        assigned_vars
            .into_iter()
            .filter(|v| preheader_vars.contains_key(v))
            .collect()
    }

    /// Collect all variables that are assigned within the loop body.
    ///
    /// Uses the existing `collect_carrier_assigns` utility from phi_core::loop_phi.
    ///
    /// # Parameters
    /// - `body`: AST nodes representing the loop body
    ///
    /// # Returns
    /// Vector of variable names assigned in the body
    fn collect_assigned_vars(body: &[ASTNode]) -> Vec<String> {
        let mut vars = Vec::new();
        let mut has_ctrl = false;

        for stmt in body {
            crate::mir::phi_core::loop_phi::collect_carrier_assigns(
                stmt,
                &mut vars,
                &mut has_ctrl,
            );
        }

        vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ASTNode, Span, BinaryOperator, LiteralValue};
    use crate::mir::ValueId;

    #[test]
    fn test_analyze_simple_loop() {
        // Preheader defines: i=0, acc=0, s=param
        let mut preheader_vars = HashMap::new();
        preheader_vars.insert("i".to_string(), ValueId(0));
        preheader_vars.insert("acc".to_string(), ValueId(1));
        preheader_vars.insert("s".to_string(), ValueId(2)); // parameter

        // Body assigns: i, acc (not s or ch)
        let body = vec![
            // i = i + 1
            ASTNode::Assignment {
                target: Box::new(ASTNode::Variable {
                    name: "i".to_string(),
                    span: Span::unknown(),
                }),
                value: Box::new(ASTNode::BinaryOp {
                    operator: BinaryOperator::Add,
                    left: Box::new(ASTNode::Variable {
                        name: "i".to_string(),
                        span: Span::unknown(),
                    }),
                    right: Box::new(ASTNode::Literal {
                        value: LiteralValue::Integer(1),
                        span: Span::unknown(),
                    }),
                    span: Span::unknown(),
                }),
                span: Span::unknown(),
            },
            // acc = acc + 1
            ASTNode::Assignment {
                target: Box::new(ASTNode::Variable {
                    name: "acc".to_string(),
                    span: Span::unknown(),
                }),
                value: Box::new(ASTNode::BinaryOp {
                    operator: BinaryOperator::Add,
                    left: Box::new(ASTNode::Variable {
                        name: "acc".to_string(),
                        span: Span::unknown(),
                    }),
                    right: Box::new(ASTNode::Literal {
                        value: LiteralValue::Integer(1),
                        span: Span::unknown(),
                    }),
                    span: Span::unknown(),
                }),
                span: Span::unknown(),
            },
        ];

        let carriers = LoopCarrierAnalyzerBox::analyze(&preheader_vars, &body);

        // Should contain i and acc, but NOT s (not assigned)
        assert!(carriers.contains("i"));
        assert!(carriers.contains("acc"));
        assert!(!carriers.contains("s"));
        assert_eq!(carriers.len(), 2);
    }

    #[test]
    fn test_analyze_no_assignments() {
        let mut preheader_vars = HashMap::new();
        preheader_vars.insert("x".to_string(), ValueId(0));

        let body = vec![]; // empty body

        let carriers = LoopCarrierAnalyzerBox::analyze(&preheader_vars, &body);

        // No assignments = no carriers
        assert!(carriers.is_empty());
    }

    #[test]
    fn test_analyze_assignment_not_in_preheader() {
        let mut preheader_vars = HashMap::new();
        preheader_vars.insert("i".to_string(), ValueId(0));

        // Body assigns a variable NOT in preheader
        let body = vec![ASTNode::Assignment {
            target: Box::new(ASTNode::Variable {
                name: "ch".to_string(), // NEW variable in loop body
                span: Span::unknown(),
            }),
            value: Box::new(ASTNode::Literal {
                value: LiteralValue::String("x".to_string()),
                span: Span::unknown(),
            }),
            span: Span::unknown(),
        }];

        let carriers = LoopCarrierAnalyzerBox::analyze(&preheader_vars, &body);

        // ch is assigned but NOT in preheader, so NOT a carrier
        assert!(!carriers.contains("ch"));
        assert!(carriers.is_empty());
    }
}
