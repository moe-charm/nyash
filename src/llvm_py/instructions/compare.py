"""
Compare instruction lowering
Handles comparison operations (<, >, <=, >=, ==, !=)
"""

import llvmlite.ir as ir
from typing import Dict

def lower_compare(
    builder: ir.IRBuilder,
    op: str,
    lhs: int,
    rhs: int,
    dst: int,
    vmap: Dict[int, ir.Value]
) -> None:
    """
    Lower MIR Compare instruction
    
    Args:
        builder: Current LLVM IR builder
        op: Comparison operation (<, >, <=, >=, ==, !=)
        lhs: Left operand value ID
        rhs: Right operand value ID
        dst: Destination value ID
        vmap: Value map
    """
    # Get operands
    lhs_val = vmap.get(lhs, ir.Constant(ir.IntType(64), 0))
    rhs_val = vmap.get(rhs, ir.Constant(ir.IntType(64), 0))
    
    # Ensure both are i64
    i64 = ir.IntType(64)
    if hasattr(lhs_val, 'type') and lhs_val.type.is_pointer:
        lhs_val = builder.ptrtoint(lhs_val, i64)
    if hasattr(rhs_val, 'type') and rhs_val.type.is_pointer:
        rhs_val = builder.ptrtoint(rhs_val, i64)
    
    # Map operations to LLVM predicates
    op_map = {
        '<': 'slt',   # signed less than
        '>': 'sgt',   # signed greater than
        '<=': 'sle',  # signed less or equal
        '>=': 'sge',  # signed greater or equal
        '==': 'eq',   # equal
        '!=': 'ne'    # not equal
    }
    
    pred = op_map.get(op, 'eq')
    
    # Perform comparison (returns i1)
    cmp_result = builder.icmp_signed(pred, lhs_val, rhs_val, name=f"cmp_{dst}")
    
    # Convert i1 to i64 (0 or 1)
    result = builder.zext(cmp_result, i64, name=f"cmp_i64_{dst}")
    
    # Store result
    vmap[dst] = result

def lower_fcmp(
    builder: ir.IRBuilder,
    op: str,
    lhs: int,
    rhs: int,
    dst: int,
    vmap: Dict[int, ir.Value]
) -> None:
    """
    Lower floating point comparison
    
    Args:
        builder: Current LLVM IR builder
        op: Comparison operation
        lhs: Left operand value ID
        rhs: Right operand value ID
        dst: Destination value ID
        vmap: Value map
    """
    # Get operands as f64
    f64 = ir.DoubleType()
    lhs_val = vmap.get(lhs, ir.Constant(f64, 0.0))
    rhs_val = vmap.get(rhs, ir.Constant(f64, 0.0))
    
    # Map operations to LLVM predicates
    op_map = {
        '<': 'olt',   # ordered less than
        '>': 'ogt',   # ordered greater than
        '<=': 'ole',  # ordered less or equal
        '>=': 'oge',  # ordered greater or equal
        '==': 'oeq',  # ordered equal
        '!=': 'one'   # ordered not equal
    }
    
    pred = op_map.get(op, 'oeq')
    
    # Perform comparison (returns i1)
    cmp_result = builder.fcmp_ordered(pred, lhs_val, rhs_val, name=f"fcmp_{dst}")
    
    # Convert i1 to i64
    i64 = ir.IntType(64)
    result = builder.zext(cmp_result, i64, name=f"fcmp_i64_{dst}")
    
    # Store result
    vmap[dst] = result