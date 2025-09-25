use super::{MirInstruction, MirType, ValueId};
use crate::mir::loop_api::LoopBuilderApi; // for current_block()
use crate::ast::{ASTNode, BinaryOperator};
use crate::mir::{BinaryOp, CompareOp, TypeOpKind, UnaryOp};

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
        // Short-circuit logical ops: lower to control-flow so RHS is evaluated conditionally
        if matches!(operator, BinaryOperator::And | BinaryOperator::Or) {
            return self.build_logical_shortcircuit(left, operator, right);
        }

        let lhs = self.build_expression(left)?;
        let rhs = self.build_expression(right)?;
        let dst = self.value_gen.next();

        let mir_op = self.convert_binary_operator(operator)?;

        match mir_op {
            // Arithmetic operations
            BinaryOpType::Arithmetic(op) => {
                self.emit_instruction(MirInstruction::BinOp { dst, op, lhs, rhs })?;
                // '+' は文字列連結の可能性がある。オペランドが String/StringBox なら結果を String と注釈。
                if matches!(op, crate::mir::BinaryOp::Add) {
                    let lhs_is_str = match self.value_types.get(&lhs) {
                        Some(MirType::String) => true,
                        Some(MirType::Box(bt)) if bt == "StringBox" => true,
                        _ => false,
                    };
                    let rhs_is_str = match self.value_types.get(&rhs) {
                        Some(MirType::String) => true,
                        Some(MirType::Box(bt)) if bt == "StringBox" => true,
                        _ => false,
                    };
                    if lhs_is_str || rhs_is_str {
                        self.value_types.insert(dst, MirType::String);
                    } else {
                        self.value_types.insert(dst, MirType::Integer);
                    }
                } else {
                    // その他の算術は整数
                    self.value_types.insert(dst, MirType::Integer);
                }
            }
            // Comparison operations
            BinaryOpType::Comparison(op) => {
                // 80/20: If both operands originate from IntegerBox, cast to integer first
                let (lhs2_raw, rhs2_raw) = if self
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
                // Materialize operands in the current block to avoid dominance/undef issues
                let lhs2 = lhs2_raw;
                let rhs2 = rhs2_raw;
                self.emit_instruction(MirInstruction::Compare {
                    dst,
                    op,
                    lhs: lhs2,
                    rhs: rhs2,
                })?;
                self.value_types.insert(dst, MirType::Bool);
            }
        }

        Ok(dst)
    }

    /// Lower logical && / || with proper short-circuit semantics.
    /// Result is a Bool, and RHS is only evaluated if needed.
    fn build_logical_shortcircuit(
        &mut self,
        left: ASTNode,
        operator: BinaryOperator,
        right: ASTNode,
    ) -> Result<ValueId, String> {
        let is_and = matches!(operator, BinaryOperator::And);

        // Evaluate LHS only once and pin to a slot so it can be reused safely across blocks
        let lhs_val0 = self.build_expression(left)?;
        let lhs_val = self.pin_to_slot(lhs_val0, "@sc_lhs")?;

        // Prepare blocks
        let then_block = self.block_gen.next();
        let else_block = self.block_gen.next();
        let merge_block = self.block_gen.next();

        // Branch on LHS truthiness (runtime to_bool semantics in interpreter/LLVM)
        self.emit_instruction(MirInstruction::Branch {
            condition: lhs_val,
            then_bb: then_block,
            else_bb: else_block,
        })?;

        // Snapshot variables before entering branches
        let pre_if_var_map = self.variable_map.clone();

        // ---- THEN branch ----
        self.current_block = Some(then_block);
        self.ensure_block_exists(then_block)?;
        self.hint_scope_enter(0);
        // Reset scope to pre-if snapshot for clean deltas
        self.variable_map = pre_if_var_map.clone();

        // AND: then → evaluate RHS and reduce to bool
        //  OR: then → constant true
        let then_value_raw = if is_and {
            // Reduce arbitrary RHS to bool by branching on its truthiness and returning consts
            let rhs_true = self.block_gen.next();
            let rhs_false = self.block_gen.next();
            let rhs_join = self.block_gen.next();
            let rhs_val = self.build_expression(right.clone())?;
            self.emit_instruction(MirInstruction::Branch {
                condition: rhs_val,
                then_bb: rhs_true,
                else_bb: rhs_false,
            })?;
            // true path
            self.start_new_block(rhs_true)?;
            let t_id = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const { dst: t_id, value: crate::mir::ConstValue::Bool(true) })?;
            self.emit_instruction(MirInstruction::Jump { target: rhs_join })?;
            let rhs_true_exit = self.current_block()?;
            // false path
            self.start_new_block(rhs_false)?;
            let f_id = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const { dst: f_id, value: crate::mir::ConstValue::Bool(false) })?;
            self.emit_instruction(MirInstruction::Jump { target: rhs_join })?;
            let rhs_false_exit = self.current_block()?;
            // join rhs result into a single bool
            self.start_new_block(rhs_join)?;
            let rhs_bool = self.value_gen.next();
            let inputs = vec![(rhs_true_exit, t_id), (rhs_false_exit, f_id)];
            if let (Some(func), Some(cur_bb)) = (&self.current_function, self.current_block) {
                crate::mir::phi_core::common::debug_verify_phi_inputs(func, cur_bb, &inputs);
            }
            self.emit_instruction(MirInstruction::Phi { dst: rhs_bool, inputs })?;
            self.value_types.insert(rhs_bool, MirType::Bool);
            rhs_bool
        } else {
            let t_id = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const { dst: t_id, value: crate::mir::ConstValue::Bool(true) })?;
            t_id
        };
        let then_exit_block = self.current_block()?;
        let then_var_map_end = self.variable_map.clone();
        if !self.is_current_block_terminated() {
            self.hint_scope_leave(0);
            self.emit_instruction(MirInstruction::Jump { target: merge_block })?;
        }

        // ---- ELSE branch ----
        self.current_block = Some(else_block);
        self.ensure_block_exists(else_block)?;
        self.hint_scope_enter(0);
        self.variable_map = pre_if_var_map.clone();
        // AND: else → false
        //  OR: else → evaluate RHS and reduce to bool
        let else_value_raw = if is_and {
            let f_id = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const { dst: f_id, value: crate::mir::ConstValue::Bool(false) })?;
            f_id
        } else {
            let rhs_true = self.block_gen.next();
            let rhs_false = self.block_gen.next();
            let rhs_join = self.block_gen.next();
            let rhs_val = self.build_expression(right)?;
            self.emit_instruction(MirInstruction::Branch {
                condition: rhs_val,
                then_bb: rhs_true,
                else_bb: rhs_false,
            })?;
            // true path
            self.start_new_block(rhs_true)?;
            let t_id = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const { dst: t_id, value: crate::mir::ConstValue::Bool(true) })?;
            self.emit_instruction(MirInstruction::Jump { target: rhs_join })?;
            let rhs_true_exit = self.current_block()?;
            // false path
            self.start_new_block(rhs_false)?;
            let f_id = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const { dst: f_id, value: crate::mir::ConstValue::Bool(false) })?;
            self.emit_instruction(MirInstruction::Jump { target: rhs_join })?;
            let rhs_false_exit = self.current_block()?;
            // join rhs result into a single bool
            self.start_new_block(rhs_join)?;
            let rhs_bool = self.value_gen.next();
            let inputs = vec![(rhs_true_exit, t_id), (rhs_false_exit, f_id)];
            if let (Some(func), Some(cur_bb)) = (&self.current_function, self.current_block) {
                crate::mir::phi_core::common::debug_verify_phi_inputs(func, cur_bb, &inputs);
            }
            self.emit_instruction(MirInstruction::Phi { dst: rhs_bool, inputs })?;
            self.value_types.insert(rhs_bool, MirType::Bool);
            rhs_bool
        };
        let else_exit_block = self.current_block()?;
        let else_var_map_end = self.variable_map.clone();
        if !self.is_current_block_terminated() {
            self.hint_scope_leave(0);
            self.emit_instruction(MirInstruction::Jump { target: merge_block })?;
        }

        // ---- MERGE ----
        self.current_block = Some(merge_block);
        self.ensure_block_exists(merge_block)?;
        self.push_if_merge(merge_block);

        // Result PHI (bool)
        let result_val = self.value_gen.next();
        let inputs = vec![(then_exit_block, then_value_raw), (else_exit_block, else_value_raw)];
        if let (Some(func), Some(cur_bb)) = (&self.current_function, self.current_block) {
            crate::mir::phi_core::common::debug_verify_phi_inputs(func, cur_bb, &inputs);
        }
        self.emit_instruction(MirInstruction::Phi { dst: result_val, inputs })?;
        self.value_types.insert(result_val, MirType::Bool);

        // Merge modified vars from both branches back into current scope
        self.merge_modified_vars(
            then_block,
            else_block,
            then_exit_block,
            Some(else_exit_block),
            &pre_if_var_map,
            &then_var_map_end,
            &Some(else_var_map_end),
            None,
        )?;

        self.pop_if_merge();
        Ok(result_val)
    }

    // Build a unary operation
    pub(super) fn build_unary_op(
        &mut self,
        operator: String,
        operand: ASTNode,
    ) -> Result<ValueId, String> {
        let operand_val = self.build_expression(operand)?;
        // Core-13 純化: UnaryOp を直接 展開（Neg/Not/BitNot）
        if crate::config::env::mir_core13_pure() {
            match operator.as_str() {
                "-" => {
                    let zero = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Const {
                        dst: zero,
                        value: crate::mir::ConstValue::Integer(0),
                    })?;
                    let dst = self.value_gen.next();
                    self.emit_instruction(MirInstruction::BinOp {
                        dst,
                        op: crate::mir::BinaryOp::Sub,
                        lhs: zero,
                        rhs: operand_val,
                    })?;
                    return Ok(dst);
                }
                "!" | "not" => {
                    let f = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Const {
                        dst: f,
                        value: crate::mir::ConstValue::Bool(false),
                    })?;
                    let dst = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Compare {
                        dst,
                        op: crate::mir::CompareOp::Eq,
                        lhs: operand_val,
                        rhs: f,
                    })?;
                    return Ok(dst);
                }
                "~" => {
                    let all1 = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Const {
                        dst: all1,
                        value: crate::mir::ConstValue::Integer(-1),
                    })?;
                    let dst = self.value_gen.next();
                    self.emit_instruction(MirInstruction::BinOp {
                        dst,
                        op: crate::mir::BinaryOp::BitXor,
                        lhs: operand_val,
                        rhs: all1,
                    })?;
                    return Ok(dst);
                }
                _ => {}
            }
        }
        let dst = self.value_gen.next();
        let mir_op = self.convert_unary_operator(operator)?;
        self.emit_instruction(MirInstruction::UnaryOp {
            dst,
            op: mir_op,
            operand: operand_val,
        })?;
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
            BinaryOperator::Shl => Ok(BinaryOpType::Arithmetic(BinaryOp::Shl)),
            BinaryOperator::Shr => Ok(BinaryOpType::Arithmetic(BinaryOp::Shr)),
            BinaryOperator::BitAnd => Ok(BinaryOpType::Arithmetic(BinaryOp::BitAnd)),
            BinaryOperator::BitOr => Ok(BinaryOpType::Arithmetic(BinaryOp::BitOr)),
            BinaryOperator::BitXor => Ok(BinaryOpType::Arithmetic(BinaryOp::BitXor)),
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
