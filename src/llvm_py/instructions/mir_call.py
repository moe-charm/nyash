"""
Unified MIR Call instruction handler - ChatGPT5 Pro A++ Design
Replaces call.py, boxcall.py, externcall.py, newbox.py, plugin_invoke.py, newclosure.py
"""

from typing import Dict, Any, Optional
from llvmlite import ir
import os
import json

def lower_mir_call(owner, builder: ir.IRBuilder, mir_call: Dict[str, Any], dst_vid: Optional[int], vmap: Dict, resolver):
    """
    Lower unified MirCall instruction.

    Parameters:
    - owner: NyashLLVMBuilder instance
    - builder: LLVM IR builder
    - mir_call: MirCall dict containing 'callee', 'args', 'flags', 'effects'
    - dst_vid: Optional destination register
    - vmap: Value mapping dict
    - resolver: Value resolver instance
    """
    # Check if unified call is enabled
    use_unified = os.getenv("NYASH_MIR_UNIFIED_CALL", "1").lower() not in ("0", "false", "off")
    if not use_unified:
        # Fall back to legacy dispatching
        return lower_legacy_call(owner, builder, mir_call, dst_vid, vmap, resolver)

    callee = mir_call.get("callee", {})
    args = mir_call.get("args", [])
    flags = mir_call.get("flags", {})
    effects = mir_call.get("effects", {})

    # Parse callee type
    callee_type = callee.get("type")

    # Optional trace: dump callee info (including certainty for Method)
    if os.getenv('NYASH_LLVM_TRACE_CALLS') == '1':
        try:
            evt = { 'type': callee_type }
            if callee_type == 'Global':
                evt.update({'name': callee.get('name')})
            elif callee_type == 'Method':
                evt.update({
                    'box_name': callee.get('box_name'),
                    'method': callee.get('method'),
                    'receiver': callee.get('receiver'),
                    'certainty': callee.get('certainty'),
                })
            elif callee_type == 'Extern':
                evt.update({'name': callee.get('name')})
            print(json.dumps({'phase':'llvm','cat':'mir_call','event':evt}))
        except Exception:
            pass

    # Decide safepoint policy (dev-flag + effects-aware)
    def _allow_safepoint() -> bool:
        auto = os.getenv('NYASH_LLVM_AUTO_SAFEPOINT', '0') not in ('0','false','off')
        eff = effects or {}
        if os.getenv('NYASH_LLVM_SAFEPOINT_READONLY_SKIP', '1') in ('1','true','on'):
            try:
                is_read = bool(eff.get('read', False))
                has_other = any(bool(eff.get(k, False)) for k in ('write','alloc','io'))
                if is_read and not has_other:
                    return False and auto
            except Exception:
                pass
        return auto

    if callee_type == "Global":
        # Global function call (e.g., print, panic)
        func_name = callee.get("name")
        lower_global_call(builder, owner.module, func_name, args, dst_vid, vmap, resolver, owner, _allow_safepoint())

    elif callee_type == "ModuleFunction":
        # Module function call (static box methods) - Phase 15.7
        func_name = callee.get("name")
        lower_global_call(builder, owner.module, func_name, args, dst_vid, vmap, resolver, owner, _allow_safepoint())

    elif callee_type == "Method":
        # Box method call
        box_name = callee.get("box_name")
        method = callee.get("method")
        receiver = callee.get("receiver")
        lower_method_call(builder, owner.module, box_name, method, receiver, args, dst_vid, vmap, resolver, owner, _allow_safepoint())

    elif callee_type == "Constructor":
        # Box constructor (NewBox)
        box_type = callee.get("box_type")
        lower_constructor_call(builder, owner.module, box_type, args, dst_vid, vmap, resolver, owner, _allow_safepoint())

    elif callee_type == "Closure":
        # Closure creation (NewClosure)
        params = callee.get("params", [])
        captures = callee.get("captures", [])
        me_capture = callee.get("me_capture")
        lower_closure_creation(builder, owner.module, params, captures, me_capture, dst_vid, vmap, resolver, owner)

    elif callee_type == "Value":
        # Dynamic function value call
        func_vid = callee.get("value")
        lower_value_call(builder, owner.module, func_vid, args, dst_vid, vmap, resolver, owner, _allow_safepoint())

    elif callee_type == "Extern":
        # External C ABI function call
        extern_name = callee.get("name")
        lower_extern_call(builder, owner.module, extern_name, args, dst_vid, vmap, resolver, owner, _allow_safepoint())

    else:
        raise ValueError(f"Unknown callee type: {callee_type}")


def lower_legacy_call(owner, builder, mir_call, dst_vid, vmap, resolver):
    """Legacy dispatcher for backward compatibility"""
    # This would dispatch to the old instruction handlers
    # For now, just raise an error if unified is disabled
    raise NotImplementedError("Legacy call dispatch not implemented in mir_call.py")


