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
        for inst in phi_ops:
            try:
                # 箱化: 直接PHI命令を生成（lower_phi経由だと重複する）
                dst = inst.get('dst')
                incoming_list = inst.get('incoming', [])

                if not incoming_list:
                    # incoming無しの場合は0を設定
                    self.builder.vmap[dst] = ir.Constant(ir.IntType(64), 0)
                    if self.verbose:
                        print(f"[PhiHandler] PHI dst={dst} has no incoming, set to 0")
                    continue

                # PHI命令をブロック先頭に作成
                phi_type = ir.IntType(64)
                phi = phi_builder.phi(phi_type, name=f"phi_{dst}")

                # incoming値を解決して追加
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
                        # 見つからない場合は0
                        val = ir.Constant(phi_type, 0)
                        if self.verbose:
                            print(f"[PhiHandler] Warning: value {value_id} not found, using 0")
                    else:
                        if self.verbose:
                            print(f"[PhiHandler] Resolved value {value_id} = {val}")

                    # PHIに追加
                    phi.add_incoming(val, pred_block)

                # 箱理論: vmapに登録（グローバルと現在の両方）
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

    def get_statistics(self) -> Dict[str, int]:
        """
        統計情報取得

        Returns:
            統計情報dict
        """
        return {
            'total_phi': self.phi_count,
            'collected': len(self.phi_instructions)
        }
