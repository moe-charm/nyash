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
    vmap: Dict[int, ir.Value]
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
        # String constant - create global and get pointer
        i8 = ir.IntType(8)
        str_val = str(const_val)
        # Create array constant for the string
        str_bytes = str_val.encode('utf-8') + b'\0'
        str_const = ir.Constant(ir.ArrayType(i8, len(str_bytes)),
                               bytearray(str_bytes))
        
        # Create global string constant
        global_name = f".str.{dst}"
        global_str = ir.GlobalVariable(module, str_const.type, name=global_name)
        global_str.initializer = str_const
        global_str.linkage = 'private'
        global_str.global_constant = True
        
        # Get pointer to first element
        indices = [ir.Constant(ir.IntType(32), 0), ir.Constant(ir.IntType(32), 0)]
        ptr = builder.gep(global_str, indices, name=f"str_ptr_{dst}")
        vmap[dst] = ptr
        
    elif const_type == 'void':
        # Void/null constant - use i64 zero
        i64 = ir.IntType(64)
        vmap[dst] = ir.Constant(i64, 0)
        
    else:
        # Unknown type - default to i64 zero
        i64 = ir.IntType(64)
        vmap[dst] = ir.Constant(i64, 0)