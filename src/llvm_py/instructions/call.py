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
    resolver=None,
    preds=None,
    block_end_values=None,
    bb_map=None
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
    # Resolve function: accepts string name or value-id referencing a string literal
    actual_name = func_name
    if not isinstance(func_name, str):
        # Try resolver.string_literals
        if resolver is not None and hasattr(resolver, 'string_literals'):
            actual_name = resolver.string_literals.get(func_name)
    # Look up function in module
    func = None
    if isinstance(actual_name, str):
        for f in module.functions:
            if f.name == actual_name:
                func = f
                break
    
    if not func:
        # Function not found - create declaration with default i64 signature
        ret_type = ir.IntType(64)
        arg_types = [ir.IntType(64)] * len(args)
        name = actual_name if isinstance(actual_name, str) else "unknown_fn"
        func_type = ir.FunctionType(ret_type, arg_types)
        func = ir.Function(module, func_type, name=name)
    
    # Prepare arguments
    call_args = []
    for i, arg_id in enumerate(args):
        arg_val = None
        if i < len(func.args):
            expected_type = func.args[i].type
            if resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
                if hasattr(expected_type, 'is_pointer') and expected_type.is_pointer:
                    arg_val = resolver.resolve_ptr(arg_id, builder.block, preds, block_end_values, vmap)
                else:
                    arg_val = resolver.resolve_i64(arg_id, builder.block, preds, block_end_values, vmap, bb_map)
        if arg_val is None:
            arg_val = vmap.get(arg_id)
        if arg_val is None:
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
        if i < len(func.args):
            expected_type = func.args[i].type
            if hasattr(arg_val, 'type') and arg_val.type != expected_type:
                if expected_type.is_pointer and isinstance(arg_val.type, ir.IntType):
                    arg_val = builder.inttoptr(arg_val, expected_type, name=f"call_i2p_{i}")
                elif isinstance(expected_type, ir.IntType) and arg_val.type.is_pointer:
                    arg_val = builder.ptrtoint(arg_val, expected_type, name=f"call_p2i_{i}")
        call_args.append(arg_val)
    
    # Make the call
    result = builder.call(func, call_args, name=f"call_{func_name}")
    
    # Store result if needed
    if dst_vid is not None:
        vmap[dst_vid] = result
