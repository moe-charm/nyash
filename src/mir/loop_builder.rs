/*!
 * MIR Loop Builder - SSA形式でのループ構築専用モジュール
 *
 * Sealed/Unsealed blockとPhi nodeを使った正しいループ実装
 * Based on Gemini's recommendation for proper SSA loop handling
 */

use super::{BasicBlockId, ConstValue, MirInstruction, ValueId};
use crate::ast::ASTNode;
use std::collections::{HashMap, HashSet};

/// 不完全なPhi nodeの情報
#[derive(Debug, Clone)]
struct IncompletePhi {
    /// Phi nodeの結果ValueId
    phi_id: ValueId,
    /// 変数名
    var_name: String,
    /// 既知の入力値 (predecessor block id, value)
    known_inputs: Vec<(BasicBlockId, ValueId)>,
}

/// ループビルダー - SSA形式でのループ構築を管理
pub struct LoopBuilder<'a> {
    /// 親のMIRビルダーへの参照
    parent_builder: &'a mut super::builder::MirBuilder,

    /// ループ内で追跡する変数の不完全Phi node
    incomplete_phis: HashMap<BasicBlockId, Vec<IncompletePhi>>,

    /// ブロックごとの変数マップ（スコープ管理）
    #[allow(dead_code)]
    block_var_maps: HashMap<BasicBlockId, HashMap<String, ValueId>>,

    /// ループヘッダーID（continueで使用）
    loop_header: Option<BasicBlockId>,

    /// continue文からの変数スナップショット
    continue_snapshots: Vec<(BasicBlockId, HashMap<String, ValueId>)>,

    /// PHI を生成しないモードかどうか
    no_phi_mode: bool,
}

// Local copy: detect a variable name assigned within an AST fragment
fn extract_assigned_var_local(ast: &ASTNode) -> Option<String> {
    match ast {
        ASTNode::Assignment { target, .. } => {
            if let ASTNode::Variable { name, .. } = target.as_ref() {
                Some(name.clone())
            } else {
                None
            }
        }
        ASTNode::Program { statements, .. } => statements
            .last()
            .and_then(|st| extract_assigned_var_local(st)),
        ASTNode::If {
            then_body,
            else_body,
            ..
        } => {
            let then_prog = ASTNode::Program {
                statements: then_body.clone(),
                span: crate::ast::Span::unknown(),
            };
            let tvar = extract_assigned_var_local(&then_prog);
            let evar = else_body.as_ref().and_then(|eb| {
                let ep = ASTNode::Program {
                    statements: eb.clone(),
                    span: crate::ast::Span::unknown(),
                };
                extract_assigned_var_local(&ep)
            });
            match (tvar, evar) {
                (Some(tv), Some(ev)) if tv == ev => Some(tv),
                _ => None,
            }
        }
        _ => None,
    }
}

impl<'a> LoopBuilder<'a> {
    /// 新しいループビルダーを作成
    pub fn new(parent: &'a mut super::builder::MirBuilder) -> Self {
        let no_phi_mode = parent.is_no_phi_mode();
        Self {
            parent_builder: parent,
            incomplete_phis: HashMap::new(),
            block_var_maps: HashMap::new(),
            loop_header: None,
            continue_snapshots: Vec::new(),
            no_phi_mode,
        }
    }

    /// SSA形式でループを構築
    pub fn build_loop(
        &mut self,
        condition: ASTNode,
        body: Vec<ASTNode>,
    ) -> Result<ValueId, String> {
        // 1. ブロックの準備
        let preheader_id = self.current_block()?;
        let (header_id, body_id, after_loop_id) =
            crate::mir::builder::loops::create_loop_blocks(self.parent_builder);
        self.loop_header = Some(header_id);
        self.continue_snapshots.clear();

        // 2. Preheader -> Header へのジャンプ
        self.emit_jump(header_id)?;
        let _ = crate::mir::builder::loops::add_predecessor(self.parent_builder, header_id, preheader_id);

        // 3. Headerブロックの準備（unsealed状態）
        self.set_current_block(header_id)?;
        let _ = self.mark_block_unsealed(header_id);

        // 4. ループ変数のPhi nodeを準備
        // ここでは、ループ内で変更される可能性のある変数を事前に検出するか、
        // または変数アクセス時に遅延生成する
        self.prepare_loop_variables(header_id, preheader_id)?;

        // 5. 条件評価（Phi nodeの結果を使用）
        let condition_value = self.build_expression_with_phis(condition)?;

        // 6. 条件分岐
        self.emit_branch(condition_value, body_id, after_loop_id)?;
        let _ = crate::mir::builder::loops::add_predecessor(self.parent_builder, body_id, header_id);
        let _ = crate::mir::builder::loops::add_predecessor(self.parent_builder, after_loop_id, header_id);

        // 7. ループボディの構築
        self.set_current_block(body_id)?;
        // Optional safepoint per loop-iteration
        if std::env::var("NYASH_BUILDER_SAFEPOINT_LOOP")
            .ok()
            .as_deref()
            == Some("1")
        {
            self.emit_safepoint()?;
        }

        // ボディをビルド
        for stmt in body {
            self.build_statement(stmt)?;
        }
        // 8. Latchブロック（ボディの最後）からHeaderへ戻る
        // 現在の挿入先が latch（最後のブロック）なので、そのブロックIDでスナップショットを保存する
        let latch_id = self.current_block()?;
        let latch_snapshot = self.get_current_variable_map();
        // 以前は body_id に保存していたが、複数ブロックのボディや continue 混在時に不正確になるため
        // 実際の latch_id に対してスナップショットを紐づける
        self.block_var_maps.insert(latch_id, latch_snapshot);
        self.emit_jump(header_id)?;
        let _ = crate::mir::builder::loops::add_predecessor(self.parent_builder, header_id, latch_id);

        // 9. Headerブロックをシール（全predecessors確定）
        self.seal_block(header_id, latch_id)?;

        // 10. ループ後の処理
        self.set_current_block(after_loop_id)?;
        // Pop loop context
        crate::mir::builder::loops::pop_loop_context(self.parent_builder);

        // void値を返す
        let void_dst = self.new_value();
        self.emit_const(void_dst, ConstValue::Void)?;

        Ok(void_dst)
    }

