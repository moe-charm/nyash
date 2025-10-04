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
        // Dev-only: dump varmap bindings for BinOp operands to chase sporadic mis-binding
        crate::mir::builder::observe::varmap::emit_recv_names(self, lhs, "binop-lhs");
        crate::mir::builder::observe::varmap::emit_recv_names(self, rhs, "binop-rhs");
        let dst = self.value_gen.next();

        let mir_op = self.convert_binary_operator(operator)?;

        let all_call = std::env::var("NYASH_BUILDER_OPERATOR_BOX_ALL_CALL").ok().as_deref() == Some("1");

        match mir_op {
            // Arithmetic operations
            BinaryOpType::Arithmetic(op) => {
                // Dev: Lower '+' を演算子ボックス呼び出しに置換（既定OFF）
                let in_add_op = self
                    .current_function
                    .as_ref()
                    .map(|f| f.signature.name.starts_with("AddOperator.apply/"))
                    .unwrap_or(false);
                if matches!(op, crate::mir::BinaryOp::Add)
                    && !in_add_op
                    && (all_call || std::env::var("NYASH_BUILDER_OPERATOR_BOX_ADD_CALL").ok().as_deref() == Some("1"))
                {
                    // AddOperator.apply/2(lhs, rhs)
                    let name = "AddOperator.apply/2".to_string();
                    self.emit_legacy_call(Some(dst), super::builder_calls::CallTarget::Global(name), vec![lhs, rhs])?;
                    // 型注釈（従来と同等）
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
                } else if all_call {
                    // Lower other arithmetic ops to operator boxes under ALL flag
                    let (name, guard_prefix) = match op {
                        crate::mir::BinaryOp::Sub => ("SubOperator.apply/2", "SubOperator.apply/"),
                        crate::mir::BinaryOp::Mul => ("MulOperator.apply/2", "MulOperator.apply/"),
                        crate::mir::BinaryOp::Div => ("DivOperator.apply/2", "DivOperator.apply/"),
                        crate::mir::BinaryOp::Mod => ("ModOperator.apply/2", "ModOperator.apply/"),
                        crate::mir::BinaryOp::Shl => ("ShlOperator.apply/2", "ShlOperator.apply/"),
                        crate::mir::BinaryOp::Shr => ("ShrOperator.apply/2", "ShrOperator.apply/"),
                        crate::mir::BinaryOp::BitAnd => ("BitAndOperator.apply/2", "BitAndOperator.apply/"),
                        crate::mir::BinaryOp::BitOr => ("BitOrOperator.apply/2", "BitOrOperator.apply/"),
                        crate::mir::BinaryOp::BitXor => ("BitXorOperator.apply/2", "BitXorOperator.apply/"),
                        _ => ("", ""),
                    };
                    if !name.is_empty() {
                        let in_guard = self.current_function.as_ref().map(|f| f.signature.name.starts_with(guard_prefix)).unwrap_or(false);
                        if !in_guard {
                            self.emit_legacy_call(Some(dst), super::builder_calls::CallTarget::Global(name.to_string()), vec![lhs, rhs])?;
                            // 型注釈: 算術はおおむね整数（Addは上で注釈済み）
                            self.value_types.insert(dst, MirType::Integer);
                        } else {
                            // guard中は従来のBinOp
                            self.emit_instruction(MirInstruction::BinOp { dst, op, lhs, rhs })?;
                            self.value_types.insert(dst, MirType::Integer);
                        }
                    } else {
                        // 既存の算術経路
                        self.emit_instruction(MirInstruction::BinOp { dst, op, lhs, rhs })?;
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
                            self.value_types.insert(dst, MirType::Integer);
                        }
                    }
                } else {
                    // 既存の算術経路
                    self.emit_instruction(MirInstruction::BinOp { dst, op, lhs, rhs })?;
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
                        self.value_types.insert(dst, MirType::Integer);
                    }
                }
            }
            // Comparison operations
            BinaryOpType::Comparison(op) => {
                // Root-fix: always use native compare path (no operator-box lowering)
                let (lhs2_raw, rhs2_raw) = if self
                    .origin_get(lhs)
                    .map(|s| s == "IntegerBox")
                    .unwrap_or(false)
                    && self
                        .origin_get(rhs)
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
                let mut lhs2 = lhs2_raw;
                let mut rhs2 = rhs2_raw;
                crate::mir::builder::ssa::local::finalize_compare(self, &mut lhs2, &mut rhs2);
                crate::mir::builder::emission::compare::emit_to(self, dst, op, lhs2, rhs2)?;
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
        let mut lhs_cond = self.local_cond(lhs_val);
        crate::mir::builder::ssa::local::finalize_branch_cond(self, &mut lhs_cond);
        crate::mir::builder::emission::branch::emit_conditional(self, lhs_cond, then_block, else_block)?;
        // Record predecessor block for branch (for single-pred PHI materialization)
        let pre_branch_bb = self.current_block()?;

        // Snapshot variables before entering branches
        let pre_if_var_map = self.variable_map.clone();

        // ---- THEN branch ----
        self.start_new_block(then_block)?;
        self.hint_scope_enter(0);
        // Reset scope to pre-if snapshot for clean deltas
        self.variable_map = pre_if_var_map.clone();
        // Materialize all variables at entry via single-pred PHI (correctness-first)
        for (name, &pre_v) in pre_if_var_map.iter() {
            let phi_val = self.value_gen.next();
            let inputs = vec![(pre_branch_bb, pre_v)];
            self.emit_instruction(MirInstruction::Phi { dst: phi_val, inputs })?;
            self.variable_map.insert(name.clone(), phi_val);
        }

        // AND: then → evaluate RHS and reduce to bool
        //  OR: then → constant true
        let then_value_raw = if is_and {
            // Reduce arbitrary RHS to bool by branching on its truthiness and returning consts
            let rhs_true = self.block_gen.next();
            let rhs_false = self.block_gen.next();
            let rhs_join = self.block_gen.next();
            let rhs_val = self.build_expression(right.clone())?;
            let mut rhs_cond = self.local_cond(rhs_val);
            crate::mir::builder::ssa::local::finalize_branch_cond(self, &mut rhs_cond);
            crate::mir::builder::emission::branch::emit_conditional(self, rhs_cond, rhs_true, rhs_false)?;
            // true path
            self.start_new_block(rhs_true)?;
            let t_id = crate::mir::builder::emission::constant::emit_bool(self, true);
            crate::mir::builder::emission::branch::emit_jump(self, rhs_join)?;
            let rhs_true_exit = self.current_block()?;
            // false path
            self.start_new_block(rhs_false)?;
            let f_id = crate::mir::builder::emission::constant::emit_bool(self, false);
            crate::mir::builder::emission::branch::emit_jump(self, rhs_join)?;
            let rhs_false_exit = self.current_block()?;
            // join rhs result into a single bool（Helper 統一）
            self.start_new_block(rhs_join)?;
            let rhs_bool = self.value_gen.next();
            let merged = super::phi_merge_helper::PhiMergeHelper::merge_var_value(
                self,
                Some(rhs_true_exit),
                t_id,
                Some(rhs_false_exit),
                f_id,
                rhs_false_exit,
                Some("@sc_rhs"),
                Some(rhs_bool),
            )?.expect("rhs merge must produce a value");
            self.value_types.insert(merged, MirType::Bool);
            merged
        } else {
            let t_id = crate::mir::builder::emission::constant::emit_bool(self, true);
            t_id
        };
        let then_exit_block = self.current_block()?;
        let then_var_map_end = self.variable_map.clone();
        if !self.is_current_block_terminated() {
            self.hint_scope_leave(0);
            crate::mir::builder::emission::branch::emit_jump(self, merge_block)?;
        }

        // ---- ELSE branch ----
        self.start_new_block(else_block)?;
        self.hint_scope_enter(0);
        self.variable_map = pre_if_var_map.clone();
        // Materialize all variables at entry via single-pred PHI (correctness-first)
        for (name, &pre_v) in pre_if_var_map.iter() {
            let phi_val = self.value_gen.next();
            let inputs = vec![(pre_branch_bb, pre_v)];
            self.emit_instruction(MirInstruction::Phi { dst: phi_val, inputs })?;
            self.variable_map.insert(name.clone(), phi_val);
        }
        // AND: else → false
        //  OR: else → evaluate RHS and reduce to bool
        let else_value_raw = if is_and {
            let f_id = crate::mir::builder::emission::constant::emit_bool(self, false);
            f_id
        } else {
            let rhs_true = self.block_gen.next();
            let rhs_false = self.block_gen.next();
            let rhs_join = self.block_gen.next();
            let rhs_val = self.build_expression(right)?;
            let mut rhs_cond = self.local_cond(rhs_val);
            crate::mir::builder::ssa::local::finalize_branch_cond(self, &mut rhs_cond);
            crate::mir::builder::emission::branch::emit_conditional(self, rhs_cond, rhs_true, rhs_false)?;
            // true path
            self.start_new_block(rhs_true)?;
            let t_id = crate::mir::builder::emission::constant::emit_bool(self, true);
            crate::mir::builder::emission::branch::emit_jump(self, rhs_join)?;
            let rhs_true_exit = self.current_block()?;
            // false path
            self.start_new_block(rhs_false)?;
            let f_id = crate::mir::builder::emission::constant::emit_bool(self, false);
            crate::mir::builder::emission::branch::emit_jump(self, rhs_join)?;
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
        // Merge block: suppress entry pin copy so PHIs remain first and materialize pins explicitly
        self.suppress_next_entry_pin_copy();
        self.start_new_block(merge_block)?;
        self.push_if_merge(merge_block);

        // Result merge（Helper 統一）
        let result_val_dst = self.value_gen.next();
        let (then_pred_opt, else_pred_opt) =
            super::phi_merge_helper::PhiMergeHelper::compute_if_merge_preds(self, then_exit_block, else_exit_block);
        let result_val = super::phi_merge_helper::PhiMergeHelper::merge_var_value(
            self,
            then_pred_opt,
            then_value_raw,
            else_pred_opt,
            else_value_raw,
            else_exit_block,
            Some("@sc_result"),
            Some(result_val_dst),
        )?.expect("if result must produce a value");
        self.value_types.insert(result_val, MirType::Bool);

        // Merge modified vars from both branches back into current scope

        self.merge_modified_vars(
            then_block,
            else_block,
            then_pred_opt,
            else_pred_opt,
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
        let all_call = std::env::var("NYASH_BUILDER_OPERATOR_BOX_ALL_CALL").ok().as_deref() == Some("1");
        if all_call {
            let (name, guard_prefix, rett) = match operator.as_str() {
                "-" => ("NegOperator.apply/1", "NegOperator.apply/", MirType::Integer),
                "!" | "not" => ("NotOperator.apply/1", "NotOperator.apply/", MirType::Bool),
                "~" => ("BitNotOperator.apply/1", "BitNotOperator.apply/", MirType::Integer),
                _ => ("", "", MirType::Integer),
            };
            if !name.is_empty() {
                let in_guard = self.current_function.as_ref().map(|f| f.signature.name.starts_with(guard_prefix)).unwrap_or(false);
                let dst = self.value_gen.next();
                if !in_guard {
                    self.emit_legacy_call(Some(dst), super::builder_calls::CallTarget::Global(name.to_string()), vec![operand_val])?;
                    self.value_types.insert(dst, rett);
                    return Ok(dst);
                }
            }
        }
        // Core‑13 pure mode removed; normal UnaryOp path only.
        let mir_op = self.convert_unary_operator(operator)?;
        self.emit_unop(mir_op, operand_val)
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

    pub(super) fn emit_unop(&mut self, op: UnaryOp, operand: ValueId) -> Result<ValueId, String> {
        let dst = self.value_gen.next();
        self.emit_instruction(MirInstruction::UnaryOp {
            dst,
            op,
            operand,
        })?;
        Ok(dst)
    }
}
