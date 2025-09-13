"""
BinOp (Binary Operation) instruction lowering
Handles +, -, *, /, %, &, |, ^, <<, >>
"""

import llvmlite.ir as ir
from typing import Dict
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
    bb_map=None
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
    # Use concat_hh when either side is a pointer string OR tagged as string handle.
    if op == '+':
        i64 = ir.IntType(64)
        i8p = ir.IntType(8).as_pointer()
        lhs_raw = vmap.get(lhs)
        rhs_raw = vmap.get(rhs)
        # pointer present?
        is_ptr_side = (hasattr(lhs_raw, 'type') and isinstance(lhs_raw.type, ir.PointerType)) or \
                      (hasattr(rhs_raw, 'type') and isinstance(rhs_raw.type, ir.PointerType))
        # tagged string handles?（両辺ともに string-ish のときのみ）
        both_tagged = False
        try:
            if resolver is not None and hasattr(resolver, 'is_stringish'):
                both_tagged = resolver.is_stringish(lhs) and resolver.is_stringish(rhs)
        except Exception:
            pass
        is_str = is_ptr_side or both_tagged
        if is_str:
            # Helper: convert raw or resolved value to string handle
            def to_handle(raw, val, tag: str):
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
                    return val
                return ir.Constant(i64, 0)

            hl = to_handle(lhs_raw, lhs_val, 'l')
            hr = to_handle(rhs_raw, rhs_val, 'r')
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