    /// ループ変数の準備（事前検出または遅延生成）
    fn prepare_loop_variables(
        &mut self,
        header_id: BasicBlockId,
        preheader_id: BasicBlockId,
    ) -> Result<(), String> {
        // 現在の変数マップから、ループで使用される可能性のある変数を取得
        let current_vars = self.get_current_variable_map();
        // preheader時点のスナップショット（後でphi入力の解析に使う）
        self.block_var_maps
            .insert(preheader_id, current_vars.clone());

        // 各変数に対して不完全なPhi nodeを作成
        let mut incomplete_phis = Vec::new();
        for (var_name, &value_before) in &current_vars {
            let phi_id = self.new_value();

            // 不完全なPhi nodeを作成（preheaderからの値のみ設定）
            let incomplete_phi = IncompletePhi {
                phi_id,
                var_name: var_name.clone(),
                known_inputs: vec![(preheader_id, value_before)],
            };

            incomplete_phis.push(incomplete_phi);

            if self.no_phi_mode {
                self.parent_builder
                    .insert_edge_copy(preheader_id, phi_id, value_before)?;
            }

            // 変数マップを更新（Phi nodeの結果を使用）
            self.update_variable(var_name.clone(), phi_id);
        }

        // 不完全なPhi nodeを記録
        self.incomplete_phis.insert(header_id, incomplete_phis);

        Ok(())
    }

    /// ブロックをシールし、不完全なPhi nodeを完成させる
    fn seal_block(&mut self, block_id: BasicBlockId, latch_id: BasicBlockId) -> Result<(), String> {
        // 不完全なPhi nodeを取得
        if let Some(incomplete_phis) = self.incomplete_phis.remove(&block_id) {
            for mut phi in incomplete_phis {
                for (cid, snapshot) in &self.continue_snapshots {
                    if let Some(v) = snapshot.get(&phi.var_name) {
                        phi.known_inputs.push((*cid, *v));
                    }
                }

                let value_after = self
                    .get_variable_at_block(&phi.var_name, latch_id)
                    .ok_or_else(|| format!("Variable {} not found at latch block", phi.var_name))?;

                phi.known_inputs.push((latch_id, value_after));

                if self.no_phi_mode {
                    let mut seen: HashSet<BasicBlockId> = HashSet::new();
                    for &(pred, val) in &phi.known_inputs {
                        if seen.insert(pred) {
                            self.parent_builder
                                .insert_edge_copy(pred, phi.phi_id, val)?;
                        }
                    }
                } else {
                    self.emit_phi_at_block_start(block_id, phi.phi_id, phi.known_inputs)?;
                }
                self.update_variable(phi.var_name.clone(), phi.phi_id);
            }
        }

        // ブロックをシール済みとしてマーク
        self.mark_block_sealed(block_id)?;

        Ok(())
    }

    // --- ヘルパーメソッド（親ビルダーへの委譲） ---

    fn current_block(&self) -> Result<BasicBlockId, String> {
        self.parent_builder
            .current_block
            .ok_or_else(|| "No current block".to_string())
    }

    fn new_block(&mut self) -> BasicBlockId {
        self.parent_builder.block_gen.next()
    }

    fn new_value(&mut self) -> ValueId {
        self.parent_builder.value_gen.next()
    }

