/*!
 * MIR Loop Builder - SSA形式でのループ構築専用モジュール
 *
 * Sealed/Unsealed blockとPhi nodeを使った正しいループ実装
 * Based on Gemini's recommendation for proper SSA loop handling
 */

use super::{BasicBlockId, ConstValue, MirInstruction, ValueId};
use crate::mir::phi_core::loop_phi::IncompletePhi;
use crate::ast::ASTNode;
use std::collections::HashMap;

// Phase 15 段階的根治戦略：制御フローユーティリティ
use super::utils::is_current_block_terminated;

// Submodules
mod control_flow;
mod phi;
mod build;
pub(crate) mod carrier_analyzer;

/// ループビルダー - SSA形式でのループ構築を管理
pub struct LoopBuilder<'a> {
    /// 親のMIRビルダーへの参照
    pub(super) parent_builder: &'a mut super::builder::MirBuilder,

    /// ループ内で追跡する変数の不完全Phi node
    pub(super) incomplete_phis: HashMap<BasicBlockId, Vec<IncompletePhi>>,

    /// ブロックごとの変数マップ（スコープ管理）
    #[allow(dead_code)]
    pub(super) block_var_maps: HashMap<BasicBlockId, HashMap<String, ValueId>>,

    /// ループヘッダーID（continueで使用）
    pub(super) loop_header: Option<BasicBlockId>,

    /// continue文からの変数スナップショット
    pub(super) continue_snapshots: Vec<(BasicBlockId, HashMap<String, ValueId>)>,

    /// break文からの変数スナップショット（exit PHI生成用）
    pub(super) exit_snapshots: Vec<(BasicBlockId, HashMap<String, ValueId>)>,

    // フェーズM: no_phi_modeフィールド削除（常にPHI使用）
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{
        BasicBlock, BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirFunction,
        MirInstruction, MirModule, MirType, ValueId,
    };

    #[test]
    fn phi_is_inserted_at_block_start_and_updated_in_place() {
        let mut builder = crate::mir::builder::MirBuilder::new();
        let entry_block = builder.block_gen.next();
        let header_block = builder.block_gen.next();

        let signature = FunctionSignature {
            name: "test_loop".to_string(),
            params: vec![],
            return_type: MirType::Void,
            effects: EffectMask::PURE,
        };

        let mut function = MirFunction::new(signature, entry_block);
        {
            let mut header = BasicBlock::new(header_block);
            header.add_instruction(MirInstruction::Const {
                dst: ValueId::new(1),
                value: ConstValue::Integer(42),
            });
            function.blocks.insert(header_block, header);
        }

        builder.current_module = Some(MirModule::new("test".to_string()));
        builder.current_block = Some(header_block);
        builder.current_function = Some(function);

        let expected_inputs = vec![(entry_block, ValueId::new(3))];
        let phi_dst = ValueId::new(0);

        {
            let mut loop_builder = LoopBuilder::new(&mut builder);
            loop_builder
                .emit_phi_at_block_start(header_block, phi_dst, expected_inputs.clone())
                .expect("phi insertion should succeed");
            loop_builder
                .update_phi_inputs_at_block_start(header_block, phi_dst, expected_inputs.clone())
                .expect("phi update should succeed");
        }

        let function = builder
            .current_function
            .as_ref()
            .expect("function should remain attached");
        let block = function
            .get_block(header_block)
            .expect("header block should exist");

        if let Some(MirInstruction::Phi { dst, inputs }) = block.instructions.first() {
            assert_eq!(*dst, phi_dst);
            assert_eq!(inputs, &expected_inputs);
        } else {
            panic!("expected phi as first instruction, got {:?}", block.instructions.first());
        }

        match block.instructions.get(1) {
            Some(MirInstruction::Const { dst, value }) => {
                assert_eq!(*dst, ValueId::new(1));
                assert_eq!(*value, ConstValue::Integer(42));
            }
            other => panic!("expected const after phi, got {:?}", other),
        }
    }
}


impl<'a> LoopBuilder<'a> {
    /// 新しいループビルダーを作成
    pub fn new(parent: &'a mut super::builder::MirBuilder) -> Self {
        // フェーズM: no_phi_mode初期化削除
        Self {
            parent_builder: parent,
            incomplete_phis: HashMap::new(),
            block_var_maps: HashMap::new(),
            loop_header: None,
            continue_snapshots: Vec::new(),
            exit_snapshots: Vec::new(),  // exit PHI用のスナップショット
            // フェーズM: no_phi_modeフィールド削除
        }
    }

