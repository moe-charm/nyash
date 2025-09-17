"""
ExternCall instruction lowering
Minimal mapping for NyRT-exported symbols (console/log family等)
"""

import llvmlite.ir as ir
from typing import Dict, List, Optional, Any

def lower_externcall(
    builder: ir.IRBuilder,
    module: ir.Module,
    func_name: str,
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
    Lower MIR ExternCall instruction
    
    Args:
        builder: Current LLVM IR builder
        module: LLVM module
        func_name: External function name
        args: List of argument value IDs
        dst_vid: Optional destination for return value
        vmap: Value map
        resolver: Optional resolver for type handling
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
    # Accept full symbol names (e.g., "nyash.console.log", "nyash.string.len_h").
    llvm_name = func_name

    i8 = ir.IntType(8)
    i64 = ir.IntType(64)
    i8p = i8.as_pointer()
    void = ir.VoidType()

    # Known NyRT signatures
    sig_map = {
        # Strings (handle-based)
        "nyash.string.len_h": (i64, [i64]),
        "nyash.string.charCodeAt_h": (i64, [i64, i64]),
        "nyash.string.concat_hh": (i64, [i64, i64]),
        "nyash.string.eq_hh": (i64, [i64, i64]),
        "nyash.string.substring_hii": (i64, [i64, i64, i64]),
        "nyash.string.lastIndexOf_hh": (i64, [i64, i64]),
        # Strings (pointer-based plugin functions)
        "nyash.string.concat_ss": (i8p, [i8p, i8p]),
        "nyash.string.concat_si": (i8p, [i8p, i64]),
        "nyash.string.concat_is": (i8p, [i64, i8p]),
        "nyash.string.substring_sii": (i8p, [i8p, i64, i64]),
        "nyash.string.lastIndexOf_ss": (i64, [i8p, i8p]),
        # Boxing helpers
        "nyash.box.from_i8_string": (i64, [i8p]),
        # Console (string pointer expected)
        # Many call sites pass handles or pointers; we coerce below.
    }

    # Find or declare function with appropriate prototype
    func = None
    for f in module.functions:
        if f.name == llvm_name:
            func = f
            break
    if not func:
        if llvm_name in sig_map:
            ret_ty, arg_tys = sig_map[llvm_name]
            fnty = ir.FunctionType(ret_ty, arg_tys)
            func = ir.Function(module, fnty, name=llvm_name)
        elif llvm_name.startswith("nyash.console."):
            # console.*: (i8*) -> i64
            fnty = ir.FunctionType(i64, [i8p])
            func = ir.Function(module, fnty, name=llvm_name)
        else:
            # Unknown extern: declare as void(...no args...) and call without args
            fnty = ir.FunctionType(void, [])
            func = ir.Function(module, fnty, name=llvm_name)

    # Prepare/coerce arguments
    call_args: List[ir.Value] = []
    for i, arg_id in enumerate(args):
        orig_arg_id = arg_id
        # Prefer resolver/ctx
        aval = None
        if resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
            try:
                if len(func.args) > i and isinstance(func.args[i].type, ir.PointerType):
                    aval = resolver.resolve_ptr(arg_id, builder.block, preds, block_end_values, vmap)
                else:
                    aval = resolver.resolve_i64(arg_id, builder.block, preds, block_end_values, vmap, bb_map)
            except Exception:
                aval = None
        if aval is None:
            aval = vmap.get(arg_id)
        if aval is None:
            # Default guess
            aval = ir.Constant(i64, 0)

        # If function prototype is known, coerce to expected type
        if len(func.args) > i:
            expected_ty = func.args[i].type
            if isinstance(expected_ty, ir.PointerType):
                # Need pointer
                # Prefer string literal pointer or handle->i8* bridge when argument is string-ish
                used_string_h2p = False
                try:
                    if resolver is not None and hasattr(resolver, 'string_ptrs'):
                        sp = resolver.string_ptrs.get(orig_arg_id)
                        if sp is not None:
                            aval = sp
                            used_string_h2p = True
                    if not used_string_h2p and resolver is not None and hasattr(resolver, 'is_stringish') and resolver.is_stringish(orig_arg_id):
                        # Declare nyash.string.to_i8p_h(i64) and call with handle
                        i64 = ir.IntType(64)
                        i8p = ir.IntType(8).as_pointer()
                        to_i8p = None
                        for f in module.functions:
                            if f.name == 'nyash.string.to_i8p_h':
                                to_i8p = f; break
                        if to_i8p is None:
                            to_i8p = ir.Function(module, ir.FunctionType(i8p, [i64]), name='nyash.string.to_i8p_h')
                        # Ensure we have an i64 handle to pass
                        if hasattr(aval, 'type') and isinstance(aval.type, ir.PointerType):
                            aval = builder.ptrtoint(aval, i64, name=f"ext_p2h_{i}")
                        elif hasattr(aval, 'type') and isinstance(aval.type, ir.IntType) and aval.type.width != 64:
                            aval = builder.zext(aval, i64, name=f"ext_zext_h_{i}")
                        aval = builder.call(to_i8p, [aval], name=f"ext_h2p_arg{i}")
                        used_string_h2p = True
                except Exception:
                    used_string_h2p = used_string_h2p or False
                if not used_string_h2p:
                    if hasattr(aval, 'type'):
                        if isinstance(aval.type, ir.IntType):
                            aval = builder.inttoptr(aval, expected_ty, name=f"ext_i2p_arg{i}")
                        elif not aval.type.is_pointer:
                            aval = ir.Constant(expected_ty, None)
                        else:
                            # Pointer but wrong element type: if pointer-to-array -> GEP to i8*
                            try:
                                if isinstance(aval.type.pointee, ir.ArrayType) and isinstance(expected_ty.pointee, ir.IntType) and expected_ty.pointee.width == 8:
                                    c0 = ir.Constant(ir.IntType(32), 0)
                                    aval = builder.gep(aval, [c0, c0], name=f"ext_gep_arg{i}")
                            except Exception:
                                pass
                else:
                    aval = ir.Constant(expected_ty, None)
            elif isinstance(expected_ty, ir.IntType) and expected_ty.width == 64:
                # Need i64
                if hasattr(aval, 'type'):
                    if isinstance(aval.type, ir.PointerType):
                        aval = builder.ptrtoint(aval, i64, name=f"ext_p2i_arg{i}")
                    elif isinstance(aval.type, ir.IntType) and aval.type.width != 64:
                        # extend/trunc
                        if aval.type.width < 64:
                            aval = builder.zext(aval, i64, name=f"ext_zext_{i}")
                        else:
                            aval = builder.trunc(aval, i64, name=f"ext_trunc_{i}")
                else:
                    aval = ir.Constant(i64, 0)
        else:
            # Prototype shorter than args: best-effort pointer->i64 for string-ish APIs
            if hasattr(aval, 'type') and isinstance(aval.type, ir.PointerType):
                aval = builder.ptrtoint(aval, i64, name=f"ext_p2i_arg{i}")
        call_args.append(aval)

    # Truncate extra args if prototype shorter
    if len(call_args) > len(func.args):
        call_args = call_args[:len(func.args)]

    # Issue the call
    if len(call_args) == len(func.args):
        result = builder.call(func, call_args, name=f"extern_{func_name}")
    else:
        result = builder.call(func, call_args[:len(func.args)])

    # Materialize result into vmap
    if dst_vid is not None:
        rty = func.function_type.return_type
        if isinstance(rty, ir.VoidType):
            vmap[dst_vid] = ir.Constant(i64, 0)
        else:
            vmap[dst_vid] = result
