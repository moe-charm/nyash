"""
PHI wiring helpers

- setup_phi_placeholders: Predeclare PHIs and collect incoming metadata
- finalize_phis: Wire PHI incomings using end-of-block snapshots and resolver

These operate on the NyashLLVMBuilder instance to keep changes minimal.

Refactor note: core responsibilities are split into smaller helpers so they
can be unit-tested in isolation.
"""

from typing import Dict, List, Any, Optional, Tuple
import os
import json
import llvmlite.ir as ir

# ---- Small helpers (analyzable/testable) ----

def _collect_produced_stringish(blocks: List[Dict[str, Any]]) -> Dict[int, bool]:
    produced_str: Dict[int, bool] = {}
    for block_data in blocks:
        for inst in block_data.get("instructions", []) or []:
            try:
                opx = inst.get("op")
                dstx = inst.get("dst")
                if dstx is None:
                    continue
                is_str = False
                if opx == "const":
                    v = inst.get("value", {}) or {}
                    t = v.get("type")
                    if t == "string" or (isinstance(t, dict) and t.get("kind") in ("handle", "ptr") and t.get("box_type") == "StringBox"):
                        is_str = True
                elif opx in ("binop", "boxcall", "externcall"):
                    t = inst.get("dst_type")
                    if isinstance(t, dict) and t.get("kind") == "handle" and t.get("box_type") == "StringBox":
                        is_str = True
                if is_str:
                    produced_str[int(dstx)] = True
            except Exception:
                pass
    return produced_str

def _trace(msg: Any):
    if os.environ.get("NYASH_LLVM_TRACE_PHI", "0") == "1":
        out = os.environ.get("NYASH_LLVM_TRACE_OUT")
        # Format as single-line JSON for machine parsing
        if not isinstance(msg, (str, bytes)):
            try:
                msg = json.dumps(msg, ensure_ascii=False, separators=(",", ":"))
            except Exception:
                msg = str(msg)
        if out:
            try:
                with open(out, "a", encoding="utf-8") as f:
                    f.write(msg.rstrip() + "\n")
            except Exception:
                pass
        else:
            try:
                print(msg)
            except Exception:
                pass


def analyze_incomings(blocks: List[Dict[str, Any]]) -> Dict[int, Dict[int, List[Tuple[int, int]]]]:
    """Return block_phi_incomings map: block_id -> { dst_vid -> [(decl_b, v_src), ...] }"""
    result: Dict[int, Dict[int, List[Tuple[int, int]]]] = {}
    for block_data in blocks:
        bid0 = block_data.get("id", 0)
        for inst in block_data.get("instructions", []) or []:
            if inst.get("op") == "phi":
                try:
                    dst0 = int(inst.get("dst"))
                    incoming0 = inst.get("incoming", []) or []
                except Exception:
                    dst0 = None; incoming0 = []
                if dst0 is None:
                    continue
                try:
                    pairs = [(int(b), int(v)) for (v, b) in incoming0]
                    result.setdefault(int(bid0), {})[dst0] = pairs
                    _trace({
                        "phi": "analyze",
                        "block": int(bid0),
                        "dst": dst0,
                        "incoming": pairs,
                    })
                except Exception:
                    pass
    return result

def ensure_phi(builder, block_id: int, dst_vid: int, bb: ir.Block) -> ir.Instruction:
    """Ensure a PHI placeholder exists at the block head for dst_vid and return it."""
    b = ir.IRBuilder(bb)
    try:
        b.position_at_start(bb)
    except Exception:
        pass
    # Prefer predeclared PHIs (e.g., from if-merge prepass)
    predecl = getattr(builder, 'predeclared_ret_phis', {}) if hasattr(builder, 'predeclared_ret_phis') else {}
    phi = predecl.get((int(block_id), int(dst_vid))) if predecl else None
    if phi is not None:
        builder.vmap[dst_vid] = phi
        _trace({"phi": "ensure_predecl", "block": int(block_id), "dst": int(dst_vid)})
        return phi
    # Reuse current if it is a PHI in the correct block
    cur = builder.vmap.get(dst_vid)
    try:
        if cur is not None and hasattr(cur, 'add_incoming') and getattr(getattr(cur, 'basic_block', None), 'name', None) == bb.name:
            return cur
    except Exception:
        pass
    # Create a new placeholder
    ph = b.phi(builder.i64, name=f"phi_{dst_vid}")
    builder.vmap[dst_vid] = ph
    _trace({"phi": "ensure_create", "block": int(block_id), "dst": int(dst_vid)})
    return ph

