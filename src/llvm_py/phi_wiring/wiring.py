from __future__ import annotations
from typing import Dict, List, Any, Optional, Tuple

import llvmlite.ir as ir

from .common import trace

def _const_i64(builder, n: int) -> ir.Constant:
    try:
        return ir.Constant(builder.i64, int(n))
    except Exception:
        # Failsafe: llvmlite requires a Module-bound type; fallback to 64-bit 0
        return ir.Constant(ir.IntType(64), int(n) if isinstance(n, int) else 0)


def ensure_phi(builder, block_id: int, dst_vid: int, bb: ir.Block) -> ir.Instruction:
    """Ensure a PHI placeholder exists at the block head for dst_vid and return it."""
    # Always place PHI at block start to keep LLVM invariant "PHI nodes at top"
    b = ir.IRBuilder(bb)
    try:
        b.position_at_start(bb)
    except Exception:
        pass
    predecl = getattr(builder, "predeclared_ret_phis", {}) if hasattr(builder, "predeclared_ret_phis") else {}
    phi = predecl.get((int(block_id), int(dst_vid))) if predecl else None
    if phi is not None:
        builder.vmap[dst_vid] = phi
        trace({"phi": "ensure_predecl", "block": int(block_id), "dst": int(dst_vid)})
        return phi
    cur = builder.vmap.get(dst_vid)
    try:
        if cur is not None and hasattr(cur, "add_incoming") and getattr(getattr(cur, "basic_block", None), "name", None) == bb.name:
            return cur
    except Exception:
        pass
    ph = b.phi(builder.i64, name=f"phi_{dst_vid}")
    builder.vmap[dst_vid] = ph
    trace({"phi": "ensure_create", "block": int(block_id), "dst": int(dst_vid)})
    return ph


def phi_at_block_head(block: ir.Block, ty: ir.Type, name: str | None = None) -> ir.Instruction:
    """Create a PHI at the very start of `block` and return it.
    Keeps LLVM's requirement that PHI nodes are grouped at the top of a block.
    """
    b = ir.IRBuilder(block)
    try:
        b.position_at_start(block)
    except Exception:
        pass
    return b.phi(ty, name=name) if name is not None else b.phi(ty)


def build_succs(preds: Dict[int, List[int]]) -> Dict[int, List[int]]:
    succs: Dict[int, List[int]] = {}
    for to_bid, from_list in (preds or {}).items():
        for fr in from_list:
            succs.setdefault(fr, []).append(to_bid)
    return succs


def nearest_pred_on_path(
    succs: Dict[int, List[int]], preds_list: List[int], decl_b: int, target_bid: int
) -> Optional[int]:
    from collections import deque

    q = deque([decl_b])
    visited = set([decl_b])
    parent: Dict[int, Any] = {decl_b: None}
    while q:
        cur = q.popleft()
        if cur == target_bid:
            par = parent.get(target_bid)
            return par if par in preds_list else None
        for nx in succs.get(cur, []):
            if nx not in visited:
                visited.add(nx)
                parent[nx] = cur
                q.append(nx)
    return None


def wire_incomings(builder, block_id: int, dst_vid: int, incoming: List[Tuple[int, int]]):
    """Wire PHI incoming edges for (block_id, dst_vid) using declared (decl_b, v_src) pairs."""
    bb = builder.bb_map.get(block_id)
    if bb is None:
        return
    phi = ensure_phi(builder, block_id, dst_vid, bb)
    # Include self-loops for loop PHIs
    preds_raw = builder.preds.get(block_id, [])
    seen = set()
    preds_list: List[int] = []
    for p in preds_raw:
        if p not in seen:
            preds_list.append(p)
            seen.add(p)
    succs = build_succs(builder.preds)
    init_src_vid = None
    for (_bd0, vs0) in incoming:
        try:
            vi = int(vs0)
        except Exception:
            continue
        if vi != int(dst_vid):
            init_src_vid = vi
            break
    chosen: Dict[int, ir.Value] = {}
    for (b_decl, v_src) in incoming:
        try:
            bd = int(b_decl)
            vs = int(v_src)
        except Exception:
            continue
        pred_match = nearest_pred_on_path(succs, preds_list, bd, block_id)
        if pred_match is None:
            trace({"phi": "wire_skip_no_path", "decl_b": bd, "target": int(block_id), "src": vs})
            continue
        if vs == int(dst_vid) and init_src_vid is not None:
            vs = int(init_src_vid)
        try:
            val = builder.resolver._value_at_end_i64(
                vs, pred_match, builder.preds, builder.block_end_values, builder.vmap, builder.bb_map
            )
        except Exception:
            val = None
        # Normalize to a well-typed LLVM value (i64)
        if val is None:
            val = _const_i64(builder, 0)
        else:
            try:
                # Some paths can accidentally pass plain integers; coerce to i64 const
                if not hasattr(val, 'type'):
                    val = _const_i64(builder, int(val))
            except Exception:
                val = _const_i64(builder, 0)
        chosen[pred_match] = val
        trace({"phi": "wire_choose", "pred": int(pred_match), "dst": int(dst_vid), "src": int(vs)})
    wired = 0
    for pred_bid, val in chosen.items():
        pred_bb = builder.bb_map.get(pred_bid)
        if pred_bb is None:
            continue
        # llvmlite requires (value, block) of correct types
        phi.add_incoming(val, pred_bb)
        trace({"phi": "add_incoming", "dst": int(dst_vid), "pred": int(pred_bid)})
        wired += 1
    return wired


def finalize_phis(builder):
    total_blocks = 0
    total_dsts = 0
    total_wired = 0
    for block_id, dst_map in (getattr(builder, "block_phi_incomings", {}) or {}).items():
        total_blocks += 1
        for dst_vid, incoming in (dst_map or {}).items():
            total_dsts += 1
            # Unification policy: PHI is created by PhiHandler at block head.
            # Here we only wire incomings to an existing PHI. We do NOT create new PHIs by default.
            # If no PHI exists for (block,dst), skip wiring to avoid creating empty/unwired PHIs.
            phi_obj = None
            try:
                phi_obj = builder.vmap.get(int(dst_vid))
            except Exception:
                phi_obj = None

            allow_create = False
            try:
                import os
                allow_create = os.environ.get('NYASH_LLVM_PHI_ALLOW_CREATE') == '1'
            except Exception:
                allow_create = False

            if phi_obj is None or not hasattr(phi_obj, 'add_incoming'):
                if allow_create:
                    # Fallback (opt‑in): ensure/create a PHI only when explicitly allowed.
                    wired = wire_incomings(builder, int(block_id), int(dst_vid), incoming)
                else:
                    trace({"phi": "finalize_skip_missing_phi", "block": int(block_id), "dst": int(dst_vid)})
                    wired = 0
            else:
                # PHI exists at block head; wire incomings normally.
                wired = wire_incomings(builder, int(block_id), int(dst_vid), incoming)
            total_wired += int(wired or 0)
            trace({"phi": "finalize", "block": int(block_id), "dst": int(dst_vid), "wired": int(wired or 0)})
    trace({"phi": "finalize_summary", "blocks": int(total_blocks), "dsts": int(total_dsts), "incoming_wired": int(total_wired)})
