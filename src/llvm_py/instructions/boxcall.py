"""
BoxCall instruction lowering
Core of Nyash's "Everything is Box" philosophy
"""

import llvmlite.ir as ir
from typing import Dict, List, Optional

def lower_boxcall(
    builder: ir.IRBuilder,
    module: ir.Module,
    box_vid: int,
    method_name: str,
    args: List[int],
    dst_vid: Optional[int],
    vmap: Dict[int, ir.Value],
    resolver=None
) -> None:
    """
    Lower MIR BoxCall instruction
    
    Current implementation uses method_id approach for plugin boxes.
    
    Args:
        builder: Current LLVM IR builder
        module: LLVM module
        box_vid: Box instance value ID (handle)
        method_name: Method name to call
        args: List of argument value IDs
        dst_vid: Optional destination for return value
        vmap: Value map
        resolver: Optional resolver for type handling
    """
    # Get box handle (i64)
    box_handle = vmap.get(box_vid, ir.Constant(ir.IntType(64), 0))
    
    # Ensure handle is i64
    if hasattr(box_handle, 'type') and box_handle.type.is_pointer:
        box_handle = builder.ptrtoint(box_handle, ir.IntType(64))
    
    # Method ID dispatch for plugin boxes
    # This matches the current LLVM backend approach
    method_id = hash(method_name) & 0xFFFF  # Simple hash for demo
    
    # Look up or create ny_boxcall_by_id function
    boxcall_func = None
    for f in module.functions:
        if f.name == "ny_boxcall_by_id":
            boxcall_func = f
            break
    
    if not boxcall_func:
        # Declare ny_boxcall_by_id(handle: i64, method_id: i64, args: i8*) -> i64
        i8 = ir.IntType(8)
        i64 = ir.IntType(64)
        i8_ptr = i8.as_pointer()
        
        func_type = ir.FunctionType(i64, [i64, i64, i8_ptr])
        boxcall_func = ir.Function(module, func_type, name="ny_boxcall_by_id")
    
    # Prepare arguments array
    i8 = ir.IntType(8)
    i64 = ir.IntType(64)
    
    if args:
        # Allocate space for arguments (8 bytes per arg)
        args_size = len(args) * 8
        args_ptr = builder.alloca(i8, size=args_size, name="boxcall_args")
        
        # Cast to i64* for storing arguments
        i64_ptr_type = i64.as_pointer()
        args_i64_ptr = builder.bitcast(args_ptr, i64_ptr_type)
        
        # Store each argument
        for i, arg_id in enumerate(args):
            arg_val = vmap.get(arg_id, ir.Constant(i64, 0))
            
            # Ensure i64
            if hasattr(arg_val, 'type'):
                if arg_val.type.is_pointer:
                    arg_val = builder.ptrtoint(arg_val, i64)
                elif arg_val.type != i64:
                    # TODO: Handle other conversions
                    pass
            
            # Calculate offset and store
            idx = ir.Constant(ir.IntType(32), i)
            ptr = builder.gep(args_i64_ptr, [idx])
            builder.store(arg_val, ptr)
        
        # Cast back to i8* for call
        call_args_ptr = builder.bitcast(args_i64_ptr, i8.as_pointer())
    else:
        # No arguments - pass null
        call_args_ptr = ir.Constant(i8.as_pointer(), None)
    
    # Make the boxcall
    method_id_val = ir.Constant(i64, method_id)
    result = builder.call(boxcall_func, [box_handle, method_id_val, call_args_ptr],
                         name=f"boxcall_{method_name}")
    
    # Store result if needed
    if dst_vid is not None:
        vmap[dst_vid] = result