"""
Compare instruction lowering
Handles comparison operations (<, >, <=, >=, ==, !=)
"""

import llvmlite.ir as ir
from typing import Dict, Optional, Any
from utils.values import resolve_i64_strict
from .externcall import lower_externcall
from trace import values as trace_values

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
    bb_map=None,
    meta: Optional[Dict[str, Any]] = None,
    ctx: Optional[Any] = None,
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
    # If BuildCtx is provided, prefer its maps for consistency.
    if ctx is not None:
        try:
            if getattr(ctx, 'resolver', None) is not None:
                resolver = ctx.resolver
            if getattr(ctx, 'preds', None) is not None and preds is None:
                preds = ctx.preds
            if getattr(ctx, 'block_end_values', None) is not None and block_end_values is None:
                block_end_values = ctx.block_end_values
            if getattr(ctx, 'bb_map', None) is not None and bb_map is None:
                bb_map = ctx.bb_map
        except Exception:
            pass
    # Get operands
    # Prefer same-block SSA from vmap; fallback to resolver for cross-block dominance
    lhs_val = resolve_i64_strict(resolver, lhs, current_block, preds, block_end_values, vmap, bb_map)
    rhs_val = resolve_i64_strict(resolver, rhs, current_block, preds, block_end_values, vmap, bb_map)

    i64 = ir.IntType(64)
    i8p = ir.IntType(8).as_pointer()

    # String-aware equality: if meta marks string or either side is tagged string-ish,
    # compare handles directly via nyash.string.eq_hh
    if op in ('==','!='):
        force_string = False
        try:
            if isinstance(meta, dict) and meta.get('cmp_kind') == 'string':
                force_string = True
        except Exception:
            pass
        lhs_tag = False
        rhs_tag = False
        try:
            if resolver is not None and hasattr(resolver, 'is_stringish'):
                lhs_tag = resolver.is_stringish(lhs)
                rhs_tag = resolver.is_stringish(rhs)
        except Exception:
            pass
        if force_string or lhs_tag or rhs_tag:
            trace_values(f"[compare] string-eq path: lhs={lhs} rhs={rhs} force={force_string} tagL={lhs_tag} tagR={rhs_tag}")
            # Prefer same-block SSA (vmap) since string handles are produced in-place; fallback to resolver
            lh = lhs_val if lhs_val is not None else (
                resolver.resolve_i64(lhs, current_block, preds, block_end_values, vmap, bb_map)
                if (resolver is not None and preds is not None and block_end_values is not None and current_block is not None) else ir.Constant(i64, 0)
            )
            rh = rhs_val if rhs_val is not None else (
                resolver.resolve_i64(rhs, current_block, preds, block_end_values, vmap, bb_map)
                if (resolver is not None and preds is not None and block_end_values is not None and current_block is not None) else ir.Constant(i64, 0)
            )
            trace_values(f"[compare] string-eq args: lh_is_const={isinstance(lh, ir.Constant)} rh_is_const={isinstance(rh, ir.Constant)}")
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
    # Store the canonical i1 compare result. Consumers that require i64
    # should explicitly cast at their use site (e.g., via resolver or
    # instruction-specific lowering) to avoid emitting casts after
    # terminators when used as branch conditions.
    vmap[dst] = cmp_result

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
