"""
Unary operation lowering (negation, logical not, bitwise not)
"""

from typing import Dict, Any, Optional
import llvmlite.ir as ir
from utils.values import resolve_i64_strict


def lower_unop(
    builder: ir.IRBuilder,
    resolver,
    kind: str,
    src: int,
    dst: int,
    vmap: Dict[int, ir.Value],
    current_block: ir.Block,
    preds=None,
    block_end_values=None,
    bb_map=None,
    *,
    ctx: Optional[Any] = None,
) -> None:
    """
    Lower MIR unary op:
      - kind: 'neg' | 'not' | 'bitnot'
    """
    # Fail-Fast: src must be present
    if src is None:
        fn_name = getattr(getattr(builder, 'function', None), 'name', 'unknown')
        bb_name = getattr(current_block, 'name', 'bb?')
        raise TypeError(f"lower_unop: src=None (kind={kind}, fn={fn_name}, bb={bb_name}). Upstream MIR JSON missing operand.")

    # Try to use local SSA first
    val = vmap.get(src)
    # If unknown, resolve as i64 (resolver may localize through PHI)
    if val is None:
        val = resolve_i64_strict(resolver, src, current_block, preds, block_end_values, vmap, bb_map, builder=builder)
    # Logical NOT: prefer i1 when available; otherwise compare == 0
    if kind in ('not', 'logical_not', '!'):
        # If already i1, xor with 1
        if hasattr(val, 'type') and isinstance(val.type, ir.IntType) and val.type.width == 1:
            one = ir.Constant(ir.IntType(1), 1)
            vmap[dst] = builder.xor(val, one, name=f"not_{dst}")
            return
        # If pointer: null check (== null) yields i1
        if hasattr(val, 'type') and isinstance(val.type, ir.PointerType):
            null = ir.Constant(val.type, None)
            vmap[dst] = builder.icmp_unsigned('==', val, null, name=f"notp_{dst}")
            return
        # Else numeric: compare == 0 (i1)
        i64 = ir.IntType(64)
        zero = ir.Constant(i64, 0)
        # Cast to i64 when needed
        if hasattr(val, 'type') and isinstance(val.type, ir.PointerType):
            val = builder.ptrtoint(val, i64, name=f"not_p2i_{dst}")
        elif hasattr(val, 'type') and isinstance(val.type, ir.IntType) and val.type.width != 64:
            val = builder.zext(val, i64, name=f"not_zext_{dst}")
        vmap[dst] = builder.icmp_signed('==', val, zero, name=f"notz_{dst}")
        return
    # Numeric NEG: 0 - val (result i64)
    if kind in ('neg', '-'):
        i64 = ir.IntType(64)
        # Ensure i64
        if hasattr(val, 'type') and isinstance(val.type, ir.PointerType):
            val = builder.ptrtoint(val, i64, name=f"neg_p2i_{dst}")
        elif hasattr(val, 'type') and isinstance(val.type, ir.IntType) and val.type.width != 64:
            val = builder.zext(val, i64, name=f"neg_zext_{dst}")
        zero = ir.Constant(i64, 0)
        vmap[dst] = builder.sub(zero, val, name=f"neg_{dst}")
        return
    # Bitwise NOT: xor with all-ones (result i64)
    if kind in ('bitnot', '~'):
        i64 = ir.IntType(64)
        if hasattr(val, 'type') and isinstance(val.type, ir.PointerType):
            val = builder.ptrtoint(val, i64, name=f"bnot_p2i_{dst}")
        elif hasattr(val, 'type') and isinstance(val.type, ir.IntType) and val.type.width != 64:
            val = builder.zext(val, i64, name=f"bnot_zext_{dst}")
        all1 = ir.Constant(i64, -1)
        vmap[dst] = builder.xor(val, all1, name=f"bnot_{dst}")
        return
    # Fallback: store 0
    vmap[dst] = ir.Constant(ir.IntType(64), 0)
