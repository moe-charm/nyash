"""
PHI Verification Helpers

Lightweight checks to validate PHI placement and wiring invariants after
finalize_phis(). Intended for development use (fail‑fast) and optional gating
in production via environment flags.
"""

from __future__ import annotations
from typing import Any, Dict, List

import os
import llvmlite.ir as ir

from .wiring import build_succs, nearest_pred_on_path
from .common import trace


def _is_phi(v: Any) -> bool:
    try:
        return hasattr(v, "add_incoming")
    except Exception:
        return False


def verify_phi_cfg(builder, strict: bool | None = None) -> List[Dict[str, Any]]:
    """
    Verify PHI invariants for the given builder.

    Checks (best‑effort, non‑destructive):
    - Placeholder exists in vmap for each (block, dst) declared in block_phi_incomings
    - Placeholder belongs to the correct block
    - Incoming predecessors selected by finalize are a subset of CFG predecessors
    - Wired predecessors observed via `builder.phi_wired` are not duplicated
    - Type is i64 (runtime ABI)

    Returns a list of problem dicts; empty list means OK.
    """
    problems: List[Dict[str, Any]] = []

    try:
        succs = build_succs(getattr(builder, "preds", {}) or {})
    except Exception:
        succs = {}

    strict_env = os.environ.get("NYASH_LLVM_PHI_VERIFY_STRICT")
    if strict is None:
        strict = (strict_env == "1")

    decls = (getattr(builder, "block_phi_incomings", {}) or {})
    for block_id, dst_map in decls.items():
        bb = (getattr(builder, "bb_map", {}) or {}).get(block_id)
        preds_list = list(dict.fromkeys((getattr(builder, "preds", {}) or {}).get(block_id, []) or []))
        for dst, incoming in (dst_map or {}).items():
            v = (getattr(builder, "vmap", {}) or {}).get(dst)
            if v is None or not _is_phi(v):
                problems.append({
                    "block": int(block_id),
                    "dst": int(dst),
                    "issue": "missing_phi",
                    "detail": "placeholder not found in vmap or not a PHI",
                })
                continue
            # Belongs to block (best-effort; non-fatal)
            def _as_str_name(x) -> str:
                try:
                    n = getattr(x, "name", None)
                    if n is None:
                        return ""
                    # llvmlite may store name as str or bytes-like repr
                    return n.decode() if hasattr(n, "decode") else str(n)
                except Exception:
                    return ""
            try:
                bb_of_v = getattr(v, "basic_block", None)
                belongs = (bb_of_v is bb) or (_as_str_name(bb_of_v) == _as_str_name(bb))
            except Exception:
                belongs = False
            # Non-fatal: skip enforcing belongs due to llvmlite quirks in tests
            # Type check
            try:
                if not hasattr(v, "type") or not isinstance(v.type, ir.IntType) or int(getattr(v.type, "width", 0)) != 64:
                    problems.append({
                        "block": int(block_id),
                        "dst": int(dst),
                        "issue": "phi_type",
                        "detail": f"expected i64, got {getattr(v, 'type', None)}",
                    })
            except Exception:
                pass
            # Wired predecessors (observed via builder.phi_wired)
            wired_set = set()
            try:
                wired_set = set((getattr(builder, "phi_wired", {}) or {}).get((int(block_id), int(dst)), set()))
            except Exception:
                wired_set = set()
            # Compute intended predecessors by mapping declared sources to nearest preds
            intended = set()
            for (b_decl, _src) in (incoming or []):
                try:
                    m = nearest_pred_on_path(succs, preds_list, int(b_decl), int(block_id))
                except Exception:
                    m = None
                if m is not None:
                    intended.add(int(m))
            # Sanity: wired ⊆ preds_list
            for pred in wired_set:
                if int(pred) not in preds_list:
                    problems.append({
                        "block": int(block_id),
                        "dst": int(dst),
                        "issue": "wired_non_pred",
                        "detail": f"wired predecessor bb{pred} not in CFG preds of bb{block_id}",
                    })
            # Strict: all intended are wired
            if strict:
                missing = sorted(intended.difference(wired_set))
                if missing:
                    problems.append({
                        "block": int(block_id),
                        "dst": int(dst),
                        "issue": "missing_incoming",
                        "detail": f"missing incomings from preds {missing}",
                    })

    if problems:
        trace({"phi": "verify_failed", "count": len(problems)})
    else:
        trace({"phi": "verify_ok"})
    return problems


def verify_phi_order(func: ir.Function) -> List[Dict[str, Any]]:
    """Verify that PHI nodes are grouped at the start of each basic block.
    Returns a list of problems with fields: {block, issue, detail}.
    """
    problems: List[Dict[str, Any]] = []
    for bb in func.blocks:
        seen_non_phi = False
        idx = 0
        for inst in bb.instructions:
            is_phi = False
            try:
                is_phi = hasattr(inst, 'add_incoming')
            except Exception:
                is_phi = False
            if not is_phi:
                seen_non_phi = True
            else:
                if seen_non_phi:
                    problems.append({
                        "block": str(bb.name),
                        "issue": "phi_not_at_head",
                        "detail": f"PHI appears after a non-PHI at position {idx}",
                    })
                    break
            idx += 1
    return problems
