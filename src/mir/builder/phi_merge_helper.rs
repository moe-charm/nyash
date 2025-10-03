/*!
 * PHI Merge Helper - 到達可能predecessorの計算とPHI生成の統一処理
 *
 * 箱理論4原則の実践:
 * 1. 箱にする: Predecessor判定とPHI生成を独立した箱に分離
 * 2. 境界を作る: PHI生成 vs 直接代入の判断を単一箇所に集約
 * 3. 戻せる: Pure Function設計で副作用なし
 * 4. 見える化: 責務が明確、デバッグトレース付き
 */

use super::{BasicBlockId, MirBuilder, MirInstruction, ValueId};

/// PHI生成とマージ処理のヘルパー（箱理論実践）
pub struct PhiMergeHelper;

impl PhiMergeHelper {
    /// if/else mergeで到達可能predecessorを計算
    ///
    /// # 引数
    /// - `builder`: MIRビルダー
    /// - `then_exit_block`: then分岐の出口ブロック
    /// - `else_exit_block`: else分岐の出口ブロック
    ///
    /// # 戻り値
    /// - `(Option<BasicBlockId>, Option<BasicBlockId>)`: 到達可能なpredecessor
    ///   - `None`の場合、その分岐はreturn/throwで終了している
    pub fn compute_if_merge_preds(
        builder: &MirBuilder,
        then_exit_block: BasicBlockId,
        else_exit_block: BasicBlockId,
    ) -> (Option<BasicBlockId>, Option<BasicBlockId>) {
        // ⚠️ CRITICAL FIX: check for return/throw, NOT just any terminator!
        // Jump is a normal terminator and should be included in predecessors.
        let then_pred = if builder.is_block_ends_with_return_or_throw(then_exit_block) {
            None  // terminated with return/throw → unreachable
        } else {
            Some(then_exit_block)  // has jump or no terminator → reachable
        };
        let else_pred = if builder.is_block_ends_with_return_or_throw(else_exit_block) {
            None
        } else {
            Some(else_exit_block)
        };
        (then_pred, else_pred)
    }

    /// 単一変数の値をマージ（到達可能predecessorのみ）
    ///
    /// # 引数
    /// - `builder`: MIRビルダー
    /// - `then_pred_opt`: then分岐のpredecessor（Noneなら到達不能）
    /// - `then_v`: then分岐の値
    /// - `else_pred_opt`: else分岐のpredecessor（Noneなら到達不能）
    /// - `else_v`: else分岐の値
    /// - `else_block_fallback`: 両方到達不能時のfallbackブロック
    /// - `var_name`: デバッグ用変数名（診断メッセージで使用）
    /// - `dst_opt`: 事前に決まったPHI命令のdst（Noneなら新規生成）
    ///
    /// # 戻り値
    /// - `None`: スキップ（predecessorなし）
    /// - `Some(value)`: マージ後の値（直接代入 or PHI結果）
    ///
    /// # アルゴリズム
    /// 1. 到達可能predecessorのみでinputsを構築
    /// 2. inputs.len() == 0: スキップ（到達不能）
    /// 3. inputs.len() == 1: 直接代入（PHI不要）
    /// 4. inputs.len() >= 2: PHI命令発行（dst_optが指定されていればそれを使用）
    pub fn merge_var_value(
        builder: &mut MirBuilder,
        then_pred_opt: Option<BasicBlockId>,
        then_v: ValueId,
        else_pred_opt: Option<BasicBlockId>,
        else_v: ValueId,
        else_block_fallback: BasicBlockId,
        var_name: Option<&str>,
        dst_opt: Option<ValueId>,
    ) -> Result<Option<ValueId>, String> {
        // 到達可能predecessorのみでinputsを構築
        let mut inputs = Vec::new();
        if let Some(tp) = then_pred_opt {
            inputs.push((tp, then_v));
        }
        if let Some(ep) = else_pred_opt {
            inputs.push((ep, else_v));
        } else if then_pred_opt.is_none() {
            // 両方終了の場合はelse_blockをfallback
            inputs.push((else_block_fallback, else_v));
        }

        match inputs.len() {
            0 => Ok(None), // スキップ
            1 => Ok(Some(inputs[0].1)), // 単一predecessor: 直接代入
            _ => {
                // 複数predecessors: PHI発行
                let merged = dst_opt.unwrap_or_else(|| builder.value_gen.next());

                // デバッグ診断（変数名情報付き）
                if let (Some(func), Some(cur_bb)) = (&builder.current_function, builder.current_block) {
                    if std::env::var("NYASH_PHI_TRACE").ok().as_deref() == Some("1") {
                        let name_str = var_name.unwrap_or("<unnamed>");
                        eprintln!(
                            "[phi-merge] var={} merge_bb={:?} inputs={:?}",
                            name_str, cur_bb, inputs
                        );
                    }
                    crate::mir::phi_core::common::debug_verify_phi_inputs(func, cur_bb, &inputs);
                }

                builder.emit_instruction(MirInstruction::Phi { dst: merged, inputs })?;
                Ok(Some(merged))
            }
        }
    }
}