def lower_global_call(builder, module, func_name, args, dst_vid, vmap, resolver, owner, allow_safepoint: bool = True):
    """Lower global function call - TRUE UNIFIED IMPLEMENTATION"""
    from llvmlite import ir
    from instructions.safepoint import insert_automatic_safepoint
    from call_resolver import CallArgResolverBox, ArgResolveContext
    import os

    # Insert automatic safepoint
    if allow_safepoint and os.environ.get('NYASH_LLVM_AUTO_SAFEPOINT', '1') == '1':
        insert_automatic_safepoint(builder, module, "function_call")

    # NEW: Use CallArgResolverBox (Box Theory: 箱化)
    context = ArgResolveContext(builder=builder, vmap=vmap, resolver=resolver, owner=owner, module=module)
    arg_resolver = CallArgResolverBox(context)

    # Phase 1: self-recursive direct call (env-gated)
    try:
        if os.getenv('NYASH_MIR_SELFREC_DIRECT', '0') in ('1','true','on'):
            cur_name = getattr(owner, 'current_function_name', '')
            if cur_name:
                want_arity = len(args)
                base = func_name.split('/')[0] if '/' in func_name else func_name
                expect_full = func_name if '/' in func_name else f"{func_name}/{want_arity}"
                # Direct when targeting the same function (full or base match)
                if cur_name == func_name or cur_name == expect_full or cur_name.split('/')[0] == base:
                    i64 = ir.IntType(64)
                    target = None
                    for f in module.functions:
                        if f.name == expect_full:
                            target = f
                            break
                    if target is None:
                        fty = ir.FunctionType(i64, [i64] * want_arity)
                        target = ir.Function(module, fty, name=expect_full)
                    # NEW: Use arg_resolver instead of _resolve_arg_i64
                    call_args = [arg_resolver.resolve_i64_or_zero(a) for a in args]
                    # Optional trace
                    if os.getenv('NYASH_MIR_OPTIMIZE_TRACE', '0') in ('1','true','on'):
                        try:
                            print(json.dumps({'phase':'llvm','cat':'opt','kind':'selfrec_direct','func': expect_full}))
                        except Exception:
                            pass
                    result = builder.call(target, call_args, name='direct_self_global')
                    if dst_vid is not None:
                        vmap[dst_vid] = result
                    return
    except Exception:
        pass

    # Special-case: ModuleFunction shims → extern (StringBox.* minimal set)
    # Support both static-call shape (recv,arg) and direct-global arity where applicable.
    try:
        from call_resolver import CallArgResolverBox, ArgResolveContext
        context = ArgResolveContext(builder=builder, vmap=vmap, resolver=resolver, owner=owner, module=module)
        arg_resolver = CallArgResolverBox(context)

        def _declare(name: str, ret, args_types):
            for f in owner.module.functions:
                if f.name == name:
                    return f
            fnty = ir.FunctionType(ret, args_types)
            return ir.Function(owner.module, fnty, name=name)

        # StringBox.indexOf/1 → nyash.string.indexOf_hh(recv_h, needle_h)
        if func_name == 'StringBox.indexOf/1' and len(args) >= 2:
            i64 = ir.IntType(64)
            recv = arg_resolver.resolve_i64_or_zero(args[0])
            needle = arg_resolver.resolve_i64_or_zero(args[1])
            callee = _declare('nyash.string.indexOf_hh', i64, [i64, i64])
            result = builder.call(callee, [recv, needle], name='unified_StringBox_indexOf')
            if dst_vid is not None:
                vmap[dst_vid] = result
            return
        # StringBox.substring/2 → nyash.string.substring_hii(recv_h, start_i, end_i)
        if func_name == 'StringBox.substring/2' and len(args) >= 3:
            i64 = ir.IntType(64)
            recv = arg_resolver.resolve_i64_or_zero(args[0])
            a1 = arg_resolver.resolve_i64_or_zero(args[1])
            a2 = arg_resolver.resolve_i64_or_zero(args[2])
            callee = _declare('nyash.string.substring_hii', i64, [i64, i64, i64])
            result = builder.call(callee, [recv, a1, a2], name='unified_StringBox_substring')
            if dst_vid is not None:
                vmap[dst_vid] = result
            return
    except Exception:
        pass

    # Special mapping for built-in globals via a simple map
    # Normalize names: strip arity suffix ("/N") and take last dotted segment
    base_name = func_name
    try:
        import re as _re
        base_name = _re.sub(r"/\d+$", "", base_name)
    except Exception:
        pass
    tail = base_name.rsplit('.', 1)[-1] if '.' in base_name else base_name
    BUILTIN_GLOBALS = {
        'print': 'nyash.console.log',
        'println': 'nyash.console.log',
        'log': 'nyash.console.log',
    }
    if tail in BUILTIN_GLOBALS:
        from llvmlite import ir
        i64 = ir.IntType(64)
        i8 = ir.IntType(8)
        i8p = i8.as_pointer()
        # Resolve first argument to i64 handle
        if len(args) >= 1:
            vid0 = args[0]
            val0 = resolver.resolve_i64(vid0, builder.block, owner.preds, owner.block_end_values, vmap, owner.bb_map) if resolver else vmap.get(vid0)
        else:
            val0 = ir.Constant(i64, 0)
        if val0 is None:
            val0 = ir.Constant(i64, 0)
        # Declare helpers
        def _declare(name: str, ret, args_types):
            for f in owner.module.functions:
                if f.name == name:
                    return f
            fnty = ir.FunctionType(ret, args_types)
            return ir.Function(owner.module, fnty, name=name)
        to_cstr = _declare('nyash.string.to_i8p_h', i8p, [i64])
        log_fn = _declare('nyash.console.log', i64, [i8p])
        cstr = builder.call(to_cstr, [val0], name='print_to_cstr')
        result = builder.call(log_fn, [cstr], name='print_log')
        if dst_vid is not None:
            vmap[dst_vid] = result
        return
    # Look up function in module
    func = None
    for f in module.functions:
        if f.name == func_name:
            func = f
            break

    if not func:
        # Create function declaration with i64 signature
        ret_type = ir.IntType(64)
        arg_types = [ir.IntType(64)] * len(args)
        func_type = ir.FunctionType(ret_type, arg_types)
        func = ir.Function(module, func_type, name=func_name)

    # Prepare arguments with type conversion (NEW: using arg_resolver)
    call_args = []
    for i, arg_id in enumerate(args):
        arg_val = arg_resolver.resolve_i64_or_zero(arg_id)

        # Type conversion for function signature matching
        if i < len(func.args):
            expected_type = func.args[i].type
            if expected_type.is_pointer and isinstance(arg_val.type, ir.IntType):
                arg_val = builder.inttoptr(arg_val, expected_type, name=f"global_i2p_{i}")
            elif isinstance(expected_type, ir.IntType) and arg_val.type.is_pointer:
                arg_val = builder.ptrtoint(arg_val, expected_type, name=f"global_p2i_{i}")

        call_args.append(arg_val)

    # Make the call - TRUE UNIFIED
    # Special mapping: JSON.stringify/1 → nyash.json.stringify_h
    base_global = func_name.split('/')[0]
    if base_global == 'JSON.stringify':
        i64 = ir.IntType(64)
        target = None
        for f in module.functions:
            if f.name == 'nyash.json.stringify_h':
                target = f
                break
        if target is None:
            target = ir.Function(module, ir.FunctionType(i64, [i64]), name='nyash.json.stringify_h')
        # Allow either direct arity1 or pass first arg only
        arg0 = call_args[0] if len(call_args) >= 1 else ir.Constant(i64, 0)
        result = builder.call(target, [arg0], name='global_json_stringify')
    else:
        result = builder.call(func, call_args, name=f"unified_global_{func_name}")
    # Optional: tail call hint (env or hints)
    try:
        hint_tail = os.getenv('NYASH_LLVM_TAILCALL', '0') in ('1','true','on') or \
                    os.getenv('NYASH_MIR_HINTS', '').lower() in ('tail','all')
        if hint_tail:
            try:
                result.tail = True
            except Exception:
                pass
            if os.getenv('NYASH_MIR_OPTIMIZE_TRACE', '0') in ('1','true','on'):
                try:
                    print(json.dumps({'phase':'llvm','cat':'opt','kind':'tail_call','func': func_name}))
                except Exception:
                    pass
    except Exception:
        pass

    # Store result
    if dst_vid is not None:
        vmap[dst_vid] = result
        # Mark string-producing functions
        if resolver and hasattr(resolver, 'mark_string'):
            if any(key in func_name for key in ['esc_json', 'node_json', 'dirname', 'join', 'read_all', 'toJson']):
                resolver.mark_string(dst_vid)


