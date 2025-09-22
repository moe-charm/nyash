/*!
 * Control Flow Utilities - 制御フロー処理の共通ユーティリティ
 * 
 * PHI incoming修正とブロック終端検出の汎用関数群
 * フェーズS（即効止血）からフェーズL（根本解決）まで共通利用
 */

use super::super::{BasicBlockId, MirBuilder};

/// **外部関数**: 現在のブロックが終端済みかチェック
/// 
/// loop_builder.rsで3箇所重複していた処理を統一
/// 
/// # 使用例
/// ```rust
/// if is_current_block_terminated(builder)? {
///     break; // 早期終了
/// }
/// ```
pub fn is_current_block_terminated(builder: &MirBuilder) -> Result<bool, String> {
    let cur_id = builder.current_block
        .ok_or_else(|| "No current block".to_string())?;
    
    if let Some(ref function) = builder.current_function {
        if let Some(bb) = function.get_block(cur_id) {
            Ok(bb.is_terminated())
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

/// **外部関数**: 実到達ブロックを捕捉してJump発行
/// 
/// 最強モード指摘の「実到達predecessor捕捉」を汎用化
/// break/continue後の到達不能ブロックは除外
/// 
/// # 戻り値
/// - `Some(predecessor_id)`: Jump発行済み、PHI incomingに使用可能
/// - `None`: 既に終端済み、PHI incomingから除外すべき
/// 
/// # 使用例
/// ```rust
/// if let Some(pred_id) = capture_actual_predecessor_and_jump(builder, merge_bb)? {
///     phi_incomings.push((pred_id, value));
/// }
/// ```
pub fn capture_actual_predecessor_and_jump(
    builder: &mut MirBuilder,
    target_block: BasicBlockId,
) -> Result<Option<BasicBlockId>, String> {
    let cur_id = builder.current_block
        .ok_or_else(|| "No current block".to_string())?;
    
    let need_jump = !is_current_block_terminated(builder)?;
    
    if need_jump {
        // Jump発行前に実到達ブロックID捕捉（重要！）
        // 既存control_flowモジュールと同じパターンを使用
        builder.emit_instruction(super::super::MirInstruction::Jump { 
            target: target_block 
        })?;
        Ok(Some(cur_id))
    } else {
        // 既に終端済み（break/continue等）、PHI incomingから除外
        Ok(None)
    }
}

/// **外部関数**: 条件付きPHI incoming収集
/// 
/// 到達可能な場合のみincomingをリストに追加
/// フェーズM、フェーズLでの型安全性向上にも対応
/// 
/// # 使用例
/// ```rust
/// let mut incomings = Vec::new();
/// collect_phi_incoming_if_reachable(&mut incomings, then_pred, then_value);
/// collect_phi_incoming_if_reachable(&mut incomings, else_pred, else_value);
/// ```
pub fn collect_phi_incoming_if_reachable(
    incomings: &mut Vec<(BasicBlockId, super::super::ValueId)>,
    predecessor: Option<BasicBlockId>,
    value: super::super::ValueId,
) {
    if let Some(pred_id) = predecessor {
        incomings.push((pred_id, value));
    }
    // None（到達不能）の場合は何もしない
}

/// **外部関数**: 終端チェック付きステートメント実行
/// 
/// build_statement後の終端チェックを自動化
/// フェーズSでの「終端ガード徹底」を支援
/// 
/// # 戻り値
/// - `Ok(true)`: 正常実行、継続可能
/// - `Ok(false)`: 終端済み、ループ脱出すべき
/// - `Err(_)`: エラー
pub fn execute_statement_with_termination_check(
    builder: &mut MirBuilder,
    statement: crate::ast::ASTNode,
) -> Result<bool, String> {
    let _result = builder.build_expression(statement)?;
    
    // 終端チェック（統一処理）
    let terminated = is_current_block_terminated(builder)?;
    Ok(!terminated)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // ユニットテスト（将来追加）
    // - 終端検出の正確性
    // - 実到達ブロック捕捉の正確性
    // - PHI incoming除外の正確性
}