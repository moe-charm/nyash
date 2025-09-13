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
        # Any.length_h: Array/String/Map に対応
        recv_h = _ensure_handle(builder, module, recv_val)
        callee = _declare(module, "nyash.any.length_h", i64, [i64])
        result = builder.call(callee, [recv_h], name="any_length_h")
        if dst_vid is not None:
            vmap[dst_vid] = result
        return

    if method_name == "substring":
        # substring(start, end)
        # If receiver is a handle (i64), use handle-based helper; else pointer-based API
        if resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
            s = resolver.resolve_i64(args[0], builder.block, preds, block_end_values, vmap, bb_map) if args else ir.Constant(i64, 0)
            e = resolver.resolve_i64(args[1], builder.block, preds, block_end_values, vmap, bb_map) if len(args) > 1 else ir.Constant(i64, 0)
        else:
            s = vmap.get(args[0], ir.Constant(i64, 0)) if args else ir.Constant(i64, 0)
            e = vmap.get(args[1], ir.Constant(i64, 0)) if len(args) > 1 else ir.Constant(i64, 0)
        if hasattr(recv_val, 'type') and isinstance(recv_val.type, ir.IntType):
            # handle-based
            callee = _declare(module, "nyash.string.substring_hii", i64, [i64, i64, i64])
            h = builder.call(callee, [recv_val, s, e], name="substring_h")
            if dst_vid is not None:
                vmap[dst_vid] = h
                try:
                    if resolver is not None and hasattr(resolver, 'mark_string'):
                        resolver.mark_string(dst_vid)
                except Exception:
                    pass
            return
        else:
            # pointer-based
            recv_p = recv_val
            if hasattr(recv_p, 'type') and isinstance(recv_p.type, ir.PointerType):
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
            conv = _declare(module, "nyash.box.from_i8_string", i64, [i8p])
            h = builder.call(conv, [p], name="str_ptr2h_sub")
            if dst_vid is not None:
                vmap[dst_vid] = h
                try:
                    if resolver is not None and hasattr(resolver, 'mark_string'):
                        resolver.mark_string(dst_vid)
                    if resolver is not None and hasattr(resolver, 'string_ptrs'):
                        resolver.string_ptrs[int(dst_vid)] = p
                except Exception:
                    pass
            return

    if method_name == "lastIndexOf":
        # lastIndexOf(needle)
        if resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
            n_i64 = resolver.resolve_i64(args[0], builder.block, preds, block_end_values, vmap, bb_map) if args else ir.Constant(i64, 0)
        else:
            n_i64 = vmap.get(args[0], ir.Constant(i64, 0)) if args else ir.Constant(i64, 0)
        if hasattr(recv_val, 'type') and isinstance(recv_val.type, ir.IntType):
            # handle-based
            callee = _declare(module, "nyash.string.lastIndexOf_hh", i64, [i64, i64])
            res = builder.call(callee, [recv_val, n_i64], name="lastIndexOf_hh")
            if dst_vid is not None:
                vmap[dst_vid] = res
            return
        else:
            # pointer-based
            recv_p = recv_val
            if hasattr(recv_p, 'type') and isinstance(recv_p.type, ir.PointerType):
                try:
                    if isinstance(recv_p.type.pointee, ir.ArrayType):
                        c0 = ir.Constant(ir.IntType(32), 0)
                        recv_p = builder.gep(recv_p, [c0, c0], name="bc_gep_recv2")
                except Exception:
                    pass
            else:
                recv_p = ir.Constant(i8p, None)
            needle = n_i64
            if hasattr(needle, 'type') and isinstance(needle.type, ir.IntType):
                needle = builder.inttoptr(needle, i8p, name="bc_i2p_needle")
            elif hasattr(needle, 'type') and isinstance(needle.type, ir.PointerType):
                try:
                    if isinstance(needle.type.pointee, ir.ArrayType):
                        c0 = ir.Constant(ir.IntType(32), 0)
                        needle = builder.gep(needle, [c0, c0], name="bc_gep_needle")
                except Exception:
                    pass
            callee = _declare(module, "nyash.string.lastIndexOf_ss", i64, [i8p, i8p])
            res = builder.call(callee, [recv_p, needle], name="lastIndexOf")
            if dst_vid is not None:
                vmap[dst_vid] = res
            return

    if method_name == "get":
        # ArrayBox.get(index) → nyash.array.get_h(handle, idx)
        recv_h = _ensure_handle(builder, module, recv_val)
        if resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
            idx = resolver.resolve_i64(args[0], builder.block, preds, block_end_values, vmap, bb_map) if args else ir.Constant(i64, 0)
        else:
            idx = vmap.get(args[0], ir.Constant(i64, 0)) if args else ir.Constant(i64, 0)
        callee = _declare(module, "nyash.array.get_h", i64, [i64, i64])
        res = builder.call(callee, [recv_h, idx], name="arr_get_h")
        if dst_vid is not None:
            vmap[dst_vid] = res
            try:
                if resolver is not None and hasattr(resolver, 'mark_string'):
                    # Heuristic: args array often stores strings for CLI; tag as string-ish
                    resolver.mark_string(dst_vid)
            except Exception:
                pass
        return

    if method_name in ("print", "println", "log"):
        # Console mapping (prefer pointer-API when possible to avoid handle registry mismatch)
        use_ptr = False
        arg0_vid = args[0] if args else None
        arg0_ptr = None
        if resolver is not None and hasattr(resolver, 'string_ptrs') and arg0_vid is not None:
            try:
                arg0_ptr = resolver.string_ptrs.get(int(arg0_vid))
                if arg0_ptr is not None:
                    use_ptr = True
            except Exception:
                pass
        if use_ptr and arg0_ptr is not None:
            callee = _declare(module, "nyash.console.log", i64, [i8p])
            _ = builder.call(callee, [arg0_ptr], name="console_log_ptr")
        else:
            # Fallback: resolve i64 and prefer pointer API via to_i8p_h bridge
            if resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
                arg0 = resolver.resolve_i64(args[0], builder.block, preds, block_end_values, vmap, bb_map) if args else None
            else:
                arg0 = vmap.get(args[0]) if args else None
            if arg0 is None:
                arg0 = ir.Constant(i64, 0)
            # If we have a handle (i64), convert to i8* via bridge and log via pointer API
            if hasattr(arg0, 'type') and isinstance(arg0.type, ir.IntType):
                if arg0.type.width != 64:
                    arg0 = builder.zext(arg0, i64)
                bridge = _declare(module, "nyash.string.to_i8p_h", i8p, [i64])
                p = builder.call(bridge, [arg0], name="str_h2p_for_log")
                callee = _declare(module, "nyash.console.log", i64, [i8p])
                _ = builder.call(callee, [p], name="console_log_p")
            else:
                # Non-integer value: coerce to i8* and log
                if hasattr(arg0, 'type') and isinstance(arg0.type, ir.IntType):
                    arg0 = builder.inttoptr(arg0, i8p)
                callee = _declare(module, "nyash.console.log", i64, [i8p])
                _ = builder.call(callee, [arg0], name="console_log")
        if dst_vid is not None:
            vmap[dst_vid] = ir.Constant(i64, 0)
        return

    # Special: method on `me` (self) or static dispatch to Main.* → direct call to `Main.method/arity`
    try:
        cur_fn_name = str(builder.block.parent.name)
    except Exception:
        cur_fn_name = ''
    # Heuristic: MIR encodes `me` as a string literal "__me__" or sometimes value-id 0.
    is_me = False
    try:
        if box_vid == 0:
            is_me = True
        # Prefer literal marker captured by resolver (from const lowering)
        elif resolver is not None and hasattr(resolver, 'string_literals'):
            lit = resolver.string_literals.get(box_vid)
            if lit == "__me__":
                is_me = True
    except Exception:
        pass
    if is_me and cur_fn_name.startswith('Main.'):
        # Build target function name with arity
        arity = len(args)
        target = f"Main.{method_name}/{arity}"
        # If module already has such function, prefer direct call
        callee = None
        for f in module.functions:
            if f.name == target:
                callee = f
                break
        if callee is not None:
            a = []
            for i, aid in enumerate(args):
                raw = vmap.get(aid)
                if raw is not None and hasattr(raw, 'type') and isinstance(raw.type, ir.PointerType):
                    aval = _ensure_handle(builder, module, raw)
                else:
                    if resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
                        aval = resolver.resolve_i64(aid, builder.block, preds, block_end_values, vmap, bb_map)
                    else:
                        aval = vmap.get(aid, ir.Constant(ir.IntType(64), 0))
                    if hasattr(aval, 'type') and isinstance(aval.type, ir.PointerType):
                        aval = _ensure_handle(builder, module, aval)
                    elif hasattr(aval, 'type') and isinstance(aval.type, ir.IntType) and aval.type.width != 64:
                        aval = builder.zext(aval, ir.IntType(64)) if aval.type.width < 64 else builder.trunc(aval, ir.IntType(64))
                a.append(aval)
            res = builder.call(callee, a, name=f"call_self_{method_name}")
            if dst_vid is not None:
                vmap[dst_vid] = res
                try:
                    if method_name in ("esc_json", "node_json", "dirname", "join", "read_all") and resolver is not None and hasattr(resolver, 'mark_string'):
                        resolver.mark_string(dst_vid)
                except Exception:
                    pass
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
    # Compute GEP in the current block so it is naturally ordered before the call
    # Use constant GEP so we don't depend on instruction ordering
    mptr = ir.Constant.gep(g, (c0, c0))

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
        # Heuristic tagging: common plugin methods returning strings
        try:
            if resolver is not None and hasattr(resolver, 'mark_string') and method_name in ("read", "dirname", "join"):
                resolver.mark_string(dst_vid)
        except Exception:
            pass
