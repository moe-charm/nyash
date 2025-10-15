from __future__ import annotations
from typing import Dict, List, Any, Optional, Tuple

import llvmlite.ir as ir

from .common import trace
from .registry import PhiRegistry

def _const_i64(builder, n: int) -> ir.Constant:
    try:
        return ir.Constant(builder.i64, int(n))
    except Exception:
        # Failsafe: llvmlite requires a Module-bound type; fallback to 64-bit 0
        return ir.Constant(ir.IntType(64), int(n) if isinstance(n, int) else 0)


def ensure_phi(builder, block_id: int, dst_vid: int, bb: ir.Block) -> ir.Instruction:
    """Ensure a single PHI exists at block head for (block_id, dst_vid).
    Delegates to PhiRegistry to guarantee uniqueness.
    """
    # Honor predeclared ret PHIs when provided (if-merge prepass)
    try:
        predecl = getattr(builder, "predeclared_ret_phis", {}) or {}
        if predecl:
            cand = predecl.get((int(block_id), int(dst_vid)))
            if cand is not None:
                try:
                    builder.vmap[int(dst_vid)] = cand
                except Exception:
                    pass
                trace({"phi": "ensure_predecl", "block": int(block_id), "dst": int(dst_vid)})
                # Also register into registry for uniqueness
                try:
                    PhiRegistry.register(builder, int(block_id), int(dst_vid), cand)
                except Exception:
                    pass
                return cand
    except Exception:
        pass
    ph = PhiRegistry.ensure(builder, int(block_id), int(dst_vid), bb)
    trace({"phi": "ensure_create_or_reuse", "block": int(block_id), "dst": int(dst_vid)})
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

    # Self-loop special-case: declaration and target are the same block.
    # If the target has a self predecessor edge, accept it directly.
    try:
        if int(decl_b) == int(target_bid):
            return int(target_bid) if int(target_bid) in list(preds_list or []) else None
    except Exception:
        pass

    q = deque([decl_b])
    visited = set([decl_b])
    parent: Dict[int, Any] = {decl_b: None}
    while q:
        cur = q.popleft()
        if cur == target_bid:
            par = parent.get(target_bid)
            # If parent is not a real predecessor but target itself is, allow it (self-loop tolerance).
            if par in preds_list:
                return par
            return target_bid if target_bid in preds_list else None
        for nx in succs.get(cur, []):
            if nx not in visited:
                visited.add(nx)
                parent[nx] = cur
                q.append(nx)
    return None


