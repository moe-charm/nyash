"""
PHI wiring helpers

- setup_phi_placeholders: Predeclare PHIs and collect incoming metadata
- finalize_phis: Wire PHI incomings using end-of-block snapshots and resolver

These operate on the NyashLLVMBuilder instance to keep changes minimal.
"""

from typing import Dict, List, Any
import llvmlite.ir as ir

def setup_phi_placeholders(builder, blocks: List[Dict[str, Any]]):
    """Predeclare PHIs and collect incoming metadata for finalize_phis.

    This pass is function-local and must be invoked after basic blocks are
    created and before lowering individual blocks. It also tags string-ish
    values eagerly to help downstream resolvers choose correct intrinsics.
    """
    try:
        # Pass A: collect producer stringish hints per value-id
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
                        if t == "string" or (isinstance(t, dict) and t.get("kind") in ("handle","ptr") and t.get("box_type") == "StringBox"):
                            is_str = True
                    elif opx in ("binop","boxcall","externcall"):
                        t = inst.get("dst_type")
                        if isinstance(t, dict) and t.get("kind") == "handle" and t.get("box_type") == "StringBox":
                            is_str = True
                    if is_str:
                        produced_str[int(dstx)] = True
                except Exception:
                    pass
        # Pass B: materialize PHI placeholders and record incoming metadata
        builder.block_phi_incomings = {}
        for block_data in blocks:
            bid0 = block_data.get("id", 0)
            bb0 = builder.bb_map.get(bid0)
            for inst in block_data.get("instructions", []) or []:
                if inst.get("op") == "phi":
                    try:
                        dst0 = int(inst.get("dst"))
                        incoming0 = inst.get("incoming", []) or []
                    except Exception:
                        dst0 = None; incoming0 = []
                    if dst0 is None:
                        continue
                    # Record incoming metadata for finalize_phis
                    try:
                        builder.block_phi_incomings.setdefault(bid0, {})[dst0] = [
                            (int(b), int(v)) for (v, b) in incoming0
                        ]
                    except Exception:
                        pass
                    # Ensure placeholder exists at block head
                    if bb0 is not None:
                        b0 = ir.IRBuilder(bb0)
                        try:
                            b0.position_at_start(bb0)
                        except Exception:
                            pass
                        existing = builder.vmap.get(dst0)
                        is_phi = False
                        try:
                            is_phi = hasattr(existing, 'add_incoming')
                        except Exception:
                            is_phi = False
                        if not is_phi:
                            ph0 = b0.phi(builder.i64, name=f"phi_{dst0}")
                            builder.vmap[dst0] = ph0
                        # Tag propagation: if explicit dst_type marks string or any incoming was produced as string-ish, tag dst
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
    except Exception:
        pass

def finalize_phis(builder):
    """Finalize PHIs declared in JSON by wiring incoming edges at block heads.
    Uses resolver._value_at_end_i64 to materialize values at predecessor ends,
    ensuring casts/boxing are inserted in predecessor blocks (dominance-safe)."""
    # Build succ map for nearest-predecessor mapping
    succs: Dict[int, List[int]] = {}
    for to_bid, from_list in (builder.preds or {}).items():
        for fr in from_list:
            succs.setdefault(fr, []).append(to_bid)
    for block_id, dst_map in (getattr(builder, 'block_phi_incomings', {}) or {}).items():
        bb = builder.bb_map.get(block_id)
        if bb is None:
            continue
        b = ir.IRBuilder(bb)
        try:
            b.position_at_start(bb)
        except Exception:
            pass
        for dst_vid, incoming in (dst_map or {}).items():
            # Ensure placeholder exists at block head
            # Prefer predeclared ret-phi when available and force using it.
            predecl = getattr(builder, 'predeclared_ret_phis', {}) if hasattr(builder, 'predeclared_ret_phis') else {}
            phi = predecl.get((int(block_id), int(dst_vid))) if predecl else None
            if phi is not None:
                builder.vmap[dst_vid] = phi
            else:
                phi = builder.vmap.get(dst_vid)
                need_local_phi = False
                try:
                    if not (phi is not None and hasattr(phi, 'add_incoming')):
                        need_local_phi = True
                    else:
                        bb_of_phi = getattr(getattr(phi, 'basic_block', None), 'name', None)
                        if bb_of_phi != bb.name:
                            need_local_phi = True
                except Exception:
                    need_local_phi = True
                if need_local_phi:
                    phi = b.phi(builder.i64, name=f"phi_{dst_vid}")
                    builder.vmap[dst_vid] = phi
            # Wire incoming per CFG predecessor; map src_vid when provided
            preds_raw = [p for p in builder.preds.get(block_id, []) if p != block_id]
            # Deduplicate while preserving order
            seen = set()
            preds_list: List[int] = []
            for p in preds_raw:
                if p not in seen:
                    preds_list.append(p)
                    seen.add(p)
            # Helper: find the nearest immediate predecessor on a path decl_b -> ... -> block_id
            def nearest_pred_on_path(decl_b: int):
                from collections import deque
                q = deque([decl_b])
                visited = set([decl_b])
                parent: Dict[int, Any] = {decl_b: None}
                while q:
                    cur = q.popleft()
                    if cur == block_id:
                        par = parent.get(block_id)
                        return par if par in preds_list else None
                    for nx in succs.get(cur, []):
                        if nx not in visited:
                            visited.add(nx)
                            parent[nx] = cur
                            q.append(nx)
                return None
            # Precompute a non-self initial source (if present) to use for self-carry cases
            init_src_vid = None
            for (b_decl0, v_src0) in incoming:
                try:
                    vs0 = int(v_src0)
                except Exception:
                    continue
                if vs0 != int(dst_vid):
                    init_src_vid = vs0
                    break
            # Pre-resolve declared incomings to nearest immediate predecessors
            chosen: Dict[int, ir.Value] = {}
            for (b_decl, v_src) in incoming:
                try:
                    bd = int(b_decl); vs = int(v_src)
                except Exception:
                    continue
                pred_match = nearest_pred_on_path(bd)
                if pred_match is None:
                    continue
                if vs == int(dst_vid) and init_src_vid is not None:
                    vs = int(init_src_vid)
                try:
                    val = builder.resolver._value_at_end_i64(vs, pred_match, builder.preds, builder.block_end_values, builder.vmap, builder.bb_map)
                except Exception:
                    val = None
                if val is None:
                    # As a last resort, zero
                    val = ir.Constant(builder.i64, 0)
                chosen[pred_match] = val
            # Finally add incomings
            for pred_bid, val in chosen.items():
                pred_bb = builder.bb_map.get(pred_bid)
                if pred_bb is None:
                    continue
                phi.add_incoming(val, pred_bb)
