"""
Branch instruction lowering
Conditional branch based on condition value
"""

import llvmlite.ir as ir
from typing import Dict
from utils.values import resolve_i64_strict
from dispatch import PhiDispatchPoint

def lower_branch(
    builder: ir.IRBuilder,
    cond_vid: int,
    then_bid: int,
    else_bid: int,
    vmap: Dict[int, ir.Value],
    bb_map: Dict[int, ir.Block],
    resolver=None,
    preds=None,
    block_end_values=None
) -> None:
    """
    Lower MIR Branch instruction
    
    Args:
        builder: Current LLVM IR builder
        cond_vid: Condition value ID
        then_bid: Then block ID
        else_bid: Else block ID
        vmap: Value map
        bb_map: Block map
    """
    # Resolve condition via DispatchPoint（統一フォールバック経路）
    cond = PhiDispatchPoint.resolve_i64(builder, resolver, cond_vid, builder.block, preds, block_end_values, vmap, bb_map)
    if cond is None:
        # Default to false if missing
        cond = ir.Constant(ir.IntType(1), 0)
    
    # Convert to i1 if needed
    if hasattr(cond, 'type'):
        # If we already have an i1 (canonical compare result), use it directly.
        if isinstance(cond.type, ir.IntType) and cond.type.width == 1:
            pass
        elif isinstance(cond.type, ir.IntType) and cond.type.width == 64:
            # i64 to i1: compare != 0
            zero = ir.Constant(ir.IntType(64), 0)
            cond = builder.icmp_unsigned('!=', cond, zero, name="cond_i1")
        elif isinstance(cond.type, ir.PointerType):
            # Pointer to i1: compare != null
            null = ir.Constant(cond.type, None)
            cond = builder.icmp_unsigned('!=', cond, null, name="cond_p1")
    
    # Get target blocks
    then_bb = bb_map.get(then_bid)
    else_bb = bb_map.get(else_bid)
    
    if then_bb and else_bb:
        builder.cbranch(cond, then_bb, else_bb)
