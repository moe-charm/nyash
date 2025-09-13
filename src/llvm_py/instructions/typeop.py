"""
TypeOp instruction lowering
Handles type conversions and type checks
"""

import llvmlite.ir as ir
from typing import Dict, Optional

def lower_typeop(
    builder: ir.IRBuilder,
    op: str,
    src_vid: int,
    dst_vid: int,
    target_type: Optional[str],
    vmap: Dict[int, ir.Value],
    resolver=None,
    preds=None,
    block_end_values=None,
    bb_map=None
) -> None:
    """
    Lower MIR TypeOp instruction
    
    Operations:
    - cast: Type conversion
    - is: Type check
    - as: Safe cast
    
    Args:
        builder: Current LLVM IR builder
        op: Operation type (cast, is, as)
        src_vid: Source value ID
        dst_vid: Destination value ID
        target_type: Target type name (e.g., "StringBox", "IntegerBox")
        vmap: Value map
        resolver: Optional resolver for type handling
    """
    if resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
        src_val = resolver.resolve_i64(src_vid, builder.block, preds, block_end_values, vmap, bb_map)
    else:
        src_val = vmap.get(src_vid, ir.Constant(ir.IntType(64), 0))
    
    if op == "cast":
        # Type casting - for now just pass through
        # In real implementation, would check/convert box types
        vmap[dst_vid] = src_val
        
    elif op == "is":
        # Type check - returns boolean (i64: 1 or 0)
        # For now, simplified implementation
        if target_type == "IntegerBox":
            # Check if it's a valid integer box handle
            # Simplified: non-zero value
            if hasattr(src_val, 'type') and src_val.type == ir.IntType(64):
                zero = ir.Constant(ir.IntType(64), 0)
                result = builder.icmp_unsigned('!=', src_val, zero)
                # Convert i1 to i64
                result = builder.zext(result, ir.IntType(64))
            else:
                result = ir.Constant(ir.IntType(64), 0)
        else:
            # For other types, would need runtime type info
            result = ir.Constant(ir.IntType(64), 0)
        
        vmap[dst_vid] = result
        
    elif op == "as":
        # Safe cast - returns value or null/0
        # For now, same as cast
        vmap[dst_vid] = src_val
        
    else:
        # Unknown operation
        vmap[dst_vid] = ir.Constant(ir.IntType(64), 0)

def lower_convert(
    builder: ir.IRBuilder,
    src_vid: int,
    dst_vid: int,
    from_type: str,
    to_type: str,
    vmap: Dict[int, ir.Value],
    resolver=None,
    preds=None,
    block_end_values=None,
    bb_map=None
) -> None:
    """
    Lower type conversion between primitive types
    
    Args:
        builder: Current LLVM IR builder
        src_vid: Source value ID
        dst_vid: Destination value ID
        from_type: Source type (i32, i64, f64, ptr)
        to_type: Target type
        vmap: Value map
    """
    if resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
        # Choose resolution based on from_type
        if from_type == "ptr":
            src_val = resolver.resolve_ptr(src_vid, builder.block, preds, block_end_values, vmap)
        else:
            src_val = resolver.resolve_i64(src_vid, builder.block, preds, block_end_values, vmap, bb_map)
    else:
        src_val = vmap.get(src_vid)
    if not src_val:
        # Default based on target type
        if to_type == "f64":
            vmap[dst_vid] = ir.Constant(ir.DoubleType(), 0.0)
        elif to_type == "ptr":
            i8 = ir.IntType(8)
            vmap[dst_vid] = ir.Constant(i8.as_pointer(), None)
        else:
            vmap[dst_vid] = ir.Constant(ir.IntType(64), 0)
        return
    
    # Perform conversion
    if from_type == "i64" and to_type == "f64":
        # int to float
        result = builder.sitofp(src_val, ir.DoubleType())
    elif from_type == "f64" and to_type == "i64":
        # float to int
        result = builder.fptosi(src_val, ir.IntType(64))
    elif from_type == "i64" and to_type == "ptr":
        # int to pointer
        i8 = ir.IntType(8)
        result = builder.inttoptr(src_val, i8.as_pointer(), name=f"conv_i2p_{dst_vid}")
    elif from_type == "ptr" and to_type == "i64":
        # pointer to int
        result = builder.ptrtoint(src_val, ir.IntType(64), name=f"conv_p2i_{dst_vid}")
    elif from_type == "i32" and to_type == "i64":
        # sign extend
        result = builder.sext(src_val, ir.IntType(64))
    elif from_type == "i64" and to_type == "i32":
        # truncate
        result = builder.trunc(src_val, ir.IntType(32))
    else:
        # Unknown conversion - pass through
        result = src_val
    
    vmap[dst_vid] = result
