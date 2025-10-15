"""
Memory operations (load/store) for WASM linear memory
"""

from typing import Dict, Any, Optional
import llvmlite.ir as ir
from utils.values import resolve_i64_strict


def lower_load(
    builder: ir.IRBuilder,
    dst: int,
    addr: int,
    vmap: Dict[int, ir.Value],
    resolver=None,
    current_block=None,
    preds=None,
    block_end_values=None,
    bb_map=None,
    ctx: Optional[Any] = None,
) -> None:
    """
    Lower MIR load instruction

    Load value from memory address (WASM linear memory)

    Args:
        builder: LLVM IR builder
        dst: Destination value ID
        addr: Address value ID
        vmap: Value map
    """
    # Resolve address value
    if resolver is not None and current_block is not None:
        addr_val = resolve_i64_strict(resolver, addr, current_block, preds, block_end_values, vmap, bb_map, builder=builder)
    else:
        addr_val = vmap.get(addr)

    if addr_val is None:
        # Default: return 0
        vmap[dst] = ir.Constant(ir.IntType(64), 0)
        return

    # Convert address (i64) to pointer (i64*)
    i64 = ir.IntType(64)
    ptr_type = i64.as_pointer()

    # inttoptr: i64 -> i64*
    ptr = builder.inttoptr(addr_val, ptr_type, name=f"load_ptr_{dst}")

    # Load from pointer
    loaded = builder.load(ptr, name=f"load_{dst}")

    vmap[dst] = loaded


def lower_store(
    builder: ir.IRBuilder,
    addr: int,
    value: int,
    vmap: Dict[int, ir.Value],
    resolver=None,
    current_block=None,
    preds=None,
    block_end_values=None,
    bb_map=None,
    ctx: Optional[Any] = None,
) -> None:
    """
    Lower MIR store instruction

    Store value to memory address (WASM linear memory)

    Args:
        builder: LLVM IR builder
        addr: Address value ID
        value: Value to store (value ID)
        vmap: Value map
    """
    # Resolve address and value
    if resolver is not None and current_block is not None:
        addr_val = resolve_i64_strict(resolver, addr, current_block, preds, block_end_values, vmap, bb_map, builder=builder)
        val = resolve_i64_strict(resolver, value, current_block, preds, block_end_values, vmap, bb_map, builder=builder)
    else:
        addr_val = vmap.get(addr)
        val = vmap.get(value)

    if addr_val is None or val is None:
        # Cannot store without valid address and value
        return

    # Convert address (i64) to pointer (i64*)
    i64 = ir.IntType(64)
    ptr_type = i64.as_pointer()

    # inttoptr: i64 -> i64*
    ptr = builder.inttoptr(addr_val, ptr_type, name=f"store_ptr")

    # Store to pointer
    builder.store(val, ptr)
