"""
Simplified PHI instruction lowering - 箱理論実践

旧実装の問題:
- resolverとの複雑な相互依存
- vmapのスコープ問題
- デバッグが困難

新実装の方針:
- PhiValueResolverだけを使用
- シンプルで追跡しやすい
- 箱理論: 各箱が単一責任
"""

import llvmlite.ir as ir
from typing import Dict, List, Tuple, Optional
from phi_wiring import phi_at_block_head
from .phi_value_resolver import PhiValueResolver
import sys


def lower_phi_simple(
    builder: ir.IRBuilder,
    dst_vid: int,
    incoming: List[Tuple[int, int]],  # [(value_id, block_id), ...]
    vmap: Dict[int, ir.Value],
    bb_map: Dict[int, ir.Block],
    current_block: ir.Block,
    block_end_values: Optional[Dict[int, Dict[int, ir.Value]]] = None,
    debug: bool = False
) -> None:
    """
    Simplified PHI instruction lowering

    Args:
        builder: Current LLVM IR builder
        dst_vid: Destination value ID
        incoming: List of (value_id, block_id) pairs
        vmap: Value map (current block)
        bb_map: Block map
        current_block: Current basic block
        block_end_values: End-of-block value snapshots
        debug: Enable debug logging
    """
    if debug:
        print(f"\n[lower_phi_simple] Processing PHI dst={dst_vid}", file=sys.stderr)
        print(f"[lower_phi_simple] incoming={incoming}", file=sys.stderr)
        print(f"[lower_phi_simple] current_block={current_block.name}", file=sys.stderr)

    # 空のincoming
    if not incoming:
        if debug:
            print(f"[lower_phi_simple] No incoming edges, using zero", file=sys.stderr)
        vmap[dst_vid] = ir.Constant(ir.IntType(64), 0)
        return

    # 箱理論: PhiValueResolverを使用
    resolver = PhiValueResolver(block_end_values, bb_map, debug=debug)

    # PHI型（通常はi64）
    phi_type = ir.IntType(64)

    # 各predecessorから値を収集
    incoming_pairs: List[Tuple[ir.Block, ir.Value]] = []

    for value_id, block_id in incoming:
        if debug:
            print(f"\n[lower_phi_simple] Processing incoming: value_id={value_id}, block_id={block_id}",
                  file=sys.stderr)

        # predecessorブロックを取得
        pred_block = bb_map.get(block_id)
        if pred_block is None:
            if debug:
                print(f"[lower_phi_simple] WARNING: block_id={block_id} not found in bb_map", file=sys.stderr)
            continue

        # PhiValueResolverで値を解決
        value = resolver.resolve_value(value_id, block_id, phi_type)

        if value is None:
            if debug:
                print(f"[lower_phi_simple] WARNING: Could not resolve value_id={value_id}, using zero",
                      file=sys.stderr)
            value = ir.Constant(phi_type, 0)

        incoming_pairs.append((pred_block, value))

        if debug:
            print(f"[lower_phi_simple] Added incoming: block={pred_block.name}, value={value}",
                  file=sys.stderr)

    # incoming_pairsが空の場合
    if not incoming_pairs:
        if debug:
            print(f"[lower_phi_simple] No valid incoming pairs, using zero", file=sys.stderr)
        vmap[dst_vid] = ir.Constant(phi_type, 0)
        return

    # PHI命令を生成
    try:
        phi = phi_at_block_head(current_block, phi_type, name=f"phi_{dst_vid}")

        for pred_block, value in incoming_pairs:
            phi.add_incoming(value, pred_block)
            if debug:
                print(f"[lower_phi_simple] phi.add_incoming({value}, {pred_block.name})", file=sys.stderr)

        # vmapに保存
        vmap[dst_vid] = phi

        if debug:
            print(f"[lower_phi_simple] SUCCESS: PHI created and stored in vmap[{dst_vid}]", file=sys.stderr)
            print(f"[lower_phi_simple] PHI instruction: {phi}\n", file=sys.stderr)

    except Exception as e:
        if debug:
            print(f"[lower_phi_simple] ERROR: Failed to create PHI: {e}", file=sys.stderr)
        # フォールバック: 0を使用
        vmap[dst_vid] = ir.Constant(phi_type, 0)
