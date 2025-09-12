"""
NewBox instruction lowering
Handles box creation (new StringBox(), new IntegerBox(), etc.)
"""

import llvmlite.ir as ir
from typing import Dict, List, Optional

def lower_newbox(
    builder: ir.IRBuilder,
    module: ir.Module,
    box_type: str,
    args: List[int],
    dst_vid: int,
    vmap: Dict[int, ir.Value],
    resolver=None
) -> None:
    """
    Lower MIR NewBox instruction
    
    Creates a new box instance and returns its handle.
    
    Args:
        builder: Current LLVM IR builder
        module: LLVM module
        box_type: Box type name (e.g., "StringBox", "IntegerBox")
        args: Constructor arguments
        dst_vid: Destination value ID for box handle
        vmap: Value map
        resolver: Optional resolver for type handling
    """
    # Look up or declare the box creation function
    create_func_name = f"ny_create_{box_type}"
    create_func = None
    
    for f in module.functions:
        if f.name == create_func_name:
            create_func = f
            break
    
    if not create_func:
        # Declare box creation function
        # Signature depends on box type
        i64 = ir.IntType(64)
        i8 = ir.IntType(8)
        
        if box_type in ["StringBox", "IntegerBox", "BoolBox"]:
            # Built-in boxes - default constructors (no args)
            # Real implementation may have optional args
            func_type = ir.FunctionType(i64, [])
        else:
            # Generic box - variable arguments
            # For now, assume no args
            func_type = ir.FunctionType(i64, [])
        
        create_func = ir.Function(module, func_type, name=create_func_name)
    
    # Prepare arguments
    call_args = []
    for i, arg_id in enumerate(args):
        arg_val = vmap.get(arg_id)
        
        if not arg_val:
            # Default based on box type
            if box_type == "StringBox":
                # Empty string
                i8 = ir.IntType(8)
                arg_val = ir.Constant(i8.as_pointer(), None)
            else:
                # Zero
                arg_val = ir.Constant(ir.IntType(64), 0)
        
        # Type conversion if needed
        if box_type == "StringBox" and hasattr(arg_val, 'type'):
            if isinstance(arg_val.type, ir.IntType):
                # int to string ptr
                i8 = ir.IntType(8)
                arg_val = builder.inttoptr(arg_val, i8.as_pointer())
        
        call_args.append(arg_val)
    
    # Create the box
    handle = builder.call(create_func, call_args, name=f"new_{box_type}")
    
    # Store handle
    vmap[dst_vid] = handle

def lower_newbox_generic(
    builder: ir.IRBuilder,
    module: ir.Module,
    dst_vid: int,
    vmap: Dict[int, ir.Value]
) -> None:
    """
    Create a generic box with runtime allocation
    
    This is used when box type is not statically known.
    """
    # Look up generic allocation function
    alloc_func = None
    for f in module.functions:
        if f.name == "ny_alloc_box":
            alloc_func = f
            break
    
    if not alloc_func:
        # Declare ny_alloc_box(size: i64) -> i64
        i64 = ir.IntType(64)
        func_type = ir.FunctionType(i64, [i64])
        alloc_func = ir.Function(module, func_type, name="ny_alloc_box")
    
    # Default box size (e.g., 64 bytes)
    size = ir.Constant(ir.IntType(64), 64)
    handle = builder.call(alloc_func, [size], name="new_box")
    
    vmap[dst_vid] = handle