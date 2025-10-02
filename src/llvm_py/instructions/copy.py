"""
Copy instruction lowering
MIR13 PHI-off uses explicit copies along edges/blocks to model merges.
"""

import llvmlite.ir as ir
from typing import Dict, Optional, Any
from utils.values import resolve_i64_strict

def lower_copy(
    builder: ir.IRBuilder,
    dst: int,
    src: int,
    vmap: Dict[int, ir.Value],
    resolver=None,
    current_block=None,
    preds=None,
    block_end_values=None,
    bb_map=None,
    ctx: Optional[Any] = None,
):
    """Lower a copy by mapping dst to src value in the current block scope.

    Prefer same-block SSA from vmap; fallback to resolver to preserve
    dominance and to localize values across predecessors.
    """
    # If BuildCtx is provided, prefer its maps for consistency.
    if ctx is not None:
        try:
            if getattr(ctx, 'resolver', None) is not None:
                resolver = ctx.resolver
            if getattr(ctx, 'vmap', None) is not None and vmap is None:
                vmap = ctx.vmap
            if getattr(ctx, 'preds', None) is not None and preds is None:
                preds = ctx.preds
            if getattr(ctx, 'block_end_values', None) is not None and block_end_values is None:
                block_end_values = ctx.block_end_values
            if getattr(ctx, 'bb_map', None) is not None and bb_map is None:
                bb_map = ctx.bb_map
        except Exception:
            pass
    # Prefer local SSA; resolve otherwise to preserve dominance
    val = resolve_i64_strict(resolver, src, current_block, preds, block_end_values, vmap, bb_map, builder=builder)
    if val is None:
        val = ir.Constant(ir.IntType(64), 0)
    vmap[dst] = val