def lower_method_call(builder, module, box_name, method, receiver, args, dst_vid, vmap, resolver, owner, allow_safepoint: bool = True):
    """Lower box method call - TRUE UNIFIED IMPLEMENTATION"""
    from llvmlite import ir
    from instructions.safepoint import insert_automatic_safepoint
    from call_resolver import CallArgResolverBox, ArgResolveContext
    import os

    i64 = ir.IntType(64)
    i8 = ir.IntType(8)
    i8p = i8.as_pointer()

    # NEW: Use CallArgResolverBox (Box Theory: 箱化)
    context = ArgResolveContext(builder=builder, vmap=vmap, resolver=resolver, owner=owner, module=module)
    arg_resolver = CallArgResolverBox(context)

    # Optional fast-path: direct self-call for methods within the same static box
    # Guarded by env NYASH_LLVM_DIRECT_SELF=1 to allow staged rollout.
    try:
        if os.getenv('NYASH_LLVM_DIRECT_SELF', '0') in ('1','true','on'):
            cur_name = getattr(owner, 'current_function_name', '')
            base = cur_name.split('/') [0] if '/' in cur_name else cur_name
            expect_base = f"{box_name}.{method}"
            if base == expect_base:
                want_arity = 1 + len(args)
                target = None
                for f in module.functions:
                    fname = getattr(f, 'name', '')
                    if fname.startswith(expect_base + '/'):
                        try:
                            ar = int(fname.split('/')[-1])
                        except Exception:
                            ar = -1
                        if ar == want_arity:
                            target = f
                            break
                if target is None:
                    fty = ir.FunctionType(i64, [i64] * want_arity)
                    target = ir.Function(module, fty, name=f"{expect_base}/{want_arity}")
                # NEW: Use arg_resolver
                call_args = [arg_resolver.resolve_i64_or_zero(receiver)]
                call_args.extend([arg_resolver.resolve_i64_or_zero(a) for a in args])
                result = builder.call(target, call_args, name=f"direct_self_{method}")
                if dst_vid is not None:
                    vmap[dst_vid] = result
                return
    except Exception:
        pass

    i8 = ir.IntType(8)
    i8p = i8.as_pointer()

    # Insert automatic safepoint
    if allow_safepoint and os.environ.get('NYASH_LLVM_AUTO_SAFEPOINT', '1') == '1':
        insert_automatic_safepoint(builder, module, "boxcall")

    # Helper to declare function
    def _declare(name: str, ret, args_types):
        for f in module.functions:
            if f.name == name:
                return f
        fnty = ir.FunctionType(ret, args_types)
        return ir.Function(module, fnty, name=name)

    # Helper to ensure i64 handle
    def _ensure_handle(v):
        if isinstance(v.type, ir.IntType) and v.type.width == 64:
            return v
        if v.type.is_pointer:
            callee = _declare("nyash.box.from_i8_string", i64, [i8p])
            return builder.call(callee, [v], name="unified_str_ptr2h")
        if isinstance(v.type, ir.IntType):
            return builder.zext(v, i64) if v.type.width < 64 else builder.trunc(v, i64)
        return v

    # Use CallArgResolverBox for unified argument resolution
    recv_val = arg_resolver.resolve_i64_or_zero(receiver)
    if recv_val is None:
        recv_val = ir.Constant(i64, 0)
    recv_h = _ensure_handle(recv_val)

    # TRUE UNIFIED METHOD DISPATCH - Everything is Box philosophy
    if method in ["length", "len"]:
        callee = _declare("nyash.any.length_h", i64, [i64])
        result = builder.call(callee, [recv_h], name="unified_length")

    elif method == "size":
        callee = _declare("nyash.any.length_h", i64, [i64])
        result = builder.call(callee, [recv_h], name="unified_size")

    elif method == "substring":
        if len(args) >= 2:
            s = arg_resolver.resolve_i64_or_zero(args[0])
            e = arg_resolver.resolve_i64_or_zero(args[1])
            callee = _declare("nyash.string.substring_hii", i64, [i64, i64, i64])
            result = builder.call(callee, [recv_h, s, e], name="unified_substring")
        elif len(args) == 1:
            # substring(start) → substring(start, len(recv))
            s = arg_resolver.resolve_i64_or_zero(args[0])
            len_decl = _declare("nyash.string.len_h", i64, [i64])
            n = builder.call(len_decl, [recv_h], name="str_len")
            callee = _declare("nyash.string.substring_hii", i64, [i64, i64, i64])
            result = builder.call(callee, [recv_h, s, n], name="unified_substring_1")
        else:
            result = recv_h

    elif method == "lastIndexOf":
        if args:
            needle = arg_resolver.resolve_i64_or_zero(args[0])
            needle_h = _ensure_handle(needle)
            callee = _declare("nyash.string.lastIndexOf_hh", i64, [i64, i64])
            result = builder.call(callee, [recv_h, needle_h], name="unified_lastIndexOf")
        else:
            result = ir.Constant(i64, -1)

    elif method == "indexOf":
        if args:
            needle = arg_resolver.resolve_i64_or_zero(args[0])
            needle_h = _ensure_handle(needle)
            callee = _declare("nyash.string.indexOf_hh", i64, [i64, i64])
            result = builder.call(callee, [recv_h, needle_h], name="unified_indexOf")
        else:
            result = ir.Constant(i64, -1)

    elif method == "get":
        if args:
            k = arg_resolver.resolve_i64_or_zero(args[0])
            callee = _declare("nyash.map.get_hh", i64, [i64, i64])
            result = builder.call(callee, [recv_h, k], name="unified_map_get")
        else:
            result = ir.Constant(i64, 0)

    elif method == "push":
        if args:
            v = arg_resolver.resolve_i64_or_zero(args[0])
            callee = _declare("nyash.array.push_h", i64, [i64, i64])
            result = builder.call(callee, [recv_h, v], name="unified_array_push")
        else:
            result = recv_h

    elif method == "set":
        if len(args) >= 2:
            k = arg_resolver.resolve_i64_or_zero(args[0])
            v = arg_resolver.resolve_i64_or_zero(args[1])
            callee = _declare("nyash.map.set_hh", i64, [i64, i64, i64])
            result = builder.call(callee, [recv_h, k, v], name="unified_map_set")
        else:
            result = recv_h

    elif method == "has":
        if args:
            k = arg_resolver.resolve_i64_or_zero(args[0])
            callee = _declare("nyash.map.has_hh", i64, [i64, i64])
            result = builder.call(callee, [recv_h, k], name="unified_map_has")
        else:
            result = ir.Constant(i64, 0)

    elif method == "log":
        if args:
            arg0 = arg_resolver.resolve_i64_or_zero(args[0])
            if isinstance(arg0.type, ir.IntType) and arg0.type.width == 64:
                bridge = _declare("nyash.string.to_i8p_h", i8p, [i64])
                p = builder.call(bridge, [arg0], name="unified_str_h2p")
                callee = _declare("nyash.console.log", i64, [i8p])
                result = builder.call(callee, [p], name="unified_console_log")
            else:
                callee = _declare("nyash.console.log", i64, [i8p])
                result = builder.call(callee, [arg0], name="unified_console_log")
        else:
            result = ir.Constant(i64, 0)

    else:
        # Generic plugin method invocation
        method_str = method.encode('utf-8') + b'\0'
        method_global = ir.GlobalVariable(module, ir.ArrayType(i8, len(method_str)), name=f"unified_method_{method}")
        method_global.initializer = ir.Constant(ir.ArrayType(i8, len(method_str)), bytearray(method_str))
        method_global.global_constant = True
        mptr = builder.gep(method_global, [ir.Constant(ir.IntType(32), 0), ir.Constant(ir.IntType(32), 0)])

        argc = ir.Constant(i64, len(args))
        a1 = arg_resolver.resolve_i64_or_zero(args[0]) if args else ir.Constant(i64, 0)
        a2 = arg_resolver.resolve_i64_or_zero(args[1]) if len(args) > 1 else ir.Constant(i64, 0)

        callee = _declare("nyash.plugin.invoke_by_name_i64", i64, [i64, i8p, i64, i64, i64])
        result = builder.call(callee, [recv_h, mptr, argc, a1, a2], name="unified_plugin_invoke")

    # Optional: tail call hint (env or hints)
    try:
        hint_tail = os.getenv('NYASH_LLVM_TAILCALL', '0') in ('1','true','on') or \
                    os.getenv('NYASH_MIR_HINTS', '').lower() in ('tail','all')
        if hint_tail:
            try:
                result.tail = True
            except Exception:
                pass
            if os.getenv('NYASH_MIR_OPTIMIZE_TRACE', '0') in ('1','true','on'):
                try:
                    print(json.dumps({'phase':'llvm','cat':'opt','kind':'tail_call','method': method}))
                except Exception:
                    pass
    except Exception:
        pass

    # Store result
    if dst_vid is not None:
        vmap[dst_vid] = result
        # Mark string-producing methods
        if resolver and hasattr(resolver, 'mark_string'):
            if method in ['substring', 'esc_json', 'node_json', 'dirname', 'join', 'read_all', 'toJson']:
                resolver.mark_string(dst_vid)


