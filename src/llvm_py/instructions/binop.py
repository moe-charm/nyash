"""
BinOp (Binary Operation) instruction lowering
Handles +, -, *, /, %, &, |, ^, <<, >>
"""

import llvmlite.ir as ir
from typing import Dict, Optional, Any
from utils.values import resolve_i64_strict
from dispatch import PhiDispatchPoint
from typing import Optional
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
    inst_ctx: Optional[Any] = None,
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
    # Resolve operands via DispatchPoint（統一フォールバック経路）。
    # 数値系の binop では DP の i64 正規化を採用し、文字列系（'+' の混在）では raw も参照する。
    lhs_val = PhiDispatchPoint.resolve_i64(builder, resolver, lhs, current_block, preds, block_end_values, vmap, bb_map)
    rhs_val = PhiDispatchPoint.resolve_i64(builder, resolver, rhs, current_block, preds, block_end_values, vmap, bb_map)

    # DP 側で PHI/同ブロックalias/直近add のフォールバックを行っているため、
    # ここでの追加ヒューリスティックは不要。
    if lhs_val is None:
        lhs_val = ir.Constant(ir.IntType(64), 0)
    if rhs_val is None:
        rhs_val = ir.Constant(ir.IntType(64), 0)

    # 旧: pred/decl の走査フォールバックは DP に委譲済み。

    try:
        p = _phi_from_decl(lhs) or _phi_from_pred(lhs)
        if p is not None:
            lhs_val = p
    except Exception:
        pass
    try:
        p = _phi_from_decl(rhs) or _phi_from_pred(rhs)
        if p is not None:
            rhs_val = p
    except Exception:
        pass

    # Convert i1 (boolean) to i64 if needed (from compare operations)
    i64 = ir.IntType(64)
    if hasattr(lhs_val, 'type') and isinstance(lhs_val.type, ir.IntType) and lhs_val.type.width == 1:
        lhs_val = builder.zext(lhs_val, i64, name=f'lhs_i64_{dst}')
    if hasattr(rhs_val, 'type') and isinstance(rhs_val.type, ir.IntType) and rhs_val.type.width == 1:
        rhs_val = builder.zext(rhs_val, i64, name=f'rhs_i64_{dst}')
    
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
            ctx=getattr(resolver, 'ctx', None),
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
                # If we already have an i64 SSA (handle) in vmap/raw or resolved val, prefer pass-through.
                if raw is not None and hasattr(raw, 'type') and isinstance(raw.type, ir.IntType) and raw.type.width == 64:
                    return raw
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
                    # Treat resolved i64 as a handle in string domain（never box numeric here）
                    return val
                return ir.Constant(i64, 0)

            # Decide route: handle+handle when both sides are string-ish; otherwise pointer+int route.
            lhs_tag = False; rhs_tag = False
            try:
                if resolver is not None and hasattr(resolver, 'is_stringish'):
                    lhs_tag = resolver.is_stringish(lhs)
                    rhs_tag = resolver.is_stringish(rhs)
            except Exception:
                pass
            if lhs_tag and rhs_tag:
                # Both sides string-ish: concat_hh(handle, handle)
                hl = to_handle(lhs_raw, lhs_val, 'l', lhs)
                hr = to_handle(rhs_raw, rhs_val, 'r', rhs)
                hh_fnty = ir.FunctionType(i64, [i64, i64])
                callee = None
                for f in builder.module.functions:
                    if f.name == 'nyash.string.concat_hh':
                        callee = f; break
                if callee is None:
                    callee = ir.Function(builder.module, hh_fnty, name='nyash.string.concat_hh')
                res = builder.call(callee, [hl, hr], name=f"concat_hh_{dst}")
                vmap[dst] = res
            else:
                # Mixed string + non-string (e.g., "len=" + 5). Use pointer concat helpers then box.
                i32 = ir.IntType(32); i8p = ir.IntType(8).as_pointer(); i64 = ir.IntType(64)
                # Helper: to i8* pointer for stringish side
                def to_i8p_from_vid(vid: int, raw, val, tag: str):
                    # If raw is pointer-to-array: GEP
                    if raw is not None and hasattr(raw, 'type') and isinstance(raw.type, ir.PointerType):
                        try:
                            if isinstance(raw.type.pointee, ir.ArrayType):
                                c0 = ir.Constant(i32, 0)
                                return builder.gep(raw, [c0, c0], name=f"bin_gep_{tag}_{dst}")
                        except Exception:
                            pass
                    # If we have a string handle: call to_i8p_h
                    to_i8p = None
                    for f in builder.module.functions:
                        if f.name == 'nyash.string.to_i8p_h':
                            to_i8p = f; break
                    if to_i8p is None:
                        to_i8p = ir.Function(builder.module, ir.FunctionType(i8p, [i64]), name='nyash.string.to_i8p_h')
                    # Ensure we pass an i64 handle
                    hv = val
                    if hv is None:
                        hv = ir.Constant(i64, 0)
                    if hasattr(hv, 'type') and isinstance(hv.type, ir.PointerType):
                        hv = builder.ptrtoint(hv, i64, name=f"bin_p2h_{tag}_{dst}")
                    elif hasattr(hv, 'type') and isinstance(hv.type, ir.IntType) and hv.type.width != 64:
                        hv = builder.zext(hv, i64, name=f"bin_zext_h_{tag}_{dst}")
                    return builder.call(to_i8p, [hv], name=f"bin_h2p_{tag}_{dst}")

                # Resolve numeric side as i64 value
                def as_i64(val):
                    if val is None:
                        return ir.Constant(i64, 0)
                    if hasattr(val, 'type') and isinstance(val.type, ir.PointerType):
                        return builder.ptrtoint(val, i64, name=f"bin_p2i_{dst}")
                    if hasattr(val, 'type') and isinstance(val.type, ir.IntType) and val.type.width != 64:
                        return builder.zext(val, i64, name=f"bin_zext_i_{dst}")
                    return val

                if lhs_tag:
                    lp = to_i8p_from_vid(lhs, lhs_raw, lhs_val, 'l')
                    ri = as_i64(rhs_val)
                    cf = None
                    for f in builder.module.functions:
                        if f.name == 'nyash.string.concat_si':
                            cf = f; break
                    if cf is None:
                        cf = ir.Function(builder.module, ir.FunctionType(i8p, [i8p, i64]), name='nyash.string.concat_si')
                    p = builder.call(cf, [lp, ri], name=f"concat_si_{dst}")
                    boxer = None
                    for f in builder.module.functions:
                        if f.name == 'nyash.box.from_i8_string':
                            boxer = f; break
                    if boxer is None:
                        boxer = ir.Function(builder.module, ir.FunctionType(i64, [i8p]), name='nyash.box.from_i8_string')
                    vmap[dst] = builder.call(boxer, [p], name=f"concat_box_{dst}")
                else:
                    li = as_i64(lhs_val)
                    rp = to_i8p_from_vid(rhs, rhs_raw, rhs_val, 'r')
                    cf = None
                    for f in builder.module.functions:
                        if f.name == 'nyash.string.concat_is':
                            cf = f; break
                    if cf is None:
                        cf = ir.Function(builder.module, ir.FunctionType(i8p, [i64, i8p]), name='nyash.string.concat_is')
                    p = builder.call(cf, [li, rp], name=f"concat_is_{dst}")
                    boxer = None
                    for f in builder.module.functions:
                        if f.name == 'nyash.box.from_i8_string':
                            boxer = f; break
                    if boxer is None:
                        boxer = ir.Function(builder.module, ir.FunctionType(i64, [i8p]), name='nyash.box.from_i8_string')
                    vmap[dst] = builder.call(boxer, [p], name=f"concat_box_{dst}")
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
