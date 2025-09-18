"""
LoopForm IR implementation
Experimental loop normalization following paper-e-loop-signal-ir
"""

import os
import llvmlite.ir as ir
from phi_wiring import phi_at_block_head
from dataclasses import dataclass
from typing import Dict, Tuple, List, Optional, Any
from instructions.safepoint import insert_automatic_safepoint

@dataclass
class LoopFormContext:
    """
    LoopForm fixed block structure
    preheader → header → body → dispatch → latch/exit
    """
    preheader: ir.Block
    header: ir.Block
    body: ir.Block
    dispatch: ir.Block
    latch: ir.Block
    exit: ir.Block
    loop_id: int
    
    # PHI nodes in dispatch block
    tag_phi: Optional[ir.PhiInstr] = None
    payload_phi: Optional[ir.PhiInstr] = None

def create_loopform_blocks(
    func: ir.Function,
    loop_id: int,
    prefix: str = "main"
) -> LoopFormContext:
    """Create the 6-block LoopForm structure"""
    ctx = LoopFormContext(
        preheader=func.append_basic_block(f"{prefix}_lf{loop_id}_preheader"),
        header=func.append_basic_block(f"{prefix}_lf{loop_id}_header"),
        body=func.append_basic_block(f"{prefix}_lf{loop_id}_body"),
        dispatch=func.append_basic_block(f"{prefix}_lf{loop_id}_dispatch"),
        latch=func.append_basic_block(f"{prefix}_lf{loop_id}_latch"),
        exit=func.append_basic_block(f"{prefix}_lf{loop_id}_exit"),
        loop_id=loop_id
    )
    return ctx

def lower_while_loopform(
    builder: ir.IRBuilder,
    func: ir.Function,
    condition_vid: int,
    body_instructions: List[Any],
    loop_id: int,
    vmap: Dict[int, ir.Value],
    bb_map: Dict[int, ir.Block],
    resolver=None,
    preds=None,
    block_end_values=None,
    ctx=None,
) -> bool:
    """
    Lower a while loop using LoopForm structure
    
    Returns:
        True if LoopForm was applied, False otherwise
    """
    # Check if enabled
    if os.environ.get('NYASH_ENABLE_LOOPFORM') != '1':
        return False
    
    # Create LoopForm blocks
    lf = create_loopform_blocks(func, loop_id)
    
    # Preheader: Jump to header
    builder.position_at_end(lf.preheader)
    builder.branch(lf.header)
    
    # Header: Evaluate condition (insert a safepoint at loop header)
    builder.position_at_end(lf.header)
    try:
        import os
        if os.environ.get('NYASH_LLVM_AUTO_SAFEPOINT', '1') == '1':
            insert_automatic_safepoint(builder, func.module, "loop_header")
    except Exception:
        pass
    if ctx is not None:
        try:
            cond64 = ctx.resolver.resolve_i64(condition_vid, builder.block, ctx.preds, ctx.block_end_values, ctx.vmap, ctx.bb_map)
            zero64 = ir.IntType(64)(0)
            cond = builder.icmp_unsigned('!=', cond64, zero64)
        except Exception:
            cond = vmap.get(condition_vid, ir.Constant(ir.IntType(1), 0))
    elif resolver is not None and preds is not None and block_end_values is not None:
        cond64 = resolver.resolve_i64(condition_vid, builder.block, preds, block_end_values, vmap, bb_map)
        zero64 = ir.IntType(64)(0)
        cond = builder.icmp_unsigned('!=', cond64, zero64)
    else:
        cond = vmap.get(condition_vid, ir.Constant(ir.IntType(1), 0))
    # Convert to i1 if needed
    if hasattr(cond, 'type') and cond.type == ir.IntType(64):
        cond = builder.icmp_unsigned('!=', cond, ir.Constant(ir.IntType(64), 0))
    builder.cbranch(cond, lf.body, lf.dispatch)
    
    # Body: Pass through to dispatch (Phase 1)
    builder.position_at_end(lf.body)
    builder.branch(lf.dispatch)
    
    # Dispatch: Central PHI point
    builder.position_at_end(lf.dispatch)
    i8 = ir.IntType(8)
    i64 = ir.IntType(64)
    
    # Create PHI nodes at the block head (LLVM requires PHIs grouped at top)
    tag_phi = phi_at_block_head(lf.dispatch, i8, name=f"lf{loop_id}_tag")
    payload_phi = phi_at_block_head(lf.dispatch, i64, name=f"lf{loop_id}_payload")
    
    # Add incoming values
    # From header (condition false): Break signal
    tag_phi.add_incoming(ir.Constant(i8, 1), lf.header)  # Break = 1
    payload_phi.add_incoming(ir.Constant(i64, 0), lf.header)
    
    # Switch on tag
    tag_val = tag_phi
    switch = builder.switch(tag_val, lf.exit)
    switch.add_case(ir.Constant(i8, 0), lf.latch)  # Next = 0
    
    # Latch: Back to header (if enabled)
    builder.position_at_end(lf.latch)
    if os.environ.get('NYASH_LOOPFORM_LATCH2HEADER') == '1':
        builder.branch(lf.header)
    else:
        builder.unreachable()
    
    # Exit: Continue after loop
    builder.position_at_end(lf.exit)
    # Builder position will be set by caller
    
    # Store context
    lf.tag_phi = tag_phi
    lf.payload_phi = payload_phi
    
    try:
        from trace import debug as trace_debug
        trace_debug(f"[LoopForm] Created loop structure (id={loop_id})")
    except Exception:
        pass
    
    return True