    // --- ヘルパーメソッド（親ビルダーへの委譲） ---

    pub(super) fn current_block(&self) -> Result<BasicBlockId, String> {
        self.parent_builder
            .current_block
            .ok_or_else(|| "No current block".to_string())
    }

    pub(super) fn new_block(&mut self) -> BasicBlockId {
        self.parent_builder.block_gen.next()
    }

    pub(super) fn new_value(&mut self) -> ValueId {
        self.parent_builder.value_gen.next()
    }

    pub(super) fn set_current_block(&mut self, block_id: BasicBlockId) -> Result<(), String> {
        self.parent_builder.start_new_block(block_id)
    }

    pub(super) fn emit_jump(&mut self, target: BasicBlockId) -> Result<(), String> {
        self.parent_builder
            .emit_instruction(MirInstruction::Jump { target })
    }

    pub(super) fn emit_branch(
        &mut self,
        condition: ValueId,
        then_bb: BasicBlockId,
        else_bb: BasicBlockId,
    ) -> Result<(), String> {
        // LocalSSA: ensure condition is materialized in the current block
        let condition_local = self.parent_builder.local_ssa_ensure(condition, 4);
        self.parent_builder
            .emit_instruction(MirInstruction::Branch {
                condition: condition_local,
                then_bb,
                else_bb,
            })
    }

    pub(super) fn emit_safepoint(&mut self) -> Result<(), String> {
        self.parent_builder
            .emit_instruction(MirInstruction::Safepoint)
    }

    pub(super) fn emit_const(&mut self, dst: ValueId, value: ConstValue) -> Result<(), String> {
        self.parent_builder
            .emit_instruction(MirInstruction::Const { dst, value })
    }