def lower_constructor_call(builder, module, box_type, args, dst_vid, vmap, resolver, owner, allow_safepoint: bool = True):
    """Lower box constructor - TRUE UNIFIED IMPLEMENTATION"""
    from llvmlite import ir
    from instructions.safepoint import insert_automatic_safepoint
    from call_resolver import CallArgResolverBox, ArgResolveContext
    import os

    # Insert automatic safepoint for constructor
    if allow_safepoint and os.environ.get('NYASH_LLVM_AUTO_SAFEPOINT', '1') == '1':
        insert_automatic_safepoint(builder, module, 'constructor')

    i64 = ir.IntType(64)
    i8 = ir.IntType(8)
    i8p = i8.as_pointer()

    # Use CallArgResolverBox for unified argument resolution
    context = ArgResolveContext(builder=builder, vmap=vmap, resolver=resolver, owner=owner, module=module)
    arg_resolver = CallArgResolverBox(context)

    # Helper to declare function
    def _declare(name: str, ret, args_types):
        for f in module.functions:
            if f.name == name:
                return f
        fnty = ir.FunctionType(ret, args_types)
        return ir.Function(module, fnty, name=name)

    # TRUE UNIFIED CONSTRUCTOR DISPATCH
    if box_type == "StringBox":
        if args and len(args) > 0:
            # String constructor with initial value
            arg0 = arg_resolver.resolve_i64_or_zero(args[0])
            if arg0 and isinstance(arg0.type, ir.IntType) and arg0.type.width == 64:
                # Already a handle, return as-is
                result = arg0
            elif arg0 and arg0.type.is_pointer:
                # Convert i8* to string handle
                callee = _declare("nyash.box.from_i8_string", i64, [i8p])
                result = builder.call(callee, [arg0], name="unified_str_new")
            else:
                # Create empty string
                callee = _declare("nyash.string.new", i64, [])
                result = builder.call(callee, [], name="unified_str_empty")
        else:
            # Empty string constructor
            callee = _declare("nyash.string.new", i64, [])
            result = builder.call(callee, [], name="unified_str_empty")

    elif box_type == "ArrayBox":
        callee = _declare("nyash.array.new", i64, [])
        result = builder.call(callee, [], name="unified_arr_new")

    elif box_type == "MapBox":
        callee = _declare("nyash.map.new", i64, [])
        result = builder.call(callee, [], name="unified_map_new")

    elif box_type == "IntegerBox":
        if args and len(args) > 0:
            arg0 = arg_resolver.resolve_i64_or_zero(args[0])
            callee = _declare("nyash.integer.new", i64, [i64])
            result = builder.call(callee, [arg0], name="unified_int_new")
        else:
            callee = _declare("nyash.integer.new", i64, [i64])
            result = builder.call(callee, [ir.Constant(i64, 0)], name="unified_int_zero")

    elif box_type == "BoolBox":
        if args and len(args) > 0:
            arg0 = arg_resolver.resolve_i64_or_zero(args[0])
            callee = _declare("nyash.bool.new", i64, [i64])
            result = builder.call(callee, [arg0], name="unified_bool_new")
        else:
            callee = _declare("nyash.bool.new", i64, [i64])
            result = builder.call(callee, [ir.Constant(i64, 0)], name="unified_bool_false")

    else:
        # Generic box constructor or plugin box
        constructor_name = f"nyash.{box_type.lower()}.new"
        if args:
            arg_vals = [arg_resolver.resolve_i64_or_zero(arg_id) for arg_id in args]
            arg_types = [i64] * len(arg_vals)
            callee = _declare(constructor_name, i64, arg_types)
            result = builder.call(callee, arg_vals, name=f"unified_{box_type.lower()}_new")
        else:
            callee = _declare(constructor_name, i64, [])
            result = builder.call(callee, [], name=f"unified_{box_type.lower()}_new")

    # Store result
    if dst_vid is not None:
        vmap[dst_vid] = result


