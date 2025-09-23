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
    use_unified = os.getenv("NYASH_MIR_UNIFIED_CALL", "0") != "0"
    if not use_unified:
        # Fall back to legacy dispatching
        return lower_legacy_call(owner, builder, mir_call, dst_vid, vmap, resolver)

    callee = mir_call.get("callee", {})
    args = mir_call.get("args", [])
    flags = mir_call.get("flags", {})
    effects = mir_call.get("effects", {})

    # Parse callee type
    callee_type = callee.get("type")

    if callee_type == "Global":
        # Global function call (e.g., print, panic)
        func_name = callee.get("name")
        lower_global_call(builder, owner.module, func_name, args, dst_vid, vmap, resolver, owner)

    elif callee_type == "Method":
        # Box method call
        box_name = callee.get("box_name")
        method = callee.get("method")
        receiver = callee.get("receiver")
        lower_method_call(builder, owner.module, box_name, method, receiver, args, dst_vid, vmap, resolver, owner)

    elif callee_type == "Constructor":
        # Box constructor (NewBox)
        box_type = callee.get("box_type")
        lower_constructor_call(builder, owner.module, box_type, args, dst_vid, vmap, resolver, owner)

    elif callee_type == "Closure":
        # Closure creation (NewClosure)
        params = callee.get("params", [])
        captures = callee.get("captures", [])
        me_capture = callee.get("me_capture")
        lower_closure_creation(builder, owner.module, params, captures, me_capture, dst_vid, vmap, resolver, owner)

    elif callee_type == "Value":
        # Dynamic function value call
        func_vid = callee.get("value")
        lower_value_call(builder, owner.module, func_vid, args, dst_vid, vmap, resolver, owner)

    elif callee_type == "Extern":
        # External C ABI function call
        extern_name = callee.get("name")
        lower_extern_call(builder, owner.module, extern_name, args, dst_vid, vmap, resolver, owner)

    else:
        raise ValueError(f"Unknown callee type: {callee_type}")


def lower_legacy_call(owner, builder, mir_call, dst_vid, vmap, resolver):
    """Legacy dispatcher for backward compatibility"""
    # This would dispatch to the old instruction handlers
    # For now, just raise an error if unified is disabled
    raise NotImplementedError("Legacy call dispatch not implemented in mir_call.py")


def lower_global_call(builder, module, func_name, args, dst_vid, vmap, resolver, owner):
    """Lower global function call (replaces part of call.py)"""
    # Import the original implementation
    from instructions.call import lower_call
    lower_call(builder, module, func_name, args, dst_vid, vmap, resolver,
               owner.preds, owner.block_end_values, owner.bb_map, getattr(owner, 'ctx', None))


def lower_method_call(builder, module, box_name, method, receiver, args, dst_vid, vmap, resolver, owner):
    """Lower box method call (replaces boxcall.py)"""
    # Import the original implementation
    from instructions.boxcall import lower_boxcall
    lower_boxcall(builder, module, receiver, method, args, dst_vid, vmap, resolver,
                  owner.preds, owner.block_end_values, owner.bb_map, getattr(owner, 'ctx', None))


def lower_constructor_call(builder, module, box_type, args, dst_vid, vmap, resolver, owner):
    """Lower box constructor (replaces newbox.py)"""
    # Import the original implementation
    from instructions.newbox import lower_newbox
    lower_newbox(builder, module, box_type, args, dst_vid, vmap, resolver,
                 getattr(owner, 'ctx', None))


def lower_closure_creation(builder, module, params, captures, me_capture, dst_vid, vmap, resolver, owner):
    """Lower closure creation (new implementation for NewClosure)"""
    # TODO: Implement closure creation
    # For now, create a placeholder i64 value
    if dst_vid is not None:
        closure_handle = ir.Constant(ir.IntType(64), 0)
        resolver.store(dst_vid, closure_handle)


def lower_value_call(builder, module, func_vid, args, dst_vid, vmap, resolver, owner):
    """Lower dynamic function value call"""
    # Resolve the function value
    func_val = resolver.resolve(func_vid, builder.block, owner.preds, owner.block_end_values, owner.bb_map)

    # TODO: Implement dynamic dispatch
    # For now, just store a dummy value
    if dst_vid is not None:
        result = ir.Constant(ir.IntType(64), 0)
        resolver.store(dst_vid, result)


def lower_extern_call(builder, module, extern_name, args, dst_vid, vmap, resolver, owner):
    """Lower external C ABI call (replaces externcall.py)"""
    # Import the original implementation
    from instructions.externcall import lower_externcall
    lower_externcall(builder, module, extern_name, args, dst_vid, vmap, resolver,
                     owner.preds, owner.block_end_values, owner.bb_map, getattr(owner, 'ctx', None))