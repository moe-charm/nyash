"""
Call instruction lowering
Handles regular function calls (not BoxCall or ExternCall)
"""

import llvmlite.ir as ir
from typing import Dict, List, Optional

def lower_call(
    builder: ir.IRBuilder,
    module: ir.Module,
    func_name: str,
    args: List[int],
    dst_vid: Optional[int],
    vmap: Dict[int, ir.Value],
    resolver=None
) -> None:
    """
    Lower MIR Call instruction
    
    Args:
        builder: Current LLVM IR builder
        module: LLVM module
        func_name: Function name to call
        args: List of argument value IDs
        dst_vid: Optional destination for return value
        vmap: Value map
        resolver: Optional resolver for type handling
    """
    # Look up function in module
    func = None
    for f in module.functions:
        if f.name == func_name:
            func = f
            break
    
    if not func:
        # Function not found - create declaration
        # Default: i64(i64, ...) signature
        ret_type = ir.IntType(64)
        arg_types = [ir.IntType(64)] * len(args)
        func_type = ir.FunctionType(ret_type, arg_types)
        func = ir.Function(module, func_type, name=func_name)
    
    # Prepare arguments
    call_args = []
    for i, arg_id in enumerate(args):
        arg_val = vmap.get(arg_id)
        
        if not arg_val:
            # Default based on expected type
            if i < len(func.args):
                expected_type = func.args[i].type
            else:
                expected_type = ir.IntType(64)
            
            if isinstance(expected_type, ir.IntType):
                arg_val = ir.Constant(expected_type, 0)
            elif isinstance(expected_type, ir.DoubleType):
                arg_val = ir.Constant(expected_type, 0.0)
            else:
                arg_val = ir.Constant(expected_type, None)
        
        # Type conversion if needed
        if i < len(func.args):
            expected_type = func.args[i].type
            if hasattr(arg_val, 'type') and arg_val.type != expected_type:
                if expected_type.is_pointer and isinstance(arg_val.type, ir.IntType):
                    arg_val = builder.inttoptr(arg_val, expected_type)
                elif isinstance(expected_type, ir.IntType) and arg_val.type.is_pointer:
                    arg_val = builder.ptrtoint(arg_val, expected_type)
        
        call_args.append(arg_val)
    
    # Make the call
    result = builder.call(func, call_args, name=f"call_{func_name}")
    
    # Store result if needed
    if dst_vid is not None:
        vmap[dst_vid] = result