"""
Compare instruction lowering
Handles comparison operations (<, >, <=, >=, ==, !=)
"""

import llvmlite.ir as ir
from typing import Dict
from .externcall import lower_externcall

def lower_compare(
    builder: ir.IRBuilder,
    op: str,
    lhs: int,
    rhs: int,
    dst: int,
    vmap: Dict[int, ir.Value],
    resolver=None,
    current_block=None,
    preds=None,
    block_end_values=None,
    bb_map=None
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
    if resolver is not None and preds is not None and block_end_values is not None and current_block is not None:
        lhs_val = resolver.resolve_i64(lhs, current_block, preds, block_end_values, vmap, bb_map)
        rhs_val = resolver.resolve_i64(rhs, current_block, preds, block_end_values, vmap, bb_map)
    else:
        lhs_val = vmap.get(lhs)
        rhs_val = vmap.get(rhs)

    i64 = ir.IntType(64)
    i8p = ir.IntType(8).as_pointer()

    # String-aware equality: if either side is a pointer or tagged as string-ish, compare via eq_hh
    if op in ('==','!='):
        lhs_ptr = hasattr(lhs_val, 'type') and isinstance(lhs_val.type, ir.PointerType)
        rhs_ptr = hasattr(rhs_val, 'type') and isinstance(rhs_val.type, ir.PointerType)
        lhs_tag = False
        rhs_tag = False
        try:
            if resolver is not None and hasattr(resolver, 'is_stringish'):
                lhs_tag = resolver.is_stringish(lhs)
                rhs_tag = resolver.is_stringish(rhs)
        except Exception:
            pass
        if lhs_ptr or rhs_ptr or lhs_tag or rhs_tag:
            # Convert both to handles (i64) then nyash.string.eq_hh
            # nyash.box.from_i8_string(i8*) -> i64
            box_from = None
            for f in builder.module.functions:
                if f.name == 'nyash.box.from_i8_string':
                    box_from = f
                    break
            if not box_from:
                box_from = ir.Function(builder.module, ir.FunctionType(i64, [i8p]), name='nyash.box.from_i8_string')
            def to_h(v):
                if hasattr(v, 'type') and isinstance(v.type, ir.PointerType):
                    return builder.call(box_from, [v])
                else:
                    # assume i64 handle or number; zext/trunc to i64 if needed
                    if hasattr(v, 'type') and isinstance(v.type, ir.IntType) and v.type.width != 64:
                        return builder.zext(v, i64) if v.type.width < 64 else builder.trunc(v, i64)
                    if hasattr(v, 'type') and isinstance(v.type, ir.PointerType):
                        return builder.ptrtoint(v, i64)
                    return v if hasattr(v, 'type') else ir.Constant(i64, 0)
            lh = to_h(lhs_val)
            rh = to_h(rhs_val)
            eqf = None
            for f in builder.module.functions:
                if f.name == 'nyash.string.eq_hh':
                    eqf = f
                    break
            if not eqf:
                eqf = ir.Function(builder.module, ir.FunctionType(i64, [i64, i64]), name='nyash.string.eq_hh')
            eq = builder.call(eqf, [lh, rh], name='str_eq')
            if op == '==':
                vmap[dst] = eq
            else:
                one = ir.Constant(i64, 1)
                ne = builder.sub(one, eq, name='str_ne')
                vmap[dst] = ne
            return

    # Default integer compare path
    if lhs_val is None:
        lhs_val = ir.Constant(i64, 0)
    if rhs_val is None:
        rhs_val = ir.Constant(i64, 0)

    # Ensure both are i64
    if hasattr(lhs_val, 'type') and isinstance(lhs_val.type, ir.PointerType):
        lhs_val = builder.ptrtoint(lhs_val, i64)
    if hasattr(rhs_val, 'type') and isinstance(rhs_val.type, ir.PointerType):
        rhs_val = builder.ptrtoint(rhs_val, i64)
    
    # Perform signed comparison using canonical predicates ('<','>','<=','>=','==','!=')
    pred = op if op in ('<','>','<=','>=','==','!=') else '=='
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
    
    # Perform ordered comparison using canonical predicates
    pred = op if op in ('<','>','<=','>=','==','!=') else '=='
    cmp_result = builder.fcmp_ordered(pred, lhs_val, rhs_val, name=f"fcmp_{dst}")
    
    # Convert i1 to i64
    i64 = ir.IntType(64)
    result = builder.zext(cmp_result, i64, name=f"fcmp_i64_{dst}")
    
    # Store result
    vmap[dst] = result
