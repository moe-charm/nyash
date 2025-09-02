use super::{MirInstruction, ValueId, MirType};
use crate::ast::{ASTNode, BinaryOperator};
use crate::mir::{BinaryOp, UnaryOp, CompareOp, TypeOpKind};

// Internal classification for binary operations
#[derive(Debug)]
enum BinaryOpType {
    Arithmetic(BinaryOp),
    Comparison(CompareOp),
}

impl super::MirBuilder {
    // Build a binary operation
    pub(super) fn build_binary_op(
        &mut self,
        left: ASTNode,
        operator: BinaryOperator,
        right: ASTNode,
    ) -> Result<ValueId, String> {
        let lhs = self.build_expression(left)?;
        let rhs = self.build_expression(right)?;
        let dst = self.value_gen.next();

        let mir_op = self.convert_binary_operator(operator)?;

        match mir_op {
            // Arithmetic operations
            BinaryOpType::Arithmetic(op) => {
                self.emit_instruction(MirInstruction::BinOp { dst, op, lhs, rhs })?;
                // Arithmetic results are integers for now (Core-1)
                self.value_types.insert(dst, MirType::Integer);
            }
            // Comparison operations
            BinaryOpType::Comparison(op) => {
                // 80/20: If both operands originate from IntegerBox, cast to integer first
                let (lhs2, rhs2) = if self
                    .value_origin_newbox
                    .get(&lhs)
                    .map(|s| s == "IntegerBox")
                    .unwrap_or(false)
                    && self
                        .value_origin_newbox
                        .get(&rhs)
                        .map(|s| s == "IntegerBox")
                        .unwrap_or(false)
                {
                    let li = self.value_gen.next();
                    let ri = self.value_gen.next();
                    self.emit_instruction(MirInstruction::TypeOp {
                        dst: li,
                        op: TypeOpKind::Cast,
                        value: lhs,
                        ty: MirType::Integer,
                    })?;
                    self.emit_instruction(MirInstruction::TypeOp {
                        dst: ri,
                        op: TypeOpKind::Cast,
                        value: rhs,
                        ty: MirType::Integer,
                    })?;
                    (li, ri)
                } else {
                    (lhs, rhs)
                };
                self.emit_instruction(MirInstruction::Compare { dst, op, lhs: lhs2, rhs: rhs2 })?;
                self.value_types.insert(dst, MirType::Bool);
            }
        }

        Ok(dst)
    }

    // Build a unary operation
    pub(super) fn build_unary_op(&mut self, operator: String, operand: ASTNode) -> Result<ValueId, String> {
        let operand_val = self.build_expression(operand)?;
        let dst = self.value_gen.next();

        let mir_op = self.convert_unary_operator(operator)?;

        self.emit_instruction(MirInstruction::UnaryOp { dst, op: mir_op, operand: operand_val })?;
        Ok(dst)
    }

    // Convert AST binary operator to MIR enum or compare
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

    // Convert AST unary operator to MIR operator
    fn convert_unary_operator(&self, op: String) -> Result<UnaryOp, String> {
        match op.as_str() {
            "-" => Ok(UnaryOp::Neg),
            "!" | "not" => Ok(UnaryOp::Not),
            "~" => Ok(UnaryOp::BitNot),
            _ => Err(format!("Unsupported unary operator: {}", op)),
        }
    }
}