def lower_closure_creation(builder, module, params, captures, me_capture, dst_vid, vmap, resolver, owner):
    """Lower closure creation - TRUE UNIFIED IMPLEMENTATION"""
    from llvmlite import ir
    from call_resolver import CallArgResolverBox, ArgResolveContext

    i64 = ir.IntType(64)
    i8 = ir.IntType(8)
    i8p = i8.as_pointer()

    # Use CallArgResolverBox for unified argument resolution
    context = ArgResolveContext(builder=builder, vmap=vmap, resolver=resolver, owner=owner, module=module)
    arg_resolver = CallArgResolverBox(context)

    # Helper to declare function
    def _declare(name: str, ret, args_types):
        for f in module.functions:
            if f.name == name:
                return f
        fnty = ir.FunctionType(ret, args_types)
        return ir.Function(module, fnty, name=name)

    # Create closure metadata structure
    num_captures = len(captures) if captures else 0
    num_params = len(params) if params else 0

    # Resolve captured values
    capture_vals = []
    if captures:
        for capture in captures:
            if isinstance(capture, dict) and 'id' in capture:
                cap_val = arg_resolver.resolve_i64_or_zero(capture['id'])
            elif isinstance(capture, int):
                cap_val = arg_resolver.resolve_i64_or_zero(capture)
            else:
                cap_val = ir.Constant(i64, 0)
            capture_vals.append(cap_val)

    # Add me_capture if present
    if me_capture is not None:
        me_val = arg_resolver.resolve_i64_or_zero(me_capture) if isinstance(me_capture, int) else ir.Constant(i64, 0)
        capture_vals.append(me_val)
        num_captures += 1

    # Call closure creation function
    if num_captures > 0:
        # Closure with captures
        callee = _declare("nyash.closure.new_with_captures", i64, [i64, i64] + [i64] * num_captures)
        args = [ir.Constant(i64, num_params), ir.Constant(i64, num_captures)] + capture_vals
        result = builder.call(callee, args, name="unified_closure_with_captures")
    else:
        # Simple closure without captures
        callee = _declare("nyash.closure.new", i64, [i64])
        result = builder.call(callee, [ir.Constant(i64, num_params)], name="unified_closure_simple")

    # Store result
    if dst_vid is not None:
        vmap[dst_vid] = result


