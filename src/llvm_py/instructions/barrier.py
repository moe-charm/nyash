"""
Barrier instruction lowering
Memory barriers for thread safety and memory ordering
"""

import llvmlite.ir as ir
from typing import Dict, Optional

def lower_barrier(
    builder: ir.IRBuilder,
    barrier_type: str,
    ordering: Optional[str] = None,
    ctx=None,
) -> None:
    """
    Lower MIR Barrier instruction
    
    Barrier types:
    - memory: Full memory fence
    - acquire: Acquire semantics
    - release: Release semantics
    - acq_rel: Acquire-release
    - seq_cst: Sequential consistency
    
    Args:
        builder: Current LLVM IR builder
        barrier_type: Type of barrier
        ordering: Optional memory ordering specification
    """
    # Map barrier types to LLVM atomic ordering
    ordering_map = {
        "acquire": "acquire",
        "release": "release", 
        "acq_rel": "acq_rel",
        "seq_cst": "seq_cst",
        "memory": "seq_cst",  # Full fence
    }
    
    llvm_ordering = ordering_map.get(barrier_type, "seq_cst")
    
    # Insert fence instruction
    builder.fence(llvm_ordering)

def lower_atomic_op(
    builder: ir.IRBuilder,
    op: str,  # "load", "store", "add", "cas"
    ptr_vid: int,
    val_vid: Optional[int],
    dst_vid: Optional[int],
    vmap: Dict[int, ir.Value],
    ordering: str = "seq_cst",
    resolver=None,
    preds=None,
    block_end_values=None,
    bb_map=None,
    ctx=None,
) -> None:
    """
    Lower atomic operations
    
    Args:
        builder: Current LLVM IR builder
        op: Atomic operation type
        ptr_vid: Pointer value ID
        val_vid: Value ID for store/rmw operations
        dst_vid: Destination ID for load/rmw operations
        vmap: Value map
        ordering: Memory ordering
    """
    # Get pointer
    if ctx is not None:
        try:
            ptr = ctx.resolver.resolve_ptr(ptr_vid, builder.block, ctx.preds, ctx.block_end_values, ctx.vmap)
        except Exception:
            ptr = vmap.get(ptr_vid)
    elif resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
        ptr = resolver.resolve_ptr(ptr_vid, builder.block, preds, block_end_values, vmap)
    else:
        ptr = vmap.get(ptr_vid)
    if not ptr:
        # Create dummy pointer
        i64 = ir.IntType(64)
        ptr = builder.alloca(i64, name="atomic_ptr")
        vmap[ptr_vid] = ptr
    
    if op == "load":
        # Atomic load
        result = builder.load_atomic(ptr, ordering=ordering, align=8)
        if dst_vid is not None:
            vmap[dst_vid] = result
            
    elif op == "store":
        # Atomic store
        if val_vid is not None:
            if ctx is not None:
                try:
                    val = ctx.resolver.resolve_i64(val_vid, builder.block, ctx.preds, ctx.block_end_values, ctx.vmap, ctx.bb_map)
                except Exception:
                    val = vmap.get(val_vid, ir.Constant(ir.IntType(64), 0))
            elif resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
                val = resolver.resolve_i64(val_vid, builder.block, preds, block_end_values, vmap, bb_map)
            else:
                val = vmap.get(val_vid, ir.Constant(ir.IntType(64), 0))
            builder.store_atomic(val, ptr, ordering=ordering, align=8)
            
    elif op == "add":
        # Atomic add (fetch_add)
        if val_vid is not None:
            if ctx is not None:
                try:
                    val = ctx.resolver.resolve_i64(val_vid, builder.block, ctx.preds, ctx.block_end_values, ctx.vmap, ctx.bb_map)
                except Exception:
                    val = ir.Constant(ir.IntType(64), 1)
            elif resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
                val = resolver.resolve_i64(val_vid, builder.block, preds, block_end_values, vmap, bb_map)
            else:
                val = ir.Constant(ir.IntType(64), 1)
            result = builder.atomic_rmw("add", ptr, val, ordering=ordering)
            if dst_vid is not None:
                vmap[dst_vid] = result
                
    elif op == "cas":
        # Compare and swap
        # TODO: Needs expected and new values
        pass

def insert_thread_fence(
    builder: ir.IRBuilder,
    module: ir.Module,
    fence_type: str = "full"
) -> None:
    """
    Insert thread synchronization fence
    
    Args:
        builder: Current LLVM IR builder
        module: LLVM module  
        fence_type: Type of fence (full, read, write)
    """
    if fence_type == "full":
        builder.fence("seq_cst")
    elif fence_type == "read":
        builder.fence("acquire")
    elif fence_type == "write":
        builder.fence("release")
    else:
        builder.fence("seq_cst")