    fn set_current_block(&mut self, block_id: BasicBlockId) -> Result<(), String> {
        self.parent_builder.start_new_block(block_id)
    }

    fn emit_jump(&mut self, target: BasicBlockId) -> Result<(), String> {
        self.parent_builder
            .emit_instruction(MirInstruction::Jump { target })
    }

    fn emit_branch(
        &mut self,
        condition: ValueId,
        then_bb: BasicBlockId,
        else_bb: BasicBlockId,
    ) -> Result<(), String> {
        self.parent_builder
            .emit_instruction(MirInstruction::Branch {
                condition,
                then_bb,
                else_bb,
            })
    }

    fn emit_safepoint(&mut self) -> Result<(), String> {
        self.parent_builder
            .emit_instruction(MirInstruction::Safepoint)
    }

    fn emit_const(&mut self, dst: ValueId, value: ConstValue) -> Result<(), String> {
        self.parent_builder
            .emit_instruction(MirInstruction::Const { dst, value })
    }

    fn emit_phi_at_block_start(
        &mut self,
        block_id: BasicBlockId,
        dst: ValueId,
        inputs: Vec<(BasicBlockId, ValueId)>,
    ) -> Result<(), String> {
        // Phi nodeをブロックの先頭に挿入
        if let Some(ref mut function) = self.parent_builder.current_function {
            if let Some(block) = function.get_block_mut(block_id) {
                // Phi命令は必ずブロックの先頭に配置
                let phi_inst = MirInstruction::Phi { dst, inputs };
                block.instructions.insert(0, phi_inst);
                Ok(())
            } else {
                Err(format!("Block {} not found", block_id))
            }
        } else {
            Err("No current function".to_string())
        }
    }

    fn add_predecessor(&mut self, block: BasicBlockId, pred: BasicBlockId) -> Result<(), String> {
        if let Some(ref mut function) = self.parent_builder.current_function {
            if let Some(block) = function.get_block_mut(block) {
                block.add_predecessor(pred);
                Ok(())
            } else {
                Err(format!("Block {} not found", block))
            }
        } else {
            Err("No current function".to_string())
        }
    }

    fn mark_block_unsealed(&mut self, _block_id: BasicBlockId) -> Result<(), String> {
        // ブロックはデフォルトでunsealedなので、特に何もしない
        // （既にBasicBlock::newでsealed: falseに初期化されている）
        Ok(())
    }

    fn mark_block_sealed(&mut self, block_id: BasicBlockId) -> Result<(), String> {
        if let Some(ref mut function) = self.parent_builder.current_function {
            if let Some(block) = function.get_block_mut(block_id) {
                block.seal();
                Ok(())
            } else {
                Err(format!("Block {} not found", block_id))
            }
        } else {
            Err("No current function".to_string())
        }
    }

    fn get_current_variable_map(&self) -> HashMap<String, ValueId> {
        self.parent_builder.variable_map.clone()
    }

    fn update_variable(&mut self, name: String, value: ValueId) {
        self.parent_builder.variable_map.insert(name, value);
    }

    fn get_variable_at_block(&self, name: &str, block_id: BasicBlockId) -> Option<ValueId> {
        // まずブロックごとのスナップショットを優先
        if let Some(map) = self.block_var_maps.get(&block_id) {
            if let Some(v) = map.get(name) {
                return Some(*v);
            }
        }
        // フォールバック：現在の変数マップ（単純ケース用）
        self.parent_builder.variable_map.get(name).copied()
    }

    fn build_expression_with_phis(&mut self, expr: ASTNode) -> Result<ValueId, String> {
        // Phi nodeの結果を考慮しながら式を構築
        self.parent_builder.build_expression(expr)
    }