def lower_value_call(builder, module, func_vid, args, dst_vid, vmap, resolver, owner, allow_safepoint: bool = True):
    """Lower dynamic function value call - TRUE UNIFIED IMPLEMENTATION"""
    from llvmlite import ir
    from instructions.safepoint import insert_automatic_safepoint
    from call_resolver import CallArgResolverBox, ArgResolveContext
    import os

    i64 = ir.IntType(64)
    i8 = ir.IntType(8)
    i8p = i8.as_pointer()

    # Insert automatic safepoint for dynamic value call
    if allow_safepoint and os.environ.get('NYASH_LLVM_AUTO_SAFEPOINT', '1') == '1':
        insert_automatic_safepoint(builder, module, 'value_call')

    # Use CallArgResolverBox for unified argument resolution
    context = ArgResolveContext(builder=builder, vmap=vmap, resolver=resolver, owner=owner, module=module)
    arg_resolver = CallArgResolverBox(context)

    # Helper to declare function
    def _declare(name: str, ret, args_types):
        for f in module.functions:
            if f.name == name:
                return f
        fnty = ir.FunctionType(ret, args_types)
        return ir.Function(module, fnty, name=name)

    # Resolve the function value (handle to function or closure)
    func_val = arg_resolver.resolve_i64_or_zero(func_vid)

    # Resolve arguments
    arg_vals = []
    for arg_id in args:
        arg_val = arg_resolver.resolve_i64_or_zero(arg_id)
        arg_vals.append(arg_val)

    # Dynamic dispatch based on function value type
    # This could be a function handle, closure handle, or method handle

    if len(arg_vals) == 0:
        # No arguments - simple function call
        callee = _declare("nyash.dynamic.call_0", i64, [i64])
        result = builder.call(callee, [func_val], name="unified_dynamic_call_0")

    elif len(arg_vals) == 1:
        # One argument
        callee = _declare("nyash.dynamic.call_1", i64, [i64, i64])
        result = builder.call(callee, [func_val, arg_vals[0]], name="unified_dynamic_call_1")

    elif len(arg_vals) == 2:
        # Two arguments
        callee = _declare("nyash.dynamic.call_2", i64, [i64, i64, i64])
        result = builder.call(callee, [func_val, arg_vals[0], arg_vals[1]], name="unified_dynamic_call_2")

    else:
        # Generic variadic call
        argc = ir.Constant(i64, len(arg_vals))

        # Create argument array for variadic call
        if len(arg_vals) <= 4:
            # Use direct argument passing for small argument lists
            arg_types = [i64] * (2 + len(arg_vals))  # func_val, argc, ...args
            callee = _declare("nyash.dynamic.call_n", i64, arg_types)
            call_args = [func_val, argc] + arg_vals
            result = builder.call(callee, call_args, name="unified_dynamic_call_n")
        else:
            # For large argument lists, use array-based approach
            callee = _declare("nyash.dynamic.call_array", i64, [i64, i64, i8p])

            # Create temporary array for arguments
            array_type = ir.ArrayType(i64, len(arg_vals))
            array_alloca = builder.alloca(array_type, name="unified_arg_array")

            # Store arguments in array
            for i, arg_val in enumerate(arg_vals):
                gep = builder.gep(array_alloca, [ir.Constant(ir.IntType(32), 0), ir.Constant(ir.IntType(32), i)])
                builder.store(arg_val, gep)

            # Cast array to i8*
            array_ptr = builder.bitcast(array_alloca, i8p, name="unified_arg_array_ptr")
            result = builder.call(callee, [func_val, argc, array_ptr], name="unified_dynamic_call_array")

    # Store result
    if dst_vid is not None:
        vmap[dst_vid] = result


