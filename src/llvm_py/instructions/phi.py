"""
PHI instruction lowering
Critical for SSA form - handles value merging from different control flow paths
"""

import llvmlite.ir as ir
from typing import Dict, List, Tuple, Optional

def lower_phi(
    builder: ir.IRBuilder,
    dst_vid: int,
    incoming: List[Tuple[int, int]],  # [(value_id, block_id), ...]
    vmap: Dict[int, ir.Value],
    bb_map: Dict[int, ir.Block],
    current_block: ir.Block,
    resolver=None,  # Resolver instance (optional)
    block_end_values: Optional[Dict[int, Dict[int, ir.Value]]] = None,
    preds_map: Optional[Dict[int, List[int]]] = None
) -> None:
    """
    Lower MIR PHI instruction
    
    Args:
        builder: Current LLVM IR builder
        dst_vid: Destination value ID
        incoming: List of (value_id, block_id) pairs
        vmap: Value map
        bb_map: Block map
        current_block: Current basic block
        resolver: Optional resolver for advanced type handling
    """
    if not incoming:
        # No incoming edges - use zero
        vmap[dst_vid] = ir.Constant(ir.IntType(64), 0)
        return
    
    # Determine PHI type from snapshots or fallback i64
    phi_type = ir.IntType(64)
    if block_end_values is not None:
        for val_id, pred_bid in incoming:
            snap = block_end_values.get(pred_bid, {})
            val = snap.get(val_id)
            if val is not None and hasattr(val, 'type'):
                phi_type = val.type
                # Prefer pointer type
                if hasattr(phi_type, 'is_pointer') and phi_type.is_pointer:
                    break
    
    # Create PHI instruction
    phi = builder.phi(phi_type, name=f"phi_{dst_vid}")
    
    # Build map from provided incoming
    incoming_map: Dict[int, int] = {}
    for val_id, block_id in incoming:
        incoming_map[block_id] = val_id

    # Resolve actual predecessor set
    cur_bid = None
    try:
        cur_bid = int(str(current_block.name).replace('bb',''))
    except Exception:
        pass
    actual_preds = []
    if preds_map is not None and cur_bid is not None:
        actual_preds = [p for p in preds_map.get(cur_bid, []) if p != cur_bid]
    else:
        # Fallback: use blocks in incoming list
        actual_preds = [b for _, b in incoming]

    # Add incoming for each actual predecessor
    for block_id in actual_preds:
        block = bb_map.get(block_id)
        # Prefer pred snapshot
        if block_end_values is not None:
            snap = block_end_values.get(block_id, {})
            vid = incoming_map.get(block_id)
            val = snap.get(vid) if vid is not None else None
        else:
            vid = incoming_map.get(block_id)
            val = vmap.get(vid) if vid is not None else None
        
        if not val:
            # Create default value based on type
            if isinstance(phi_type, ir.IntType):
                val = ir.Constant(phi_type, 0)
            elif isinstance(phi_type, ir.DoubleType):
                val = ir.Constant(phi_type, 0.0)
            else:
                # Pointer type - null
                val = ir.Constant(phi_type, None)
        
        if not block:
            # Skip if block not found
            continue
            
        # Type conversion if needed
        if hasattr(val, 'type') and val.type != phi_type:
            # Position at end (before terminator) of predecessor block
            pb = ir.IRBuilder(block)
            try:
                term = block.terminator
                if term is not None:
                    pb.position_before(term)
                else:
                    pb.position_at_end(block)
            except Exception:
                pb.position_at_end(block)
            
            # Convert types
            if isinstance(phi_type, ir.IntType) and val.type.is_pointer:
                val = pb.ptrtoint(val, phi_type, name=f"cast_p2i_{val_id}")
            elif phi_type.is_pointer and isinstance(val.type, ir.IntType):
                val = pb.inttoptr(val, phi_type, name=f"cast_i2p_{val_id}")
            elif isinstance(phi_type, ir.IntType) and isinstance(val.type, ir.IntType):
                # Int to int
                if phi_type.width > val.type.width:
                    val = pb.zext(val, phi_type, name=f"zext_{val_id}")
                else:
                    val = pb.trunc(val, phi_type, name=f"trunc_{val_id}")
        
        # Add to PHI (skip if no block)
        if block is not None:
            phi.add_incoming(val, block)
    
    # Store PHI result
    vmap[dst_vid] = phi

def defer_phi_wiring(
    dst_vid: int,
    incoming: List[Tuple[int, int]],
    phi_deferrals: List[Tuple[int, List[Tuple[int, int]]]]
) -> None:
    """
    Defer PHI wiring for sealed block approach
    
    Args:
        dst_vid: Destination value ID
        incoming: Incoming edges
        phi_deferrals: List to store deferred PHIs
    """
    phi_deferrals.append((dst_vid, incoming))
