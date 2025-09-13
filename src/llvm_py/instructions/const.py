"""
Const instruction lowering
Handles integer, float, string, and void constants
"""

import llvmlite.ir as ir
from typing import Dict, Any

def lower_const(
    builder: ir.IRBuilder,
    module: ir.Module,
    dst: int,
    value: Dict[str, Any],
    vmap: Dict[int, ir.Value],
    resolver=None
) -> None:
    """
    Lower MIR Const instruction
    
    Args:
        builder: Current LLVM IR builder
        module: LLVM module
        dst: Destination value ID
        value: Const value dict with 'type' and 'value' fields
        vmap: Value map (value_id -> llvm value)
    """
    const_type = value.get('type', 'void')
    const_val = value.get('value')
    
    if const_type == 'i64':
        # Integer constant
        i64 = ir.IntType(64)
        llvm_val = ir.Constant(i64, int(const_val))
        vmap[dst] = llvm_val
        
    elif const_type == 'f64':
        # Float constant
        f64 = ir.DoubleType()
        llvm_val = ir.Constant(f64, float(const_val))
        vmap[dst] = llvm_val
        
    elif const_type == 'string':
        # String constant - create global, store GlobalVariable (not GEP) to avoid dominance issues
        i8 = ir.IntType(8)
        str_val = str(const_val)
        str_bytes = str_val.encode('utf-8') + b'\0'
        arr_ty = ir.ArrayType(i8, len(str_bytes))
        str_const = ir.Constant(arr_ty, bytearray(str_bytes))
        try:
            fn = builder.block.parent
            fn_name = getattr(fn, 'name', 'fn')
        except Exception:
            fn_name = 'fn'
        base = f".str.{fn_name}.{dst}"
        existing = {g.name for g in module.global_values}
        name = base
        n = 1
        while name in existing:
            name = f"{base}.{n}"; n += 1
        g = ir.GlobalVariable(module, arr_ty, name=name)
        g.initializer = str_const
        g.linkage = 'private'
        g.global_constant = True
        # Store the GlobalVariable; resolver.resolve_ptr will emit GEP in the current block
        vmap[dst] = g
        if resolver is not None and hasattr(resolver, 'string_literals'):
            resolver.string_literals[dst] = str_val
        
    elif const_type == 'void':
        # Void/null constant - use i64 zero
        i64 = ir.IntType(64)
        vmap[dst] = ir.Constant(i64, 0)
        
    else:
        # Unknown type - default to i64 zero
        i64 = ir.IntType(64)
        vmap[dst] = ir.Constant(i64, 0)