def lower_extern_call(builder, module, extern_name, args, dst_vid, vmap, resolver, owner, allow_safepoint: bool = True):
    """Lower external C ABI call - TRUE UNIFIED IMPLEMENTATION"""
    from llvmlite import ir
    from instructions.safepoint import insert_automatic_safepoint
    from call_resolver import CallArgResolverBox, TypeCoercerBox, ArgResolveContext
    import os

    i64 = ir.IntType(64)
    i8 = ir.IntType(8)
    i8p = i8.as_pointer()

    # Insert automatic safepoint
    if allow_safepoint and os.environ.get('NYASH_LLVM_AUTO_SAFEPOINT', '1') == '1':
        insert_automatic_safepoint(builder, module, "externcall")

    # NEW: Use CallArgResolverBox (Box Theory: 箱化)
    # This replaces 47 lines of duplicated _resolve_arg() with 4 lines
    context = ArgResolveContext(builder=builder, vmap=vmap, resolver=resolver, owner=owner, module=module)
    arg_resolver = CallArgResolverBox(context)
    type_coercer = TypeCoercerBox(builder, module)

    # Special-case: inline nyrt.ops.op_eq to avoid requiring a kernel symbol.
    # Semantics: compare as i64 values (handle identity for boxes), return i64 0|1.
    # Note: deep user-defined equals() is VM-only; AOT/harness keeps minimal parity.
    if extern_name == "nyrt.ops.op_eq":
        # Resolve and coerce arguments (NEW: unified via TypeCoercerBox)
        a = type_coercer.to_i64_handle(arg_resolver.resolve_i64_or_zero(args[0])) if args else ir.Constant(i64, 0)
        b = type_coercer.to_i64_handle(arg_resolver.resolve_i64_or_zero(args[1])) if len(args) > 1 else ir.Constant(i64, 0)

        cmpv = builder.icmp_signed('==', a, b, name='op_eq_cmp')
        res = builder.zext(cmpv, i64, name='op_eq_zext_i1')
        if dst_vid is not None:
            vmap[dst_vid] = res
        return

    # Bridge: JSON.stringify/1 → nyash.json.stringify_h(any)
    # Accept both fully-qualified and arity-marked names in Global lowering path,
    # but here we catch Extern form if emitted as such.
    if extern_name == "nyash.json.stringify_h":
        # Just resolve args and call the extern; the symbol will be provided by kernel.
        i64 = ir.IntType(64)
        func = None
        for f in module.functions:
            if f.name == extern_name:
                func = f
                break
        if not func:
            func_type = ir.FunctionType(i64, [i64])
            func = ir.Function(module, func_type, name=extern_name)
        arg0 = arg_resolver.resolve_i64_or_zero(args[0]) if args else ir.Constant(i64, 0)
        res = builder.call(func, [arg0], name='unified_json_stringify')
        if dst_vid is not None:
            vmap[dst_vid] = res
        return

    # Special-case: env.future.await/spawn_instance (thin lowering)
    if extern_name == "env.future.await":
        a0 = arg_resolver.resolve_i64_or_zero(args[0]) if args else ir.Constant(i64, 0)
        if dst_vid is not None:
            vmap[dst_vid] = a0
        return

    if extern_name == "env.future.spawn_instance":
        if dst_vid is not None:
            vmap[dst_vid] = ir.Constant(i64, 0)
        return



    # Look up extern function in module
    func = None
    for f in module.functions:
        if f.name == extern_name:
            func = f
            break

    if not func:
        # Create C ABI function declaration
        if extern_name == "nyash.console.log":
            func_type = ir.FunctionType(i64, [i8p])
        elif extern_name in ["print", "panic", "error"]:
            func_type = ir.FunctionType(ir.VoidType(), [i8p])
        else:
            # Generic extern: i64 return, i64 args
            arg_types = [i64] * len(args)
            func_type = ir.FunctionType(i64, arg_types)

        func = ir.Function(module, func_type, name=extern_name)

    # Prepare arguments with C ABI type conversion
    # Minimal compat helper to replace legacy _resolve_arg usages
    def _resolve_arg(vid: int):
        try:
            return arg_resolver.resolve_i64_or_zero(vid)
        except Exception:
            from llvmlite import ir as _ir
            return _ir.Constant(_ir.IntType(64), 0)
    call_args = []
    for i, arg_id in enumerate(args):
        arg_val = _resolve_arg(arg_id)
        if arg_val is None:
            arg_val = ir.Constant(i64, 0)

        # Type conversion for C ABI
        if i < len(func.args):
            expected_type = func.args[i].type

            if expected_type.is_pointer:
                # Convert i64 handle to i8* for string parameters
                if isinstance(arg_val.type, ir.IntType) and arg_val.type.width == 64:
                    # Use string handle-to-pointer conversion
                    try:
                        to_i8p = None
                        for f in module.functions:
                            if f.name == "nyash.string.to_i8p_h":
                                to_i8p = f
                                break
                        if not to_i8p:
                            to_i8p_type = ir.FunctionType(i8p, [i64])
                            to_i8p = ir.Function(module, to_i8p_type, name="nyash.string.to_i8p_h")

                        arg_val = builder.call(to_i8p, [arg_val], name=f"unified_extern_h2p_{i}")
                    except:
                        # Fallback: inttoptr conversion
                        arg_val = builder.inttoptr(arg_val, expected_type, name=f"unified_extern_i2p_{i}")
                elif not arg_val.type.is_pointer:
                    arg_val = builder.inttoptr(arg_val, expected_type, name=f"unified_extern_i2p_{i}")

            elif isinstance(expected_type, ir.IntType):
                # Convert to expected integer width
                if arg_val.type.is_pointer:
                    arg_val = builder.ptrtoint(arg_val, expected_type, name=f"unified_extern_p2i_{i}")
                elif isinstance(arg_val.type, ir.IntType) and arg_val.type.width != expected_type.width:
                    if arg_val.type.width < expected_type.width:
                        arg_val = builder.zext(arg_val, expected_type, name=f"unified_extern_zext_{i}")
                    else:
                        arg_val = builder.trunc(arg_val, expected_type, name=f"unified_extern_trunc_{i}")

        call_args.append(arg_val)

    # Make the C ABI call - TRUE UNIFIED
    if len(call_args) == len(func.args):
        result = builder.call(func, call_args, name=f"unified_extern_{extern_name}")
    else:
        # Truncate args to match function signature
        result = builder.call(func, call_args[:len(func.args)], name=f"unified_extern_{extern_name}_trunc")

    # Store result
    if dst_vid is not None:
        ret_type = func.function_type.return_type
        if isinstance(ret_type, ir.VoidType):
            vmap[dst_vid] = ir.Constant(i64, 0)
        else:
            vmap[dst_vid] = result
