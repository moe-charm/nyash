"""
PHI Handler - 箱理論実装
PHI命令の処理を一元化し、block_lowerとinstruction_lowerの間を橋渡し

箱理論の実践:
- 「箱にする」: PHI処理を専用モジュールに分離
- 「境界を作る」: block/instruction layer間の責任を明確化
- 「戻せる」: 従来の処理フローも維持可能
- 「見える化」: PHI処理の流れが明確
"""

from typing import Dict, List, Any
import os
from llvmlite import ir


class PhiHandler:
    """
    PHI命令処理の統一ハンドラー

    責任:
    1. PHI命令の検出と分離
    2. PHI命令の適切なタイミングでの処理
    3. デバッグ情報の提供
    """

    def __init__(self, builder, verbose: bool = False):
        """
        Args:
            builder: NyashLLVMBuilder instance
            verbose: Enable debug output
        """
        self.builder = builder
        self.verbose = verbose
        self.phi_instructions: List[Dict[str, Any]] = []
        self.phi_count = 0
        # Track incomplete PHI nodes (for loop PHIs with forward references)
        self.incomplete_phis: List[Dict[str, Any]] = []
        # Record of already wired predecessors for each (block_id, dst_vid)
        # Used to prevent duplicate incoming edges when finalize_phis runs.
        try:
            if not hasattr(builder, 'phi_wired') or builder.phi_wired is None:
                builder.phi_wired = {}
        except Exception:
            try:
                builder.phi_wired = {}
            except Exception:
                pass

    def collect_phi_instructions(self, instructions: List[Dict[str, Any]]) -> tuple:
        """
        命令リストからPHI命令を分離

        Args:
            instructions: 命令リスト

        Returns:
            (phi_ops, non_phi_ops): PHI命令とそれ以外の命令
        """
        phi_ops = []
        non_phi_ops = []

        for inst in instructions:
            op = inst.get('op')
            if op == 'phi':
                phi_ops.append(inst)
                if self.verbose:
                    print(f"[PhiHandler] Collected PHI: dst={inst.get('dst')}")
            else:
                non_phi_ops.append(inst)

        self.phi_instructions = phi_ops
        self.phi_count = len(phi_ops)

        if self.verbose and phi_ops:
            print(f"[PhiHandler] Collected {len(phi_ops)} PHI instructions")

        return phi_ops, non_phi_ops

    def process_phi_instructions(
        self,
        phi_ops: List[Dict[str, Any]],
        block: ir.Block,
        func: ir.Function
    ) -> bool:
        """
        PHI命令を処理（ブロックの先頭に挿入）

        箱理論: 直接PHI生成（重複回避）

        Args:
            phi_ops: PHI命令リスト
            block: 現在の基本ブロック
            func: 現在の関数

        Returns:
            成功した場合True
        """
        if not phi_ops:
            return True

        # PHI命令は必ずブロックの先頭で処理
        phi_builder = ir.IRBuilder(block)
        try:
            phi_builder.position_at_start(block)
        except Exception as e:
            if self.verbose:
                print(f"[PhiHandler] Warning: position_at_start failed: {e}")

        success_count = 0
        strict = os.environ.get('NYASH_LLVM_PHI_STRICT') == '1'
        def _bid_of_block(b):
            try:
                for k, v in self.builder.bb_map.items():
                    if v is b:
                        return int(k)
            except Exception:
                pass
            return None

        for inst in phi_ops:
            try:
                # 箱化: 直接PHI命令を生成（lower_phi経由だと重複する）
                dst = inst.get('dst')
                from phi_wiring.common import incoming_pairs_vb
                incoming_list = [{'block': b, 'value': v} for (v,b) in incoming_pairs_vb(inst)]

                if not incoming_list:
                    # incoming無しの場合は0を設定
                    # strict の場合でも、定数0を vmap に登録する既存挙動は維持
                    self.builder.vmap[dst] = ir.Constant(ir.IntType(64), 0)
                    if self.verbose:
                        print(f"[PhiHandler] PHI dst={dst} has no incoming, set to 0")
                    continue

                # PHI命令をブロック先頭に作成
                phi_type = ir.IntType(64)
                phi = phi_builder.phi(phi_type, name=f"phi_{dst}")

                # strict モード: 生成のみ（配線は finalize_phis に一元化）。
                if strict:
                    self.builder.vmap[dst] = phi
                    if hasattr(self.builder, '_current_vmap'):
                        self.builder._current_vmap[dst] = phi
                    success_count += 1
                    if self.verbose:
                        print(f"[PhiHandler] (STRICT) Created PHI dst={dst} (no incoming wired)")
                    continue

                # incoming値を解決して追加（forward referenceは後で追加）
                incomplete_edges = []
                cur_bid = _bid_of_block(block)
                for item in incoming_list:
                    block_id = item.get('block')
                    value_id = item.get('value')

                    # 前のブロックを取得
                    pred_block = self.builder.bb_map.get(block_id)
                    if pred_block is None:
                        if self.verbose:
                            print(f"[PhiHandler] Warning: block {block_id} not found")
                        continue

                    # 値を解決（block_end_valuesまたはvmapから）
                    val = None
                    if hasattr(self.builder, 'block_end_values'):
                        snap = self.builder.block_end_values.get(block_id, {})
                        val = snap.get(value_id)

                    if val is None and hasattr(self.builder, 'vmap'):
                        val = self.builder.vmap.get(value_id)

                    if val is None:
                        # Forward reference（ループPHI等）- 後で追加
                        incomplete_edges.append({
                            'block_id': block_id,
                            'value_id': value_id,
                            'pred_block': pred_block
                        })
                        if self.verbose:
                            print(f"[PhiHandler] Deferred: value {value_id} from block {block_id} (forward reference)")
                    else:
                        # 解決済みの値を追加
                        if self.verbose:
                            print(f"[PhiHandler] Resolved value {value_id} = {val}")
                        phi.add_incoming(val, pred_block)
                        # Mark as wired to avoid duplicates during finalize
                        try:
                            key = (int(cur_bid) if cur_bid is not None else int(block_id), int(dst))
                            st = self.builder.phi_wired.setdefault(key, set())
                            st.add(int(block_id))
                        except Exception:
                            pass

                # 未完了のedgeがある場合は記録
                if incomplete_edges:
                    self.incomplete_phis.append({
                        'phi': phi,
                        'dst': dst,
                        'block': block,
                        'incomplete_edges': incomplete_edges
                    })
                    if self.verbose:
                        print(f"[PhiHandler] PHI dst={dst} has {len(incomplete_edges)} incomplete edges")

                # 箱理論: vmapに登録（二重登録が必要）
                # 1. vmap（グローバル）: 他のブロックから参照可能にする
                # 2. _current_vmap（ブロックローカル）: 現在のブロック内で即座に使用可能にする
                self.builder.vmap[dst] = phi
                if hasattr(self.builder, '_current_vmap'):
                    self.builder._current_vmap[dst] = phi
                success_count += 1

                if self.verbose:
                    print(f"[PhiHandler] Created PHI dst={dst} with {len(incoming_list)} incoming values")

            except Exception as e:
                if self.verbose:
                    print(f"[PhiHandler] Error processing PHI: {e}")
                    import traceback
                    traceback.print_exc()
                return False

        if self.verbose:
            print(f"[PhiHandler] Successfully processed {success_count}/{len(phi_ops)} PHI instructions")

        return success_count == len(phi_ops)

    def complete_incomplete_phis(self) -> bool:
        """
        未完了のPHI incomingエッジを追加（ループPHI対応）

        ブロック本体の処理後に呼び出して、forward referenceを解決する

        Returns:
            成功した場合True
        """
        # strict モード: 配線は finalize_phis に一元化するため、ここでは何もしない
        if os.environ.get('NYASH_LLVM_PHI_STRICT') == '1':
            if self.verbose:
                print(f"[PhiHandler] (STRICT) skip completing incomplete PHIs")
            return True

        if not self.incomplete_phis:
            if self.verbose:
                print(f"[PhiHandler] No incomplete PHIs to complete")
            return True

        if self.verbose:
            print(f"[PhiHandler] Completing {len(self.incomplete_phis)} incomplete PHIs")

        phi_type = ir.IntType(64)
        completed_count = 0

        def _bid_of_block(b):
            try:
                for k, v in self.builder.bb_map.items():
                    if v is b:
                        return int(k)
            except Exception:
                pass
            return None

        for phi_info in self.incomplete_phis:
            phi = phi_info['phi']
            dst = phi_info['dst']
            incomplete_edges = phi_info['incomplete_edges']
            cur_bid = _bid_of_block(phi_info.get('block'))

            for edge in incomplete_edges:
                value_id = edge['value_id']
                pred_block = edge['pred_block']

                # 値を再度解決（今度はブロック本体処理後なので存在するはず）
                # _current_vmapを優先（ループPHI等、同じブロック内の値）
                val = None
                if hasattr(self.builder, '_current_vmap'):
                    val = self.builder._current_vmap.get(value_id)

                # _current_vmapで見つからない場合はグローバルvmapを確認
                if val is None and hasattr(self.builder, 'vmap'):
                    val = self.builder.vmap.get(value_id)

                if val is None:
                    # それでも見つからない場合は0
                    val = ir.Constant(phi_type, 0)
                    if self.verbose:
                        print(f"[PhiHandler] Warning: value {value_id} still not found after body processing, using 0")
                else:
                    if self.verbose:
                        print(f"[PhiHandler] Completed: value {value_id} = {val}")

                # PHIに追加
                phi.add_incoming(val, pred_block)
                # Mark as wired
                try:
                    pred_bid = int(edge.get('block_id')) if edge.get('block_id') is not None else _bid_of_block(pred_block)
                    key = (int(cur_bid) if cur_bid is not None else int(pred_bid), int(dst))
                    st = self.builder.phi_wired.setdefault(key, set())
                    if pred_bid is not None:
                        st.add(int(pred_bid))
                except Exception:
                    pass
                completed_count += 1

        if self.verbose:
            print(f"[PhiHandler] Completed {completed_count} incomplete PHI edges")

        return True

    def get_statistics(self) -> Dict[str, int]:
        """
        統計情報取得

        Returns:
            統計情報dict
        """
        return {
            'total_phi': self.phi_count,
            'collected': len(self.phi_instructions),
            'incomplete': len(self.incomplete_phis)
        }