    fn build_statement(&mut self, stmt: ASTNode) -> Result<ValueId, String> {
        match stmt {
            ASTNode::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                // Lower a simple if inside loop, ensuring continue/break inside branches are handled
                let cond_val = self.parent_builder.build_expression(*condition.clone())?;
                let then_bb = self.new_block();
                let else_bb = self.new_block();
                let merge_bb = self.new_block();
                self.emit_branch(cond_val, then_bb, else_bb)?;

                // Capture pre-if variable map (used for phi normalization)
                let pre_if_var_map = self.get_current_variable_map();
                let pre_then_var_value = pre_if_var_map.clone();

                // then
                self.set_current_block(then_bb)?;
                for s in then_body.iter().cloned() {
                    let _ = self.build_statement(s)?;
                    // Stop if block terminated
                    let cur_id = self.current_block()?;
                    let terminated = {
                        if let Some(ref fun_ro) = self.parent_builder.current_function {
                            if let Some(bb) = fun_ro.get_block(cur_id) {
                                bb.is_terminated()
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    };
                    if terminated {
                        break;
                    }
                }
                let then_var_map_end = self.get_current_variable_map();
                // Only jump to merge if not already terminated (e.g., continue/break)
                {
                    let cur_id = self.current_block()?;
                    let need_jump = {
                        if let Some(ref fun_ro) = self.parent_builder.current_function {
                            if let Some(bb) = fun_ro.get_block(cur_id) {
                                !bb.is_terminated()
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    };
                    if need_jump {
                        self.emit_jump(merge_bb)?;
                    }
                }

                // else
                self.set_current_block(else_bb)?;
                let mut else_var_map_end_opt: Option<
                    std::collections::HashMap<String, super::ValueId>,
                > = None;
                if let Some(es) = else_body.clone() {
                    for s in es.into_iter() {
                        let _ = self.build_statement(s)?;
                        let cur_id = self.current_block()?;
                        let terminated = {
                            if let Some(ref fun_ro) = self.parent_builder.current_function {
                                if let Some(bb) = fun_ro.get_block(cur_id) {
                                    bb.is_terminated()
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        };
                        if terminated {
                            break;
                        }
                    }
                    else_var_map_end_opt = Some(self.get_current_variable_map());
                }
                {
                    let cur_id = self.current_block()?;
                    let need_jump = {
                        if let Some(ref fun_ro) = self.parent_builder.current_function {
                            if let Some(bb) = fun_ro.get_block(cur_id) {
                                !bb.is_terminated()
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    };
                    if need_jump {
                        self.emit_jump(merge_bb)?;
                    }
                }

                // Continue at merge
                self.set_current_block(merge_bb)?;
                // If both branches assign the same variable, emit phi and bind it
                let then_prog = ASTNode::Program {
                    statements: then_body.clone(),
                    span: crate::ast::Span::unknown(),
                };
                let assigned_then = extract_assigned_var_local(&then_prog);
                let assigned_else = else_body.as_ref().and_then(|es| {
                    let ep = ASTNode::Program {
                        statements: es.clone(),
                        span: crate::ast::Span::unknown(),
                    };
                    extract_assigned_var_local(&ep)
                });
                if let Some(var_name) = assigned_then {
                    let else_assigns_same = assigned_else
                        .as_ref()
                        .map(|s| s == &var_name)
                        .unwrap_or(false);
                    let then_value_for_var = then_var_map_end.get(&var_name).copied();
                    let else_value_for_var = if else_assigns_same {
                        else_var_map_end_opt
                            .as_ref()
                            .and_then(|m| m.get(&var_name).copied())
                    } else {
                        pre_then_var_value.get(&var_name).copied()
                    };
                    if let (Some(tv), Some(ev)) = (then_value_for_var, else_value_for_var) {
                        let phi_id = self.new_value();
                        if self.no_phi_mode {
                            self.parent_builder.insert_edge_copy(then_bb, phi_id, tv)?;
                            self.parent_builder.insert_edge_copy(else_bb, phi_id, ev)?;
                        } else {
                            self.emit_phi_at_block_start(
                                merge_bb,
                                phi_id,
                                vec![(then_bb, tv), (else_bb, ev)],
                            )?;
                        }
                        // Reset to pre-if map and bind the phi result
                        self.parent_builder.variable_map = pre_if_var_map.clone();
                        self.parent_builder.variable_map.insert(var_name, phi_id);
                    }
                }
                let void_id = self.new_value();
                self.emit_const(void_id, ConstValue::Void)?;
                Ok(void_id)
            }
            ASTNode::Break { .. } => {
                // Jump to loop exit (after_loop_id) if available
                let cur_block = self.current_block()?;
                if let Some(exit_bb) = crate::mir::builder::loops::current_exit(self.parent_builder) {
                    self.emit_jump(exit_bb)?;
                    let _ = crate::mir::builder::loops::add_predecessor(self.parent_builder, exit_bb, cur_block);
                }
                // Continue building in a fresh (unreachable) block to satisfy callers
                let next_block = self.new_block();
                self.set_current_block(next_block)?;
                let void_id = self.new_value();
                self.emit_const(void_id, ConstValue::Void)?;
                Ok(void_id)
            }
            ASTNode::Continue { .. } => {
                let snapshot = self.get_current_variable_map();
                let cur_block = self.current_block()?;
                self.block_var_maps.insert(cur_block, snapshot.clone());
                self.continue_snapshots.push((cur_block, snapshot));

                if let Some(header) = self.loop_header {
                    self.emit_jump(header)?;
                    let _ = crate::mir::builder::loops::add_predecessor(self.parent_builder, header, cur_block);
                }

                let next_block = self.new_block();
                self.set_current_block(next_block)?;

                let void_id = self.new_value();
                self.emit_const(void_id, ConstValue::Void)?;
                Ok(void_id)
            }
            other => self.parent_builder.build_expression(other),
        }
    }
}
