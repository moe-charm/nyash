"""
NewBox instruction lowering
Handles box creation (new StringBox(), new IntegerBox(), etc.)
"""

import llvmlite.ir as ir
from typing import Dict, List, Optional, Any

def lower_newbox(
    builder: ir.IRBuilder,
    module: ir.Module,
    box_type: str,
    args: List[int],
    dst_vid: int,
    vmap: Dict[int, ir.Value],
    resolver=None,
    ctx: Optional[Any] = None
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
    # Use NyRT shim: prefer birth_h for core boxes, otherwise env.box.new_i64x
    i64 = ir.IntType(64)
    i8p = ir.IntType(8).as_pointer()
    # Core fast paths
    if box_type in ("ArrayBox", "MapBox"):
        birth_name = "nyash.array.birth_h" if box_type == "ArrayBox" else "nyash.map.birth_h"
        birth = None
        for f in module.functions:
            if f.name == birth_name:
                birth = f
                break
        if not birth:
            birth = ir.Function(module, ir.FunctionType(i64, []), name=birth_name)
        handle = builder.call(birth, [], name=f"birth_{box_type}")
        vmap[dst_vid] = handle
        return
    # Prefer variadic shim: nyash.env.box.new_i64x(type_name, argc, a1, a2, a3, a4)
    new_i64x = None
    for f in module.functions:
        if f.name == "nyash.env.box.new_i64x":
            new_i64x = f
            break
    if not new_i64x:
        new_i64x = ir.Function(module, ir.FunctionType(i64, [i8p, i64, i64, i64, i64, i64]), name="nyash.env.box.new_i64x")

    # Build C-string for type name (unique global per function)
    sbytes = (box_type + "\0").encode('utf-8')
    arr_ty = ir.ArrayType(ir.IntType(8), len(sbytes))
    try:
        fn = builder.block.parent
        fn_name = getattr(fn, 'name', 'fn')
    except Exception:
        fn_name = 'fn'
    base = f".box_ty_{fn_name}_{dst_vid}"
    existing = {g.name for g in module.global_values}
    name = base
    n = 1
    while name in existing:
        name = f"{base}.{n}"; n += 1
    g = ir.GlobalVariable(module, arr_ty, name=name)
    g.linkage = 'private'
    g.global_constant = True
    g.initializer = ir.Constant(arr_ty, bytearray(sbytes))
    c0 = ir.Constant(ir.IntType(32), 0)
    ptr = builder.gep(g, [c0, c0], inbounds=True)
    zero = ir.Constant(i64, 0)
    handle = builder.call(new_i64x, [ptr, zero, zero, zero, zero, zero], name=f"new_{box_type}")
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
