"""
Return instruction lowering
Handles void and value returns
"""

import llvmlite.ir as ir
from typing import Dict, Optional

def lower_return(
    builder: ir.IRBuilder,
    value_id: Optional[int],
    vmap: Dict[int, ir.Value],
    return_type: ir.Type,
    resolver=None,
    preds=None,
    block_end_values=None,
    bb_map=None
) -> None:
    """
    Lower MIR Return instruction
    
    Args:
        builder: Current LLVM IR builder
        value_id: Optional return value ID
        vmap: Value map
        return_type: Expected return type
    """
    if value_id is None:
        # Void return
        builder.ret_void()
    else:
        # Get return value (prefer resolver)
        if resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
            if isinstance(return_type, ir.PointerType):
                ret_val = resolver.resolve_ptr(value_id, builder.block, preds, block_end_values, vmap)
            else:
                ret_val = resolver.resolve_i64(value_id, builder.block, preds, block_end_values, vmap, bb_map)
        else:
            ret_val = vmap.get(value_id)
        if not ret_val:
            # Default based on return type
            if isinstance(return_type, ir.IntType):
                ret_val = ir.Constant(return_type, 0)
            elif isinstance(return_type, ir.DoubleType):
                ret_val = ir.Constant(return_type, 0.0)
            else:
                # Pointer type - null
                ret_val = ir.Constant(return_type, None)
        
        # Type adjustment if needed
        if hasattr(ret_val, 'type') and ret_val.type != return_type:
            if isinstance(return_type, ir.IntType) and ret_val.type.is_pointer:
                # ptr to int
                ret_val = builder.ptrtoint(ret_val, return_type, name="ret_p2i")
            elif isinstance(return_type, ir.PointerType) and isinstance(ret_val.type, ir.IntType):
                # int to ptr
                ret_val = builder.inttoptr(ret_val, return_type, name="ret_i2p")
            elif isinstance(return_type, ir.IntType) and isinstance(ret_val.type, ir.IntType):
                # int to int conversion
                if return_type.width < ret_val.type.width:
                    # Truncate
                    ret_val = builder.trunc(ret_val, return_type)
                elif return_type.width > ret_val.type.width:
                    # Zero extend
                    ret_val = builder.zext(ret_val, return_type)
        
        builder.ret(ret_val)