    /// ブロック先頭に PHI 命令を挿入（不変条件: PHI は常にブロック先頭）
    pub(super) fn emit_phi_at_block_start(
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

    /// 既に挿入済みの PHI（dst をキー）に対し、入力のみを更新する。
    pub(super) fn update_phi_inputs_at_block_start(
        &mut self,
        block_id: BasicBlockId,
        dst: ValueId,
        inputs: Vec<(BasicBlockId, ValueId)>,
    ) -> Result<(), String> {
        if let Some(ref mut function) = self.parent_builder.current_function {
            if let Some(block) = function.get_block_mut(block_id) {
                for inst in block.instructions.iter_mut() {
                    if let MirInstruction::Phi { dst: d, inputs: in_ref } = inst {
                        if *d == dst {
                            *in_ref = inputs;
                            return Ok(());
                        }
                    } else {
                        break;
                    }
                }
                Err(format!("PHI dst %{} not found at block {}", dst.as_u32(), block_id.as_u32()))
            } else {
                Err(format!("Block {} not found", block_id))
            }
        } else {
            Err("No current function".to_string())
        }
    }

    #[allow(dead_code)]
    pub(super) fn add_predecessor(&mut self, block: BasicBlockId, pred: BasicBlockId) -> Result<(), String> {
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

    // =============================================================
    // Variable Map Utilities — snapshots and rebinding
    // =============================================================
    pub(super) fn get_current_variable_map(&self) -> HashMap<String, ValueId> {
        self.parent_builder.variable_map.clone()
    }

    pub(super) fn update_variable(&mut self, name: String, value: ValueId) {
        // VarMapGuard (loop): すべての関数で関数パラメータ（v%0-v%N）の ValueId を他名にそのまま束縛しない。
        // 片側が単一入力のPHIであっても、パラメータ直結だと後続の BinOp/Compare/Branch が不正な値参照を引き起こす可能性があるため、
        // Copy を噛ませて識別性を保つ（仕様不変）。
        let mut guarded = false;
        let bind_val = if let Some(fun) = self.parent_builder.current_function.as_ref() {
            // Check if value is ANY function parameter (v%0, v%1, ..., v%N) regardless of function name
            if fun.params.contains(&value) {
                // This value is a parameter register (v%0, v%1, ..., v%N)
                let loc = self.parent_builder.value_gen.next();
                // Copy を現在ブロックに挿入（PHI後でも可）。型/起源は伝播。
                let _ = self.parent_builder.emit_instruction(crate::mir::MirInstruction::Copy { dst: loc, src: value });
                guarded = true;
                loc
            } else { value }
        } else { value };
        self.parent_builder.variable_map.insert(name.clone(), bind_val);
        // 追加観測（devオンリー）: VarMapGuard の適用を1行で記録
        if guarded {
            if let Ok(val) = std::env::var("NYASH_VARMAP_TRACE") {
                if matches!(val.as_str(), "1" | "true" | "on" | "yes" | "TRUE" | "ON" | "YES") {
                    let fun_name = self
                        .parent_builder
                        .current_function
                        .as_ref()
                        .map(|f| f.signature.name.clone())
                        .unwrap_or_else(|| "<no-func>".to_string());
                    let cur_bb = self.parent_builder.current_block;
                    eprintln!(
                        "[varmap] guard applied func={} bb={:?} name={} src_me=%{} new=%{}",
                        fun_name,
                        cur_bb,
                        name,
                        value.0,
                        bind_val.0
                    );
                    // 受け手名の簡易一覧も出す（簡易版）
                    let mut names: Vec<String> = Vec::new();
                    for (k, v) in self.parent_builder.variable_map.iter() {
                        if *v == bind_val { names.push(k.clone()); }
                    }
                    names.sort();
                    eprintln!(
                        "[varmap] tag=loop.update_variable recv=%{} names={:?} map_size={}",
                        bind_val.0,
                        names,
                        self.parent_builder.variable_map.len()
                    );
                }
            }
        }
    }

    pub(super) fn get_variable_at_block(&self, name: &str, block_id: BasicBlockId) -> Option<ValueId> {
        // まずブロックごとのスナップショットを優先
        if let Some(map) = self.block_var_maps.get(&block_id) {
            if let Some(v) = map.get(name) {
                return Some(*v);
            }
        }
        // フォールバック：現在の変数マップ（単純ケース用）
        self.parent_builder.variable_map.get(name).copied()
    }

    pub(super) fn build_expression_with_phis(&mut self, expr: ASTNode) -> Result<ValueId, String> {
        // Phi nodeの結果を考慮しながら式を構築
        self.parent_builder.build_expression(expr)
    }

    pub(super) fn build_statement(&mut self, stmt: ASTNode) -> Result<ValueId, String> {
        match stmt {
            // Ensure nested bare blocks inside loops are lowered with loop-aware semantics
            ASTNode::Program { statements, .. } => {
                let mut last = None;
                for s in statements.into_iter() {
                    last = Some(self.build_statement(s)?);
                    // フェーズS修正：統一終端検出ユーティリティ使用
                    if is_current_block_terminated(self.parent_builder)? {
                        break;
                    }
                }
                Ok(last.unwrap_or_else(|| {
                    let void_id = self.new_value();
                    // Emit a void const to keep SSA consistent when block is empty
                    let _ = self.emit_const(void_id, ConstValue::Void);
                    void_id
                }))
            }
            ASTNode::If { condition, then_body, else_body, .. } =>
                self.lower_if_in_loop(*condition, then_body, else_body),
            ASTNode::Break { .. } => self.do_break(),
            ASTNode::Continue { .. } => self.do_continue(),
            other => self.parent_builder.build_expression(other),
        }
    }
}

// Implement phi_core LoopPhiOps on LoopBuilder for in-place delegation
impl crate::mir::phi_core::loop_phi::LoopPhiOps for LoopBuilder<'_> {
    fn new_value(&mut self) -> ValueId { self.new_value() }
    fn emit_phi_at_block_start(
        &mut self,
        block: BasicBlockId,
        dst: ValueId,
        inputs: Vec<(BasicBlockId, ValueId)>,
    ) -> Result<(), String> {
        self.emit_phi_at_block_start(block, dst, inputs)
    }
    fn update_phi_inputs_at_block_start(
        &mut self,
        block: BasicBlockId,
        dst: ValueId,
        inputs: Vec<(BasicBlockId, ValueId)>,
    ) -> Result<(), String> {
        self.update_phi_inputs_at_block_start(block, dst, inputs)
    }
    fn update_var(&mut self, name: String, value: ValueId) { self.update_variable(name, value) }
    fn get_variable_at_block(&mut self, name: &str, block: BasicBlockId) -> Option<ValueId> {
        // Call the inherent method (immutable borrow) to avoid recursion
        LoopBuilder::get_variable_at_block(self, name, block)
    }
    fn debug_verify_phi_inputs(&mut self, merge_bb: BasicBlockId, inputs: &[(BasicBlockId, ValueId)]) {
        if let Some(ref func) = self.parent_builder.current_function {
            crate::mir::phi_core::common::debug_verify_phi_inputs(func, merge_bb, inputs);
        }
    }
}
