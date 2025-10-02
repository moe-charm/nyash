"""
BoxCall instruction lowering
Core of Nyash's "Everything is Box" philosophy
"""

import llvmlite.ir as ir
from typing import Dict, List, Optional, Any
from instructions.safepoint import insert_automatic_safepoint
from dispatch import PhiDispatchPoint
from dispatch.type_coercion import TypeCoercion

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
            # extend/trunc to i64 (TypeCoercion統一)
            return TypeCoercion.to_i64(builder, v, "box_arg")
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
    bb_map=None,
    ctx: Optional[Any] = None,
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
    # Insert a safepoint around potential heavy boxcall sites (pre-call)
    try:
        import os
        if os.environ.get('NYASH_LLVM_AUTO_SAFEPOINT', '1') == '1':
            insert_automatic_safepoint(builder, module, "boxcall")
    except Exception:
        pass

    # Short-hands with ctx (backward-compatible fallback)
    r = resolver
    p = preds
    bev = block_end_values
    bbm = bb_map
    if ctx is not None:
        try:
            r = getattr(ctx, 'resolver', r)
            p = getattr(ctx, 'preds', p)
            bev = getattr(ctx, 'block_end_values', bev)
            bbm = getattr(ctx, 'bb_map', bbm)
        except Exception:
            pass
    def _res_i64(vid: int):
        # Unified dispatch resolution (strict→declared PHI→last add→coerce)
        try:
            if r is not None and p is not None and bev is not None and bbm is not None:
                return PhiDispatchPoint.resolve_i64(builder, r, int(vid), builder.block, p, bev, vmap, bbm)
        except Exception:
            pass
        # Fallback: local/global map
        v = vmap.get(vid)
        if v is None and r is not None and hasattr(r, 'global_vmap') and isinstance(r.global_vmap, dict):
            v = r.global_vmap.get(vid)
        return v

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
    # Receiver value
    recv_val = _res_i64(box_vid)
    if recv_val is None:
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

    if method_name == "size":
        # Map/Array size via any.length_h
        recv_h = _ensure_handle(builder, module, recv_val)
        callee = _declare(module, "nyash.any.length_h", i64, [i64])
        result = builder.call(callee, [recv_h], name="any_size_h")
        if dst_vid is not None:
            vmap[dst_vid] = result
        return

    if method_name == "substring":
        # substring(start, end)
        # If receiver is a handle (i64), use handle-based helper; else pointer-based API
        s = _res_i64(args[0]) if args else ir.Constant(i64, 0)
        if s is None:
            s = vmap.get(args[0], ir.Constant(i64, 0)) if args else ir.Constant(i64, 0)
        e = _res_i64(args[1]) if len(args) > 1 else ir.Constant(i64, 0)
        if e is None:
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
            # Coerce indices (TypeCoercion統一)
            s = TypeCoercion.to_i64(builder, s, "substr_start")
            e = TypeCoercion.to_i64(builder, e, "substr_end")
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
        # Unified get for Array/Map: try Array.get_h first, then Map.get_hh; choose non-zero
        recv_h = _ensure_handle(builder, module, recv_val)
        k = _res_i64(args[0]) if args else ir.Constant(i64, 0)
        if k is None:
            k = vmap.get(args[0], ir.Constant(i64, 0)) if args else ir.Constant(i64, 0)
        # Normalize key to i64 (TypeCoercion統一)
        k = TypeCoercion.to_i64(builder, k, "get_key")
        # Attempt Array.get_h(handle, idx)
        callee_arr = _declare(module, "nyash.array.get_h", i64, [i64, i64])
        v_arr = builder.call(callee_arr, [recv_h, k], name="arr_get_h")
        # Fallback: Map.get_hh(handle, key_any)
        callee_map = _declare(module, "nyash.map.get_hh", i64, [i64, i64])
        v_map = builder.call(callee_map, [recv_h, k], name="map_get_hh")
        # Select non-zero result: (v_arr != 0) ? v_arr : v_map
        i1 = ir.IntType(1)
        cond = builder.icmp_signed("!=", v_arr, ir.Constant(i64, 0), name="is_arr_nonzero")
        res = builder.select(cond, v_arr, v_map, name="get_unified")
        if dst_vid is not None:
            vmap[dst_vid] = res
        return

    if method_name == "push":
        # ArrayBox.push(val) → nyash.array.push_h(handle, val)
        recv_h = _ensure_handle(builder, module, recv_val)
        v0 = _res_i64(args[0]) if args else ir.Constant(i64, 0)
        if v0 is None:
            v0 = vmap.get(args[0], ir.Constant(i64, 0)) if args else ir.Constant(i64, 0)
        # Fallback coercion: pointer → handle, int → i64 width
        if hasattr(v0, 'type') and isinstance(v0.type, ir.PointerType):
            v0 = _ensure_handle(builder, module, v0)
        elif hasattr(v0, 'type') and isinstance(v0.type, ir.IntType) and v0.type.width != 64:
            v0 = TypeCoercion.to_i64(builder, v0, "push_val")
        callee = _declare(module, "nyash.array.push_h", i64, [i64, i64])
        res = builder.call(callee, [recv_h, v0], name="arr_push_h")
        if dst_vid is not None:
            vmap[dst_vid] = res
        return

    if method_name == "set":
        # MapBox.set(key, val) → nyash.map.set_hh(handle, key_any, val_any)
        recv_h = _ensure_handle(builder, module, recv_val)
        k = _res_i64(args[0]) if len(args) > 0 else ir.Constant(i64, 0)
        if k is None:
            k = vmap.get(args[0], ir.Constant(i64, 0)) if len(args) > 0 else ir.Constant(i64, 0)
        v = _res_i64(args[1]) if len(args) > 1 else ir.Constant(i64, 0)
        if v is None:
            v = vmap.get(args[1], ir.Constant(i64, 0)) if len(args) > 1 else ir.Constant(i64, 0)
        # Fallback coercion: pointer → handle, int → i64 width
        if hasattr(k, 'type') and isinstance(k.type, ir.PointerType):
            k = _ensure_handle(builder, module, k)
        else:
            k = TypeCoercion.to_i64(builder, k, "set_key")
        if hasattr(v, 'type') and isinstance(v.type, ir.PointerType):
            v = _ensure_handle(builder, module, v)
        else:
            v = TypeCoercion.to_i64(builder, v, "set_val")
        callee = _declare(module, "nyash.map.set_hh", i64, [i64, i64, i64])
        res = builder.call(callee, [recv_h, k, v], name="map_set_hh")
        if dst_vid is not None:
            vmap[dst_vid] = res
        return

    if method_name == "has":
        # MapBox.has(key) → nyash.map.has_hh(handle, key_any)
        recv_h = _ensure_handle(builder, module, recv_val)
        k = _res_i64(args[0]) if args else ir.Constant(i64, 0)
        if k is None:
            k = vmap.get(args[0], ir.Constant(i64, 0)) if args else ir.Constant(i64, 0)
        # Fallback coercion: pointer → handle, int → i64 width
        if hasattr(k, 'type') and isinstance(k.type, ir.PointerType):
            k = _ensure_handle(builder, module, k)
        else:
            k = TypeCoercion.to_i64(builder, k, "has_key")
        callee = _declare(module, "nyash.map.has_hh", i64, [i64, i64])
        res = builder.call(callee, [recv_h, k], name="map_has_hh")
        if dst_vid is not None:
            vmap[dst_vid] = res
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
            # Fallback: prefer raw vmap value; resolve only if missing (avoid synthesizing PHIs here)
            arg0 = vmap.get(args[0]) if args else None
            if arg0 is None and resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
                arg0 = resolver.resolve_i64(args[0], builder.block, preds, block_end_values, vmap, bb_map)
            if arg0 is None:
                arg0 = ir.Constant(i64, 0)
            # If we have a handle (i64), convert to i8* via bridge and log via pointer API
            if hasattr(arg0, 'type') and isinstance(arg0.type, ir.IntType):
                arg0 = TypeCoercion.to_i64(builder, arg0, "log_arg")
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
                    else:
                        aval = TypeCoercion.to_i64(builder, aval, f"call_arg{idx}")
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
    # Normalize arguments to i64
    a1 = TypeCoercion.to_i64(builder, a1, "invoke_arg1")
    a2 = TypeCoercion.to_i64(builder, a2, "invoke_arg2")

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
