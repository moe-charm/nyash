"""
Return instruction lowering
Handles void and value returns
"""

import llvmlite.ir as ir
from typing import Dict, Optional, Any
from dispatch.type_coercion import TypeCoercion

def lower_return(
    builder: ir.IRBuilder,
    value_id: Optional[int],
    vmap: Dict[int, ir.Value],
    return_type: ir.Type,
    resolver=None,
    preds=None,
    block_end_values=None,
    bb_map=None,
    ctx: Optional[Any] = None,
) -> None:
    """
    Lower MIR Return instruction
    
    Args:
        builder: Current LLVM IR builder
        value_id: Optional return value ID
        vmap: Value map
        return_type: Expected return type
    """
    # Prefer BuildCtx maps if provided
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
    if value_id is None:
        # Void return
        builder.ret_void()
    else:
        # Get return value (prefer resolver)
        ret_val = None
        # Fast path: if vmap has a value (including PHI), use it directly
        if isinstance(value_id, int):
            tmp0 = vmap.get(value_id)
            if tmp0 is not None:
                ret_val = tmp0
        if ret_val is None:
            if resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
                # Resolve direct value; PHIは finalize_phis に一任
                if isinstance(return_type, ir.PointerType):
                    ret_val = resolver.resolve_ptr(value_id, builder.block, preds, block_end_values, vmap)
                else:
                    is_stringish = False
                    if hasattr(resolver, 'is_stringish'):
                        try:
                            is_stringish = resolver.is_stringish(int(value_id))
                        except Exception:
                            is_stringish = False
                    if is_stringish and hasattr(resolver, 'string_ptrs') and int(value_id) in getattr(resolver, 'string_ptrs'):
                        p = resolver.string_ptrs[int(value_id)]
                        i8p = ir.IntType(8).as_pointer()
                        i64 = ir.IntType(64)
                        boxer = None
                        for f in builder.module.functions:
                            if f.name == 'nyash.box.from_i8_string':
                                boxer = f; break
                        if boxer is None:
                            boxer = ir.Function(builder.module, ir.FunctionType(i64, [i8p]), name='nyash.box.from_i8_string')
                        ret_val = builder.call(boxer, [p], name='ret_ptr2h')
                    else:
                        ret_val = resolver.resolve_i64(value_id, builder.block, preds, block_end_values, vmap, bb_map)
                
        if ret_val is None:
            # Default to vmap (non-PHI) if available
            tmp = vmap.get(value_id)
            try:
                is_phi = hasattr(tmp, 'add_incoming')
            except Exception:
                is_phi = False
            if tmp is not None and not is_phi:
                ret_val = tmp
        if not ret_val:
            # Default based on return type
            if isinstance(return_type, ir.IntType):
                ret_val = ir.Constant(return_type, 0)
            elif isinstance(return_type, ir.DoubleType):
                ret_val = ir.Constant(return_type, 0.0)
            else:
                # Pointer type - null
                ret_val = ir.Constant(return_type, None)
        
        # Type adjustment if needed - delegate to TypeCoercion
        ret_val = TypeCoercion.to_type(builder, ret_val, return_type, "ret")
        
        builder.ret(ret_val)
