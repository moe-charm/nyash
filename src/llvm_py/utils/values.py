"""
Value resolution helpers
Centralize policies like "prefer same-block SSA; otherwise resolve with dominance".
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
) -> ir.Value:
    """Resolve i64 under policies:
    - If prefer_local and vmap has a same-block definition, reuse it.
    - Otherwise, delegate to resolver to localize with PHI/casts as needed.
    """
    # Prefer current vmap SSA first (block-local map is passed in vmap)
    val = vmap.get(value_id)
    if prefer_local and val is not None:
        return val
    # Fallback to resolver
    if resolver is None:
        return ir.Constant(ir.IntType(64), 0)
    return resolver.resolve_i64(value_id, current_block, preds, block_end_values, vmap, bb_map)
