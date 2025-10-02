"""
Value resolution helpers - DEPRECATED, use PhiDispatchPoint instead

箱理論による統一:
- resolve_i64_strict() は PhiDispatchPoint.resolve_i64() へのラッパーに変更
- 新規コードは PhiDispatchPoint を直接使用してください
- このファイルは後方互換性のためのみ維持されています
"""

from typing import Any, Dict, Optional
import llvmlite.ir as ir

def resolve_i64_strict(
    resolver,
    value_id: int,
    current_block: ir.Block,
    preds: Dict[int, list],
    block_end_values: Dict[int, Dict[int, Any]],
    vmap: Dict[int, Any],
    bb_map: Optional[Dict[int, ir.Block]] = None,
    *,
    prefer_local: bool = True,
    builder: Optional[ir.IRBuilder] = None,
) -> ir.Value:
    """
    DEPRECATED: Use PhiDispatchPoint.resolve_i64() instead

    This function is now a thin wrapper around PhiDispatchPoint for backward compatibility.

    箱理論:
    - 「箱にする」: 値解決を PhiDispatchPoint に統一 ✅
    - 「境界を作る」: 5-tier resolution を明示的に使用 ✅
    - 「戻せる」: 後方互換性維持（既存コードは動作し続ける） ✅
    - 「見える化」: ラッパーであることが明示的 ✅
    """
    from dispatch import PhiDispatchPoint

    # builder が必要（PhiDispatchPoint.resolve_i64 の要件）
    # builderがNoneの場合は、ダミーbuilderを作成（緊急措置）
    if builder is None:
        # ダミーbuilderの作成（警告: これは一時的措置）
        # 本来は呼び出し側でbuilderを渡すべき
        if current_block is not None:
            builder = ir.IRBuilder(current_block)
        else:
            # 完全にbuilderが作れない場合は、古い実装にフォールバック
            val = vmap.get(value_id)
            if prefer_local and val is not None:
                return val
            if resolver is None:
                return ir.Constant(ir.IntType(64), 0)
            return resolver.resolve_i64(value_id, current_block, preds, block_end_values, vmap, bb_map)

    # PhiDispatchPoint の5-tier resolutionを使用（統一箱！）
    return PhiDispatchPoint.resolve_i64(
        builder, resolver, value_id, current_block,
        preds, block_end_values, vmap, bb_map
    )
