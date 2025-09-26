"""
Jump instruction lowering
Unconditional branch to target block
"""

import llvmlite.ir as ir
from typing import Dict

def lower_jump(
    builder: ir.IRBuilder,
    target_bid: int,
    bb_map: Dict[int, ir.Block]
) -> None:
    """
    Lower MIR Jump instruction
    
    Args:
        builder: Current LLVM IR builder
        target_bid: Target block ID
        bb_map: Map from block ID to LLVM block
    """
    target_bb = bb_map.get(target_bid)
    if target_bb:
        builder.branch(target_bb)

