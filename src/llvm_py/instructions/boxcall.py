"""
BoxCall instruction lowering
Core of Nyash's "Everything is Box" philosophy
"""

import llvmlite.ir as ir
from typing import Dict, List, Optional

def _declare(module: ir.Module, name: str, ret, args):
    for f in module.functions:
        if f.name == name:
            return f
    fnty = ir.FunctionType(ret, args)
    return ir.Function(module, fnty, name=name)

def _ensure_handle(builder: ir.IRBuilder, module: ir.Module, v: ir.Value) -> ir.Value:
    """Coerce a value to i64 handle. If pointer, box via nyash.box.from_i8_string."""
    i64 = ir.IntType(64)
    if hasattr(v, 'type'):
        if isinstance(v.type, ir.IntType) and v.type.width == 64:
            return v
        if isinstance(v.type, ir.PointerType):
            # call nyash.box.from_i8_string(i8*) -> i64
            i8p = ir.IntType(8).as_pointer()
            # If pointer-to-array, GEP to first element
            try:
                if isinstance(v.type.pointee, ir.ArrayType):
                    c0 = ir.IntType(32)(0)
                    v = builder.gep(v, [c0, c0], name="bc_str_gep")
            except Exception:
                pass
            callee = _declare(module, "nyash.box.from_i8_string", i64, [i8p])
            return builder.call(callee, [v], name="str_ptr2h")
        if isinstance(v.type, ir.IntType):
            # extend/trunc to i64
            return builder.zext(v, i64) if v.type.width < 64 else builder.trunc(v, i64)
    return ir.Constant(i64, 0)

