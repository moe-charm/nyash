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
    resolver=None  # Resolver instance (optional)
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
    
    # Determine PHI type from first incoming value
    first_val_id = incoming[0][0]
    first_val = vmap.get(first_val_id)
    
    if first_val and hasattr(first_val, 'type'):
        phi_type = first_val.type
    else:
        # Default to i64
        phi_type = ir.IntType(64)
    
    # Create PHI instruction
    phi = builder.phi(phi_type, name=f"phi_{dst_vid}")
    
    # Add incoming values
    for val_id, block_id in incoming:
        val = vmap.get(val_id)
        block = bb_map.get(block_id)
        
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
            # Save current position
            saved_block = builder.block
            saved_pos = None
            if hasattr(builder, '_anchor'):
                saved_pos = builder._anchor
            
            # Position at end of predecessor block
            builder.position_at_end(block)
            
            # Convert types
            if isinstance(phi_type, ir.IntType) and val.type.is_pointer:
                val = builder.ptrtoint(val, phi_type, name=f"cast_p2i_{val_id}")
            elif phi_type.is_pointer and isinstance(val.type, ir.IntType):
                val = builder.inttoptr(val, phi_type, name=f"cast_i2p_{val_id}")
            elif isinstance(phi_type, ir.IntType) and isinstance(val.type, ir.IntType):
                # Int to int
                if phi_type.width > val.type.width:
                    val = builder.zext(val, phi_type, name=f"zext_{val_id}")
                else:
                    val = builder.trunc(val, phi_type, name=f"trunc_{val_id}")
            
            # Restore position
            builder.position_at_end(saved_block)
            if saved_pos and hasattr(builder, '_anchor'):
                builder._anchor = saved_pos
        
        # Add to PHI
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