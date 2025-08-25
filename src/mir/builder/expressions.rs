/*!
 * MIR Builder Expressions - Expression AST node conversion
 * 
 * Handles conversion of expression AST nodes to MIR instructions
 */

use super::*;
use crate::ast::{ASTNode, LiteralValue, BinaryOperator};

/// Binary operation type classification
enum BinaryOpType {
    Arithmetic(BinaryOp),
    Comparison(CompareOp),
}

impl MirBuilder {
    /// Build a literal value into MIR
    pub(super) fn build_literal(&mut self, literal: LiteralValue) -> Result<ValueId, String> {
        let const_value = match literal {
            LiteralValue::Integer(n) => ConstValue::Integer(n),
            LiteralValue::Float(f) => ConstValue::Float(f),
            LiteralValue::String(s) => ConstValue::String(s),
            LiteralValue::Bool(b) => ConstValue::Bool(b),
            LiteralValue::Null => ConstValue::Null,
            LiteralValue::Void => ConstValue::Void,
        };
        
        let dst = self.value_gen.next();
        self.emit_instruction(MirInstruction::Const {
            dst,
            value: const_value,
        })?;
        
        Ok(dst)
    }

    /// Build a binary operation
    pub(super) fn build_binary_op(&mut self, left: ASTNode, operator: BinaryOperator, right: ASTNode) -> Result<ValueId, String> {
        let lhs = self.build_expression(left)?;
        let rhs = self.build_expression(right)?;
        let dst = self.value_gen.next();
        
        let mir_op = self.convert_binary_operator(operator)?;
        
        match mir_op {
            // Arithmetic operations
            BinaryOpType::Arithmetic(op) => {
                self.emit_instruction(MirInstruction::BinOp {
                    dst, op, lhs, rhs
                })?;
            },
            
            // Comparison operations
            BinaryOpType::Comparison(op) => {
                self.emit_instruction(MirInstruction::Compare {
                    dst, op, lhs, rhs
                })?;
            },
        }
        
        Ok(dst)
    }
    
    /// Build a unary operation
    pub(super) fn build_unary_op(&mut self, operator: String, operand: ASTNode) -> Result<ValueId, String> {
        let operand_val = self.build_expression(operand)?;
        let dst = self.value_gen.next();
        
        let mir_op = self.convert_unary_operator(operator)?;
        
        self.emit_instruction(MirInstruction::UnaryOp {
            dst,
            op: mir_op,
            operand: operand_val,
        })?;
        
        Ok(dst)
    }

    /// Convert AST binary operator to MIR operator
    fn convert_binary_operator(&self, op: BinaryOperator) -> Result<BinaryOpType, String> {
        match op {
            BinaryOperator::Add => Ok(BinaryOpType::Arithmetic(BinaryOp::Add)),
            BinaryOperator::Subtract => Ok(BinaryOpType::Arithmetic(BinaryOp::Sub)),
            BinaryOperator::Multiply => Ok(BinaryOpType::Arithmetic(BinaryOp::Mul)),
            BinaryOperator::Divide => Ok(BinaryOpType::Arithmetic(BinaryOp::Div)),
            BinaryOperator::Modulo => Ok(BinaryOpType::Arithmetic(BinaryOp::Mod)),
            BinaryOperator::Equal => Ok(BinaryOpType::Comparison(CompareOp::Eq)),
            BinaryOperator::NotEqual => Ok(BinaryOpType::Comparison(CompareOp::Ne)),
            BinaryOperator::Less => Ok(BinaryOpType::Comparison(CompareOp::Lt)),
            BinaryOperator::LessEqual => Ok(BinaryOpType::Comparison(CompareOp::Le)),
            BinaryOperator::Greater => Ok(BinaryOpType::Comparison(CompareOp::Gt)),
            BinaryOperator::GreaterEqual => Ok(BinaryOpType::Comparison(CompareOp::Ge)),
            BinaryOperator::And => Ok(BinaryOpType::Arithmetic(BinaryOp::And)),
            BinaryOperator::Or => Ok(BinaryOpType::Arithmetic(BinaryOp::Or)),
        }
    }
    
    /// Convert AST unary operator to MIR operator
    fn convert_unary_operator(&self, op: String) -> Result<UnaryOp, String> {
        match op.as_str() {
            "-" => Ok(UnaryOp::Neg),
            "!" | "not" => Ok(UnaryOp::Not),
            "~" => Ok(UnaryOp::BitNot),
            _ => Err(format!("Unsupported unary operator: {}", op)),
        }
    }

    // Placeholder - remaining expression methods will be moved from builder.rs
    pub(super) fn build_expression_placeholder(&mut self, _ast: ASTNode) -> Result<ValueId, String> {
        Err("Expression building not yet implemented in modular structure".to_string())
    }
}