def lower_boxcall(
    builder: ir.IRBuilder,
    module: ir.Module,
    box_vid: int,
    method_name: str,
    args: List[int],
    dst_vid: Optional[int],
    vmap: Dict[int, ir.Value],
    resolver=None,
    preds=None,
    block_end_values=None,
    bb_map=None
) -> None:
    """
    Lower MIR BoxCall instruction
    
    Current implementation uses method_id approach for plugin boxes.
    
    Args:
        builder: Current LLVM IR builder
        module: LLVM module
        box_vid: Box instance value ID (handle)
        method_name: Method name to call
        args: List of argument value IDs
        dst_vid: Optional destination for return value
        vmap: Value map
        resolver: Optional resolver for type handling
    """
    i64 = ir.IntType(64)
    i8 = ir.IntType(8)
    i8p = i8.as_pointer()

    # Receiver value
    if resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
        recv_val = resolver.resolve_i64(box_vid, builder.block, preds, block_end_values, vmap, bb_map)
    else:
        recv_val = vmap.get(box_vid, ir.Constant(i64, 0))

    # Minimal method bridging for strings and console
    if method_name in ("length", "len"):
        # Prefer handle-based len_h
        recv_h = _ensure_handle(builder, module, recv_val)
        callee = _declare(module, "nyash.string.len_h", i64, [i64])
        result = builder.call(callee, [recv_h], name="strlen_h")
        if dst_vid is not None:
            vmap[dst_vid] = result
        return

    if method_name == "substring":
        # substring(start, end) with pointer-based API
        if resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
            s = resolver.resolve_i64(args[0], builder.block, preds, block_end_values, vmap, bb_map) if args else ir.Constant(i64, 0)
            e = resolver.resolve_i64(args[1], builder.block, preds, block_end_values, vmap, bb_map) if len(args) > 1 else ir.Constant(i64, 0)
        else:
            s = vmap.get(args[0], ir.Constant(i64, 0)) if args else ir.Constant(i64, 0)
            e = vmap.get(args[1], ir.Constant(i64, 0)) if len(args) > 1 else ir.Constant(i64, 0)
        # Coerce recv to i8*
        recv_p = recv_val
        if hasattr(recv_p, 'type') and isinstance(recv_p.type, ir.IntType):
            recv_p = builder.inttoptr(recv_p, i8p, name="bc_i2p_recv")
        elif hasattr(recv_p, 'type') and isinstance(recv_p.type, ir.PointerType):
            try:
                if isinstance(recv_p.type.pointee, ir.ArrayType):
                    c0 = ir.Constant(ir.IntType(32), 0)
                    recv_p = builder.gep(recv_p, [c0, c0], name="bc_gep_recv")
            except Exception:
                pass
        else:
            recv_p = ir.Constant(i8p, None)
        # Coerce indices
        if hasattr(s, 'type') and isinstance(s.type, ir.PointerType):
            s = builder.ptrtoint(s, i64)
        if hasattr(e, 'type') and isinstance(e.type, ir.PointerType):
            e = builder.ptrtoint(e, i64)
        callee = _declare(module, "nyash.string.substring_sii", i8p, [i8p, i64, i64])
        p = builder.call(callee, [recv_p, s, e], name="substring")
        # Return as handle across blocks (i8* -> i64 via nyash.box.from_i8_string)
        conv_fnty = ir.FunctionType(i64, [i8p])
        conv = _declare(module, "nyash.box.from_i8_string", i64, [i8p])
        h = builder.call(conv, [p], name="str_ptr2h_sub")
        if dst_vid is not None:
            vmap[dst_vid] = h
        return

    if method_name == "lastIndexOf":
        # lastIndexOf(needle)
        if resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
            needle = resolver.resolve_ptr(args[0], builder.block, preds, block_end_values, vmap) if args else ir.Constant(i8p, None)
        else:
            needle = vmap.get(args[0], ir.Constant(i8p, None)) if args else ir.Constant(i8p, None)
        recv_p = recv_val
        if hasattr(recv_p, 'type') and isinstance(recv_p.type, ir.IntType):
            recv_p = builder.inttoptr(recv_p, i8p, name="bc_i2p_recv2")
        elif hasattr(recv_p, 'type') and isinstance(recv_p.type, ir.PointerType):
            try:
                if isinstance(recv_p.type.pointee, ir.ArrayType):
                    c0 = ir.Constant(ir.IntType(32), 0)
                    recv_p = builder.gep(recv_p, [c0, c0], name="bc_gep_recv2")
            except Exception:
                pass
        if hasattr(needle, 'type') and isinstance(needle.type, ir.IntType):
            needle = builder.inttoptr(needle, i8p, name="bc_i2p_needle")
        elif hasattr(needle, 'type') and isinstance(needle.type, ir.PointerType):
            try:
                if isinstance(needle.type.pointee, ir.ArrayType):
                    c0 = ir.Constant(ir.IntType(32), 0)
                    needle = builder.gep(needle, [c0, c0], name="bc_gep_needle")
            except Exception:
                pass
        elif not hasattr(needle, 'type'):
            needle = ir.Constant(i8p, None)
        callee = _declare(module, "nyash.string.lastIndexOf_ss", i64, [i8p, i8p])
        res = builder.call(callee, [recv_p, needle], name="lastIndexOf")
        if dst_vid is not None:
            vmap[dst_vid] = res
        return

    if method_name in ("print", "println", "log"):
        # Console mapping
        if resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
            arg0 = resolver.resolve_i64(args[0], builder.block, preds, block_end_values, vmap, bb_map) if args else None
        else:
            arg0 = vmap.get(args[0]) if args else None
        if arg0 is None:
            arg0 = ir.Constant(i8p, None)
        # Prefer handle API if arg is i64, else pointer API
        if hasattr(arg0, 'type') and isinstance(arg0.type, ir.IntType) and arg0.type.width == 64:
            callee = _declare(module, "nyash.console.log_handle", i64, [i64])
            _ = builder.call(callee, [arg0], name="console_log_h")
        else:
            if hasattr(arg0, 'type') and isinstance(arg0.type, ir.IntType):
                arg0 = builder.inttoptr(arg0, i8p)
            callee = _declare(module, "nyash.console.log", i64, [i8p])
            _ = builder.call(callee, [arg0], name="console_log")
        if dst_vid is not None:
            vmap[dst_vid] = ir.Constant(i64, 0)
        return

    # Default: invoke via NyRT by-name shim (runtime resolves method id)
    recv_h = _ensure_handle(builder, module, recv_val)
    # Build C string for method name
    mbytes = (method_name + "\0").encode('utf-8')
    arr_ty = ir.ArrayType(ir.IntType(8), len(mbytes))
    try:
        fn = builder.block.parent
        fn_name = getattr(fn, 'name', 'fn')
    except Exception:
        fn_name = 'fn'
    base = f".meth_{fn_name}_{method_name}"
    existing = {g.name for g in module.global_values}
    gname = base
    k = 1
    while gname in existing:
        gname = f"{base}.{k}"; k += 1
    g = ir.GlobalVariable(module, arr_ty, name=gname)
    g.linkage = 'private'
    g.global_constant = True
    g.initializer = ir.Constant(arr_ty, bytearray(mbytes))
    c0 = ir.Constant(ir.IntType(32), 0)
    mptr = builder.gep(g, [c0, c0], inbounds=True)

    # Up to 2 args for minimal path
    argc = ir.Constant(i64, min(len(args), 2))
    if resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
        a1 = resolver.resolve_i64(args[0], builder.block, preds, block_end_values, vmap, bb_map) if len(args) >= 1 else ir.Constant(i64, 0)
        a2 = resolver.resolve_i64(args[1], builder.block, preds, block_end_values, vmap, bb_map) if len(args) >= 2 else ir.Constant(i64, 0)
    else:
        a1 = vmap.get(args[0], ir.Constant(i64, 0)) if len(args) >= 1 else ir.Constant(i64, 0)
        a2 = vmap.get(args[1], ir.Constant(i64, 0)) if len(args) >= 2 else ir.Constant(i64, 0)
    if hasattr(a1, 'type') and isinstance(a1.type, ir.PointerType):
        a1 = builder.ptrtoint(a1, i64)
    if hasattr(a2, 'type') and isinstance(a2.type, ir.PointerType):
        a2 = builder.ptrtoint(a2, i64)

    callee = _declare(module, "nyash.plugin.invoke_by_name_i64", i64, [i64, i8p, i64, i64, i64])
    result = builder.call(callee, [recv_h, mptr, argc, a1, a2], name="pinvoke_by_name")
    if dst_vid is not None:
        vmap[dst_vid] = result