def _build_succs(preds: Dict[int, List[int]]) -> Dict[int, List[int]]:
    succs: Dict[int, List[int]] = {}
    for to_bid, from_list in (preds or {}).items():
        for fr in from_list:
            succs.setdefault(fr, []).append(to_bid)
    return succs

def _nearest_pred_on_path(succs: Dict[int, List[int]], preds_list: List[int], decl_b: int, target_bid: int) -> Optional[int]:
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
    # Normalize predecessor list
    preds_raw = [p for p in builder.preds.get(block_id, []) if p != block_id]
    seen = set()
    preds_list: List[int] = []
    for p in preds_raw:
        if p not in seen:
            preds_list.append(p)
            seen.add(p)
    succs = _build_succs(builder.preds)
    # Precompute a non-self initial source for self-carry
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
            bd = int(b_decl); vs = int(v_src)
        except Exception:
            continue
        pred_match = _nearest_pred_on_path(succs, preds_list, bd, block_id)
        if pred_match is None:
            _trace({
                "phi": "wire_skip_no_path",
                "decl_b": bd,
                "target": int(block_id),
                "src": vs,
            })
            continue
        if vs == int(dst_vid) and init_src_vid is not None:
            vs = int(init_src_vid)
        try:
            val = builder.resolver._value_at_end_i64(vs, pred_match, builder.preds, builder.block_end_values, builder.vmap, builder.bb_map)
        except Exception:
            val = None
        if val is None:
            val = ir.Constant(builder.i64, 0)
        chosen[pred_match] = val
        _trace({
            "phi": "wire_choose",
            "pred": int(pred_match),
            "dst": int(dst_vid),
            "src": int(vs),
        })
    for pred_bid, val in chosen.items():
        pred_bb = builder.bb_map.get(pred_bid)
        if pred_bb is None:
            continue
        phi.add_incoming(val, pred_bb)
        _trace({"phi": "add_incoming", "dst": int(dst_vid), "pred": int(pred_bid)})

# ---- Public API (used by llvm_builder) ----

def setup_phi_placeholders(builder, blocks: List[Dict[str, Any]]):
    """Predeclare PHIs and collect incoming metadata for finalize_phis.

    This pass is function-local and must be invoked after basic blocks are
    created and before lowering individual blocks. It also tags string-ish
    values eagerly to help downstream resolvers choose correct intrinsics.
    """
    try:
        produced_str = _collect_produced_stringish(blocks)
        builder.block_phi_incomings = analyze_incomings(blocks)
        _trace({"phi": "setup", "produced_str_keys": list(produced_str.keys())})
        # Materialize placeholders and propagate stringish tags
        for block_data in blocks:
            bid0 = block_data.get("id", 0)
            bb0 = builder.bb_map.get(bid0)
            for inst in block_data.get("instructions", []) or []:
                if inst.get("op") != "phi":
                    continue
                try:
                    dst0 = int(inst.get("dst"))
                    incoming0 = inst.get("incoming", []) or []
                except Exception:
                    dst0 = None; incoming0 = []
                if dst0 is None or bb0 is None:
                    continue
                _ = ensure_phi(builder, bid0, dst0, bb0)
                # Tag propagation
                try:
                    dst_type0 = inst.get("dst_type")
                    mark_str = isinstance(dst_type0, dict) and dst_type0.get("kind") == "handle" and dst_type0.get("box_type") == "StringBox"
                    if not mark_str:
                        for (_b_decl_i, v_src_i) in incoming0:
                            try:
                                if produced_str.get(int(v_src_i)):
                                    mark_str = True; break
                            except Exception:
                                pass
                    if mark_str and hasattr(builder.resolver, 'mark_string'):
                        builder.resolver.mark_string(int(dst0))
                except Exception:
                    pass
                # Definition hint: PHI defines dst in this block
                try:
                    builder.def_blocks.setdefault(int(dst0), set()).add(int(bid0))
                except Exception:
                    pass
        # Sync to resolver
        try:
            builder.resolver.block_phi_incomings = builder.block_phi_incomings
        except Exception:
            pass
    except Exception:
        pass

def finalize_phis(builder):
    """Finalize PHIs declared in JSON by wiring incoming edges at block heads.
    Uses resolver._value_at_end_i64 to materialize values at predecessor ends.
    """
    for block_id, dst_map in (getattr(builder, 'block_phi_incomings', {}) or {}).items():
        for dst_vid, incoming in (dst_map or {}).items():
            wire_incomings(builder, int(block_id), int(dst_vid), incoming)
            _trace({"phi": "finalize", "block": int(block_id), "dst": int(dst_vid)})
