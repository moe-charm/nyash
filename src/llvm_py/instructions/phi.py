"""
PHI instruction lowering
Critical for SSA form - handles value merging from different control flow paths
"""

import llvmlite.ir as ir
from typing import Dict, List, Tuple, Optional

def lower_phi(
    builder: ir.IRBuilder,
    dst_vid: int,
    incoming: List[Tuple[int, int]],  # [(value_id, block_id), ...]
    vmap: Dict[int, ir.Value],
    bb_map: Dict[int, ir.Block],
    current_block: ir.Block,
    resolver=None,  # Resolver instance (optional)
    block_end_values: Optional[Dict[int, Dict[int, ir.Value]]] = None,
    preds_map: Optional[Dict[int, List[int]]] = None
) -> None:
    """
    Lower MIR PHI instruction
    
    Args:
        builder: Current LLVM IR builder
        dst_vid: Destination value ID
        incoming: List of (value_id, block_id) pairs
        vmap: Value map
        bb_map: Block map
        current_block: Current basic block
        resolver: Optional resolver for advanced type handling
    """
    if not incoming:
        # No incoming edges - use zero
        vmap[dst_vid] = ir.Constant(ir.IntType(64), 0)
        return
    
    # Use i64 for PHI to carry handles across blocks (strings/boxes),
    # avoiding pointer PHIs that complicate dominance and boxing.
    phi_type = ir.IntType(64)
    
    # Build map from provided incoming
    incoming_map: Dict[int, int] = {}
    for val_id, block_id in incoming:
        incoming_map[block_id] = val_id

    # Resolve actual predecessor set
    cur_bid = None
    try:
        cur_bid = int(str(current_block.name).replace('bb',''))
    except Exception:
        pass
    actual_preds = []
    if preds_map is not None and cur_bid is not None:
        actual_preds = [p for p in preds_map.get(cur_bid, []) if p != cur_bid]
    else:
        # Fallback: use blocks in incoming list
        actual_preds = [b for _, b in incoming]

    # Collect incoming values
    incoming_pairs: List[Tuple[ir.Block, ir.Value]] = []
    for block_id in actual_preds:
        block = bb_map.get(block_id)
        vid = incoming_map.get(block_id)
        if block is None:
            continue
        # Prefer resolver-driven localization per predecessor block to satisfy dominance
        if vid is not None and resolver is not None and bb_map is not None:
            try:
                pred_block_obj = bb_map.get(block_id)
                if pred_block_obj is not None and hasattr(resolver, 'resolve_i64'):
                    val = resolver.resolve_i64(vid, pred_block_obj, preds_map or {}, block_end_values or {}, vmap, bb_map)
                else:
                    val = None
            except Exception:
                val = None
        else:
            # Snapshot fallback
            if block_end_values is not None:
                snap = block_end_values.get(block_id, {})
                val = snap.get(vid) if vid is not None else None
            else:
                val = vmap.get(vid) if vid is not None else None
            if not val:
                # Missing incoming for this predecessor → default 0
                val = ir.Constant(phi_type, 0)
            # Coerce pointer to i64 at predecessor end
            if hasattr(val, 'type') and val.type != phi_type:
                pb = ir.IRBuilder(block)
                try:
                    term = block.terminator
                    if term is not None:
                        pb.position_before(term)
                    else:
                        pb.position_at_end(block)
                except Exception:
                    pb.position_at_end(block)
                if isinstance(phi_type, ir.IntType) and val.type.is_pointer:
                    i8p = ir.IntType(8).as_pointer()
                    try:
                        if hasattr(val.type, 'pointee') and isinstance(val.type.pointee, ir.ArrayType):
                            c0 = ir.Constant(ir.IntType(32), 0)
                            val = pb.gep(val, [c0, c0], name=f"phi_gep_{vid}")
                    except Exception:
                        pass
                    boxer = None
                    for f in builder.module.functions:
                        if f.name == 'nyash.box.from_i8_string':
                            boxer = f
                            break
                    if boxer is None:
                        boxer = ir.Function(builder.module, ir.FunctionType(ir.IntType(64), [i8p]), name='nyash.box.from_i8_string')
                    val = pb.call(boxer, [val], name=f"phi_ptr2h_{vid}")
        incoming_pairs.append((block, val))

    # If nothing collected, use zero constant and bail out
    if not incoming_pairs:
        vmap[dst_vid] = ir.Constant(phi_type, 0)
        return

    # Create PHI instruction now and add incoming
    phi = builder.phi(phi_type, name=f"phi_{dst_vid}")
    for block, val in incoming_pairs:
        phi.add_incoming(val, block)
    
    # Store PHI result
    vmap[dst_vid] = phi

def defer_phi_wiring(
    dst_vid: int,
    incoming: List[Tuple[int, int]],
    phi_deferrals: List[Tuple[int, List[Tuple[int, int]]]]
) -> None:
    """
    Defer PHI wiring for sealed block approach
    
    Args:
        dst_vid: Destination value ID
        incoming: Incoming edges
        phi_deferrals: List to store deferred PHIs
    """
    phi_deferrals.append((dst_vid, incoming))
