"""
Safepoint instruction lowering
GC safepoints where runtime can safely collect garbage
"""

import llvmlite.ir as ir
from typing import Dict, List, Optional

def lower_safepoint(
    builder: ir.IRBuilder,
    module: ir.Module,
    live_values: List[int],
    vmap: Dict[int, ir.Value],
    safepoint_id: Optional[int] = None
) -> None:
    """
    Lower MIR Safepoint instruction
    
    Safepoints are places where GC can safely run.
    Live values must be tracked for potential relocation.
    
    Args:
        builder: Current LLVM IR builder
        module: LLVM module
        live_values: List of value IDs that are live across safepoint
        vmap: Value map
        safepoint_id: Optional safepoint identifier
    """
    # Look up or declare safepoint function
    safepoint_func = None
    for f in module.functions:
        if f.name == "ny_safepoint":
            safepoint_func = f
            break
    
    if not safepoint_func:
        # Declare ny_safepoint(live_count: i64, live_values: i64*) -> void
        i64 = ir.IntType(64)
        void = ir.VoidType()
        func_type = ir.FunctionType(void, [i64, i64.as_pointer()])
        safepoint_func = ir.Function(module, func_type, name="ny_safepoint")
    
    # Prepare live values array
    i64 = ir.IntType(64)
    if live_values:
        # Allocate array for live values
        array_size = len(live_values)
        live_array = builder.alloca(i64, size=array_size, name="live_vals")
        
        # Store each live value
        for i, vid in enumerate(live_values):
            val = vmap.get(vid, ir.Constant(i64, 0))
            
            # Ensure i64 (handles are i64)
            if hasattr(val, 'type') and val.type.is_pointer:
                val = builder.ptrtoint(val, i64)
            
            idx = ir.Constant(ir.IntType(32), i)
            ptr = builder.gep(live_array, [idx])
            builder.store(val, ptr)
        
        # Call safepoint
        count = ir.Constant(i64, array_size)
        builder.call(safepoint_func, [count, live_array])
        
        # After safepoint, reload values (they may have moved)
        for i, vid in enumerate(live_values):
            idx = ir.Constant(ir.IntType(32), i)
            ptr = builder.gep(live_array, [idx])
            new_val = builder.load(ptr, name=f"reload_{vid}")
            vmap[vid] = new_val
    else:
        # No live values
        zero = ir.Constant(i64, 0)
        null = ir.Constant(i64.as_pointer(), None)
        builder.call(safepoint_func, [zero, null])

def insert_automatic_safepoint(
    builder: ir.IRBuilder,
    module: ir.Module,
    location: str  # "loop_header", "function_call", etc.
) -> None:
    """
    Insert automatic safepoint at strategic locations
    
    Args:
        builder: Current LLVM IR builder
        module: LLVM module
        location: Location type for debugging
    """
    # Simple safepoint without tracking specific values
    # Runtime will scan stack/registers
    
    check_func = None
    for f in module.functions:
        if f.name == "ny_check_safepoint":
            check_func = f
            break
    
    if not check_func:
        # Declare ny_check_safepoint() -> void
        void = ir.VoidType()
        func_type = ir.FunctionType(void, [])
        check_func = ir.Function(module, func_type, name="ny_check_safepoint")
    
    # Insert safepoint check
    builder.call(check_func, [], name=f"safepoint_{location}")