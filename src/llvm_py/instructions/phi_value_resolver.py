"""
PHI Value Resolver - 箱理論実践

責任:
- predecessorブロックのblock_end_valuesから値を取得
- 型変換処理（i1→i64, pointer→i64）
- エラーハンドリングとデバッグログ

利点:
- 単一責任原則（Single Responsibility）
- テスト可能（Testable）
- デバッグしやすい（Debuggable）
- 再利用可能（Reusable）
"""

import llvmlite.ir as ir
from typing import Dict, Optional
import sys


class PhiValueResolver:
    """
    箱理論: PHI値解決の責任を分離

    この箱の仕事:
    1. predecessorブロックから値を取得
    2. 型変換を実施
    3. 失敗時のフォールバック
    """

    def __init__(self, block_end_values: Dict[int, Dict[int, ir.Value]],
                 bb_map: Dict[int, ir.Block],
                 debug: bool = False):
        """
        Args:
            block_end_values: {block_id: {value_id: ir.Value}}
            bb_map: {block_id: ir.Block}
            debug: デバッグログ出力
        """
        self.block_end_values = block_end_values
        self.bb_map = bb_map
        self.debug = debug

    def resolve_value(self,
                     value_id: int,
                     predecessor_block_id: int,
                     phi_type: ir.Type) -> Optional[ir.Value]:
        """
        predecessorブロックから値を解決

        Args:
            value_id: 取得したい値のID
            predecessor_block_id: predecessorブロックのID
            phi_type: PHIの型（通常はir.IntType(64)）

        Returns:
            解決されたLLVM Value、または失敗時はNone
        """
        if self.debug:
            print(f"[PhiValueResolver] Resolving value_id={value_id} from block {predecessor_block_id}",
                  file=sys.stderr)

        # Step 1: block_end_valuesから値を取得
        if self.block_end_values is None:
            if self.debug:
                print(f"[PhiValueResolver] block_end_values is None", file=sys.stderr)
            return None

        snapshot = self.block_end_values.get(predecessor_block_id)
        if snapshot is None:
            if self.debug:
                print(f"[PhiValueResolver] No snapshot for block {predecessor_block_id}", file=sys.stderr)
                print(f"[PhiValueResolver] Available blocks: {list(self.block_end_values.keys())}",
                      file=sys.stderr)
            return None

        value = snapshot.get(value_id)
        if value is None:
            if self.debug:
                print(f"[PhiValueResolver] value_id={value_id} not in snapshot", file=sys.stderr)
                print(f"[PhiValueResolver] Available values: {list(snapshot.keys())}", file=sys.stderr)
            return None

        if self.debug:
            print(f"[PhiValueResolver] Found value: {value}, type: {value.type}", file=sys.stderr)

        # Step 2: 型変換処理
        converted_value = self._convert_to_phi_type(value, phi_type, predecessor_block_id)

        if self.debug:
            print(f"[PhiValueResolver] Converted value: {converted_value}", file=sys.stderr)

        return converted_value

    def _convert_to_phi_type(self,
                            value: ir.Value,
                            phi_type: ir.Type,
                            predecessor_block_id: int) -> ir.Value:
        """
        値をPHI型に変換

        Args:
            value: 変換元の値
            phi_type: 目標の型
            predecessor_block_id: predecessorブロックID（型変換用のbuilder配置に使用）

        Returns:
            変換された値
        """
        # 既に正しい型の場合
        if hasattr(value, 'type') and value.type == phi_type:
            return value

        # 型変換が必要な場合
        if hasattr(value, 'type'):
            # i1 → i64 変換（比較演算結果など）
            if isinstance(value.type, ir.IntType) and value.type.width == 1:
                if isinstance(phi_type, ir.IntType) and phi_type.width == 64:
                    # predecessorブロックの末尾で変換を実施
                    pred_block = self.bb_map.get(predecessor_block_id)
                    if pred_block is not None:
                        builder = ir.IRBuilder(pred_block)
                        try:
                            # ブロックの終端命令の前に配置
                            term = pred_block.terminator
                            if term is not None:
                                builder.position_before(term)
                            else:
                                builder.position_at_end(pred_block)
                        except Exception:
                            builder.position_at_end(pred_block)
                        return builder.zext(value, phi_type, name=f"phi_zext_{id(value)}")

            # pointer → i64 変換
            if isinstance(value.type, ir.PointerType):
                if isinstance(phi_type, ir.IntType) and phi_type.width == 64:
                    pred_block = self.bb_map.get(predecessor_block_id)
                    if pred_block is not None:
                        builder = ir.IRBuilder(pred_block)
                        try:
                            term = pred_block.terminator
                            if term is not None:
                                builder.position_before(term)
                            else:
                                builder.position_at_end(pred_block)
                        except Exception:
                            builder.position_at_end(pred_block)
                        return builder.ptrtoint(value, phi_type, name=f"phi_ptoi_{id(value)}")

        # 変換できない場合はそのまま返す
        return value
