"""
Call instruction lowering
Handles regular function calls (not BoxCall or ExternCall)
"""

import llvmlite.ir as ir
from typing import Dict, List, Optional, Any
from trace import debug as trace_debug
from instructions.safepoint import insert_automatic_safepoint
from dispatch import PhiDispatchPoint

def lower_call(
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
    Lower MIR Call instruction
    
    Args:
        builder: Current LLVM IR builder
        module: LLVM module
        func_name: Function name to call
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
    # Insert an automatic safepoint after the function call
    try:
        import os
        if os.environ.get('NYASH_LLVM_AUTO_SAFEPOINT', '1') == '1':
            insert_automatic_safepoint(builder, module, "function_call")
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

    # Resolver helpers (prefer resolver when available)
    def _res_i64(vid: int):
        # Use DispatchPoint to avoid 0-fallbacks (loop/merge safety)
        if r is not None and p is not None and bev is not None and bbm is not None:
            try:
                return PhiDispatchPoint.resolve_i64(builder, r, int(vid), builder.block, p, bev, vmap, bbm)
            except Exception:
                pass
        return vmap.get(vid)

    def _res_ptr(vid: int):
        if r is not None and p is not None and bev is not None:
            try:
                return r.resolve_ptr(vid, builder.block, p, bev, vmap)
            except Exception:
                return None
        return vmap.get(vid)

    # Resolve function: accepts string name or value-id referencing a string literal
    actual_name = func_name
    if not isinstance(func_name, str):
        # Try resolver.string_literals
        if resolver is not None and hasattr(resolver, 'string_literals'):
            actual_name = resolver.string_literals.get(func_name)
    # Look up function in module
    func = None
    if isinstance(actual_name, str):
        for f in module.functions:
            if f.name == actual_name:
                func = f
                break
    
    if not func:
        # Function not found - create declaration with default i64 signature
        ret_type = ir.IntType(64)
        arg_types = [ir.IntType(64)] * len(args)
        name = actual_name if isinstance(actual_name, str) else "unknown_fn"
        func_type = ir.FunctionType(ret_type, arg_types)
        func = ir.Function(module, func_type, name=name)
    
    # If calling a Dev-only predicate name (e.g., 'condition_fn') that lacks a body,
    # synthesize a trivial definition that returns non-zero to satisfy linker during bring-up.
    if isinstance(actual_name, str) and actual_name == 'condition_fn':
        try:
            if func is not None and len(list(func.blocks)) == 0:
                b = ir.IRBuilder(func.append_basic_block('entry'))
                rty = func.function_type.return_type
                if isinstance(rty, ir.IntType):
                    b.ret(ir.Constant(rty, 1))
                else:
                    b.ret_void()
        except Exception:
            pass

    # Prepare arguments
    call_args = []
    for i, arg_id in enumerate(args):
        arg_val = None
        if i < len(func.args):
            expected_type = func.args[i].type
            if hasattr(expected_type, 'is_pointer') and expected_type.is_pointer:
                arg_val = _res_ptr(arg_id)
            else:
                arg_val = _res_i64(arg_id)
        if arg_val is None:
            arg_val = vmap.get(arg_id)
        if arg_val is None:
            if i < len(func.args):
                expected_type = func.args[i].type
            else:
                expected_type = ir.IntType(64)
            if isinstance(expected_type, ir.IntType):
                arg_val = ir.Constant(expected_type, 0)
            elif isinstance(expected_type, ir.DoubleType):
                arg_val = ir.Constant(expected_type, 0.0)
            else:
                arg_val = ir.Constant(expected_type, None)
        if i < len(func.args):
            expected_type = func.args[i].type
            if hasattr(arg_val, 'type') and arg_val.type != expected_type:
                if expected_type.is_pointer and isinstance(arg_val.type, ir.IntType):
                    arg_val = builder.inttoptr(arg_val, expected_type, name=f"call_i2p_{i}")
                elif isinstance(expected_type, ir.IntType) and arg_val.type.is_pointer:
                    arg_val = builder.ptrtoint(arg_val, expected_type, name=f"call_p2i_{i}")
        call_args.append(arg_val)
    
    # Make the call
    result = builder.call(func, call_args, name=f"call_{func_name}")
    # Optional trace for final debugging
    if isinstance(actual_name, str) and actual_name in ("Main.node_json/3", "Main.esc_json/1", "main"):
        trace_debug(f"[TRACE] call {actual_name} args={len(call_args)}")
    
    # Store result if needed
    if dst_vid is not None:
        vmap[dst_vid] = result
        # Heuristic: mark known string-producing functions as string handles
        try:
            name_for_tag = actual_name if isinstance(actual_name, str) else str(actual_name)
            if resolver is not None and hasattr(resolver, 'mark_string'):
                if any(key in name_for_tag for key in [
                    'esc_json', 'node_json', 'dirname', 'join', 'read_all', 'toJson'
                ]):
                    resolver.mark_string(dst_vid)
            # Additionally, create a pointer view via bridge for println pointer-API
            if resolver is not None and hasattr(resolver, 'string_ptrs'):
                i64 = ir.IntType(64)
                i8p = ir.IntType(8).as_pointer()
                if hasattr(result, 'type') and isinstance(result.type, ir.IntType) and result.type.width == 64:
                    bridge = None
                    for f in module.functions:
                        if f.name == 'nyash.string.to_i8p_h':
                            bridge = f; break
                    if bridge is None:
                        bridge = ir.Function(module, ir.FunctionType(i8p, [i64]), name='nyash.string.to_i8p_h')
                    pv = builder.call(bridge, [result], name=f"ret_h2p_{dst_vid}")
                    resolver.string_ptrs[int(dst_vid)] = pv
        except Exception:
            pass
