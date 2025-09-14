"""
BinOp (Binary Operation) instruction lowering
Handles +, -, *, /, %, &, |, ^, <<, >>
"""

import llvmlite.ir as ir
from typing import Dict, Optional, Any
from .compare import lower_compare
import llvmlite.ir as ir

def lower_binop(
    builder: ir.IRBuilder,
    resolver,  # Resolver instance
    op: str,
    lhs: int,
    rhs: int,
    dst: int,
    vmap: Dict[int, ir.Value],
    current_block: ir.Block,
    preds=None,
    block_end_values=None,
    bb_map=None,
    *,
    dst_type: Optional[Any] = None,
) -> None:
    """
    Lower MIR BinOp instruction
    
    Args:
        builder: Current LLVM IR builder
        resolver: Resolver for value resolution
        op: Operation string (+, -, *, /, etc.)
        lhs: Left operand value ID
        rhs: Right operand value ID
        dst: Destination value ID
        vmap: Value map
        current_block: Current basic block
    """
    # Resolve operands as i64 (using resolver when available)
    # For now, simple vmap lookup
    if resolver is not None and preds is not None and block_end_values is not None:
        lhs_val = resolver.resolve_i64(lhs, current_block, preds, block_end_values, vmap, bb_map)
        rhs_val = resolver.resolve_i64(rhs, current_block, preds, block_end_values, vmap, bb_map)
    else:
        lhs_val = vmap.get(lhs, ir.Constant(ir.IntType(64), 0))
        rhs_val = vmap.get(rhs, ir.Constant(ir.IntType(64), 0))
    
    # Relational/equality operators delegate to compare
    if op in ('==','!=','<','>','<=','>='):
        # Delegate to compare with resolver/preds context to maintain dominance via localization
        lower_compare(
            builder,
            op,
            lhs,
            rhs,
            dst,
            vmap,
            resolver=resolver,
            current_block=current_block,
            preds=preds,
            block_end_values=block_end_values,
            bb_map=bb_map,
        )
        return

    # String-aware concatenation unified to handles (i64).
    # Use concat_hh when either side is a pointer string OR either side is tagged as string handle
    # (including literal strings and PHI-propagated tags).
    if op == '+':
        i64 = ir.IntType(64)
        i8p = ir.IntType(8).as_pointer()
        lhs_raw = vmap.get(lhs)
        rhs_raw = vmap.get(rhs)
        # Prefer handle pipeline to keep handles consistent across blocks/ret
        # pointer present?
        is_ptr_side = (hasattr(lhs_raw, 'type') and isinstance(lhs_raw.type, ir.PointerType)) or \
                      (hasattr(rhs_raw, 'type') and isinstance(rhs_raw.type, ir.PointerType))
        # Explicit dst_type hint from MIR JSON?
        force_string = False
        try:
            if isinstance(dst_type, dict) and dst_type.get('kind') == 'handle' and dst_type.get('box_type') == 'StringBox':
                force_string = True
        except Exception:
            pass
        # tagged string handles?（どちらかが string-ish のとき）
        any_tagged = False
        try:
            if resolver is not None:
                if hasattr(resolver, 'is_stringish'):
                    any_tagged = resolver.is_stringish(lhs) or resolver.is_stringish(rhs)
                # literal strings are tracked separately
                if not any_tagged and hasattr(resolver, 'string_literals'):
                    any_tagged = (lhs in resolver.string_literals) or (rhs in resolver.string_literals)
        except Exception:
            pass
        is_str = force_string or is_ptr_side or any_tagged
        if is_str:
            # Helper: convert raw or resolved value to string handle
            def to_handle(raw, val, tag: str, vid: int):
                # If we already have an i64 in vmap (raw), prefer it
                if raw is not None and hasattr(raw, 'type') and isinstance(raw.type, ir.IntType) and raw.type.width == 64:
                    is_tag = False
                    try:
                        if resolver is not None and hasattr(resolver, 'is_stringish'):
                            is_tag = resolver.is_stringish(vid)
                    except Exception:
                        is_tag = False
                    if force_string or is_tag:
                        return raw
                    # Heuristic: PHI values in string concat are typically handles; prefer pass-through
                    try:
                        raw_is_phi = hasattr(raw, 'add_incoming')
                    except Exception:
                        raw_is_phi = False
                    if raw_is_phi:
                        return raw
                    # Otherwise, box numeric i64 to IntegerBox handle
                    cal = None
                    for f in builder.module.functions:
                        if f.name == 'nyash.box.from_i64':
                            cal = f; break
                    if cal is None:
                        cal = ir.Function(builder.module, ir.FunctionType(i64, [i64]), name='nyash.box.from_i64')
                    v64 = raw
                    return builder.call(cal, [v64], name=f"int_i2h_{tag}_{dst}")
                if raw is not None and hasattr(raw, 'type') and isinstance(raw.type, ir.PointerType):
                    # pointer-to-array -> GEP
                    try:
                        if isinstance(raw.type.pointee, ir.ArrayType):
                            c0 = ir.Constant(ir.IntType(32), 0)
                            raw = builder.gep(raw, [c0, c0], name=f"bin_gep_{tag}_{dst}")
                    except Exception:
                        pass
                    cal = None
                    for f in builder.module.functions:
                        if f.name == 'nyash.box.from_i8_string':
                            cal = f; break
                    if cal is None:
                        cal = ir.Function(builder.module, ir.FunctionType(i64, [i8p]), name='nyash.box.from_i8_string')
                    return builder.call(cal, [raw], name=f"str_ptr2h_{tag}_{dst}")
                # if already i64
                if val is not None and hasattr(val, 'type') and isinstance(val.type, ir.IntType) and val.type.width == 64:
                    # Distinguish handle vs numeric: if vid is tagged string-ish, treat as handle; otherwise box numeric to handle
                    is_tag = False
                    try:
                        if resolver is not None and hasattr(resolver, 'is_stringish'):
                            is_tag = resolver.is_stringish(vid)
                    except Exception:
                        is_tag = False
                    if force_string or is_tag:
                        return val
                    # Heuristic: if vmap has a PHI placeholder for this vid, treat as handle
                    try:
                        maybe_phi = vmap.get(vid)
                        if maybe_phi is not None and hasattr(maybe_phi, 'add_incoming'):
                            return val
                    except Exception:
                        pass
                    # Otherwise, box numeric i64 to IntegerBox handle
                    cal = None
                    for f in builder.module.functions:
                        if f.name == 'nyash.box.from_i64':
                            cal = f; break
                    if cal is None:
                        cal = ir.Function(builder.module, ir.FunctionType(i64, [i64]), name='nyash.box.from_i64')
                    # Ensure value is i64
                    v64 = val if val.type.width == 64 else builder.zext(val, i64)
                    return builder.call(cal, [v64], name=f"int_i2h_{tag}_{dst}")
                return ir.Constant(i64, 0)

            hl = to_handle(lhs_raw, lhs_val, 'l', lhs)
            hr = to_handle(rhs_raw, rhs_val, 'r', rhs)
            # concat_hh(handle, handle) -> handle
            hh_fnty = ir.FunctionType(i64, [i64, i64])
            callee = None
            for f in builder.module.functions:
                if f.name == 'nyash.string.concat_hh':
                    callee = f; break
            if callee is None:
                callee = ir.Function(builder.module, hh_fnty, name='nyash.string.concat_hh')
            res = builder.call(callee, [hl, hr], name=f"concat_hh_{dst}")
            vmap[dst] = res
            # Tag result as string handle so subsequent '+' stays in string domain
            try:
                if resolver is not None and hasattr(resolver, 'mark_string'):
                    resolver.mark_string(dst)
            except Exception:
                pass
            return

    # Ensure both are i64
    i64 = ir.IntType(64)
    if hasattr(lhs_val, 'type') and lhs_val.type != i64:
        # Type conversion if needed
        if lhs_val.type.is_pointer:
            lhs_val = builder.ptrtoint(lhs_val, i64, name=f"binop_lhs_p2i_{dst}")
    if hasattr(rhs_val, 'type') and rhs_val.type != i64:
        if rhs_val.type.is_pointer:
            rhs_val = builder.ptrtoint(rhs_val, i64, name=f"binop_rhs_p2i_{dst}")
    
    # Perform operation
    if op == '+':
        result = builder.add(lhs_val, rhs_val, name=f"add_{dst}")
    elif op == '-':
        result = builder.sub(lhs_val, rhs_val, name=f"sub_{dst}")
    elif op == '*':
        result = builder.mul(lhs_val, rhs_val, name=f"mul_{dst}")
    elif op == '/':
        # Signed division
        result = builder.sdiv(lhs_val, rhs_val, name=f"div_{dst}")
    elif op == '%':
        # Signed remainder
        result = builder.srem(lhs_val, rhs_val, name=f"rem_{dst}")
    elif op == '&':
        result = builder.and_(lhs_val, rhs_val, name=f"and_{dst}")
    elif op == '|':
        result = builder.or_(lhs_val, rhs_val, name=f"or_{dst}")
    elif op == '^':
        result = builder.xor(lhs_val, rhs_val, name=f"xor_{dst}")
    elif op == '<<':
        result = builder.shl(lhs_val, rhs_val, name=f"shl_{dst}")
    elif op == '>>':
        # Arithmetic shift right
        result = builder.ashr(lhs_val, rhs_val, name=f"ashr_{dst}")
    else:
        # Unknown op - return zero
        result = ir.Constant(i64, 0)
    
    # Store result
    vmap[dst] = result