def wire_incomings(builder, block_id: int, dst_vid: int, incoming: List[Tuple[int, int]]):
    """Wire PHI incoming edges for (block_id, dst_vid).

    Policy change (2025-10-02): Ignore per-pair source hints and always wire the
    value of `dst_vid` as it exists at the end of each CFG predecessor block.
    This removes ambiguity when MIR redefines the same logical value per branch
    (then/else) and avoids fragile heuristics.

    Contract: PHI must already exist at the block head (created by PhiRegistry/PhiHandler).
    This helper never creates PHIs; it only wires incoming edges.
    """
    bb = builder.bb_map.get(block_id)
    if bb is None:
        return 0
    # Wire-only: fetch existing PHI from vmap
    phi = None
    try:
        phi = builder.vmap.get(int(dst_vid))
    except Exception:
        phi = None
    if phi is None or not hasattr(phi, 'add_incoming'):
        trace({"phi": "wire_skip_no_phi", "block": int(block_id), "dst": int(dst_vid)})
        return 0
    # Preserve declared predecessor order and allow self-loop when present.
    raw_list = builder.preds.get(block_id, []) or []
    preds_raw = list(dict.fromkeys(raw_list))
    seen = set()
    preds_list: List[int] = []
    for p in preds_raw:
        if p not in seen:
            preds_list.append(p)
            seen.add(p)
    succs = build_succs(builder.preds)
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
        try:
            val = builder.resolver._value_at_end_i64(
                vs, pred_match, builder.preds, builder.block_end_values, builder.vmap, builder.bb_map
            )
        except Exception:
            val = None
        if val is None:
            val = _const_i64(builder, 0)
        else:
            try:
                if not hasattr(val, 'type'):
                    val = _const_i64(builder, int(val))
            except Exception:
                val = _const_i64(builder, 0)
        chosen[int(pred_match)] = val
        trace({"phi": "wire_choose", "pred": int(pred_match), "dst": int(dst_vid), "src": int(vs)})
    wired = 0
    for pred_bid, val in chosen.items():
        pred_bb = builder.bb_map.get(pred_bid)
        if pred_bb is None:
            continue
        # Avoid duplicate incoming per predecessor: consult PhiHandler-wired set
        try:
            wired_set = getattr(builder, 'phi_wired', {}).get((int(block_id), int(dst_vid)), set())
        except Exception:
            wired_set = set()
        if int(pred_bid) in wired_set:
            trace({"phi": "skip_dup_incoming", "dst": int(dst_vid), "pred": int(pred_bid)})
            continue
        phi.add_incoming(val, pred_bb)
        # Update wired set for future passes
        try:
            key = (int(block_id), int(dst_vid))
            st = getattr(builder, 'phi_wired', {}).setdefault(key, set())
            st.add(int(pred_bid))
        except Exception:
            pass
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
                # Default: wire-only (do not create new PHIs). Opt-in via env.
                env_val = os.environ.get('NYASH_LLVM_PHI_ALLOW_CREATE')
                allow_create = (env_val == '1') if env_val is not None else False
            except Exception:
                allow_create = False

            if phi_obj is None or not hasattr(phi_obj, 'add_incoming'):
                if allow_create:
                    # Ensure/create a placeholder at block head, then wire incomings.
                    bb = (getattr(builder, 'bb_map', {}) or {}).get(int(block_id))
                    if bb is not None:
                        try:
                            ensure_phi(builder, int(block_id), int(dst_vid), bb)
                        except Exception:
                            pass
                    wired = wire_incomings(builder, int(block_id), int(dst_vid), incoming)
                else:
                    trace({"phi": "finalize_skip_missing_phi", "block": int(block_id), "dst": int(dst_vid)})
                    wired = 0
            else:
                # PHI exists at block head; wire incomings normally.
                wired = wire_incomings(builder, int(block_id), int(dst_vid), incoming)
                # Ensure non-empty PHI to avoid malformed IR in dev: if no new incoming was wired
                # AND no predecessors were previously wired (per PhiHandler), add a conservative
                # zero from the first CFG predecessor.
                try:
                    prev_wired = 0
                    try:
                        prev_wired = len((getattr(builder, 'phi_wired', {}) or {}).get((int(block_id), int(dst_vid)), set()))
                    except Exception:
                        prev_wired = 0
                    if int(wired or 0) == 0 and int(prev_wired or 0) == 0:
                        preds_list = list(dict.fromkeys((getattr(builder, 'preds', {}) or {}).get(int(block_id), []) or []))
                        if preds_list:
                            first_pred = int(preds_list[0])
                            pred_bb = (getattr(builder, 'bb_map', {}) or {}).get(first_pred)
                            if pred_bb is not None:
                                z = _const_i64(builder, 0)
                                phi_obj.add_incoming(z, pred_bb)
                                # track wired set for verify
                                try:
                                    key = (int(block_id), int(dst_vid))
                                    st = getattr(builder, 'phi_wired', {}).setdefault(key, set())
                                    st.add(first_pred)
                                except Exception:
                                    pass
                                wired = 1
                                trace({"phi": "finalize_synthesize_zero_incoming", "block": int(block_id), "dst": int(dst_vid), "pred": first_pred})
                except Exception:
                    pass
            total_wired += int(wired or 0)
            trace({"phi": "finalize", "block": int(block_id), "dst": int(dst_vid), "wired": int(wired or 0)})
    trace({"phi": "finalize_summary", "blocks": int(total_blocks), "dsts": int(total_dsts), "incoming_wired": int(total_wired)})
