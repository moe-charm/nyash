from __future__ import annotations
from typing import Dict, List, Any, Tuple

from .common import trace


def collect_produced_stringish(blocks: List[Dict[str, Any]]) -> Dict[int, bool]:
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
                    if t == "string" or (
                        isinstance(t, dict)
                        and t.get("kind") in ("handle", "ptr")
                        and t.get("box_type") == "StringBox"
                    ):
                        is_str = True
                elif opx in ("binop", "boxcall", "externcall"):
                    t = inst.get("dst_type")
                    if (
                        isinstance(t, dict)
                        and t.get("kind") == "handle"
                        and t.get("box_type") == "StringBox"
                    ):
                        is_str = True
                if is_str:
                    produced_str[int(dstx)] = True
            except Exception:
                pass
    return produced_str


def analyze_incomings(blocks: List[Dict[str, Any]]) -> Dict[int, Dict[int, List[Tuple[int, int]]]]:
    """Return block_phi_incomings: block_id -> { dst_vid -> [(decl_b, v_src), ...] }"""
    result: Dict[int, Dict[int, List[Tuple[int, int]]]] = {}
    for block_data in blocks:
        bid0 = block_data.get("id", 0)
        for inst in block_data.get("instructions", []) or []:
            if inst.get("op") == "phi":
                try:
                    dst0 = int(inst.get("dst"))
                    from .common import incoming_pairs_vb
                incoming0 = incoming_pairs_vb(inst)
                except Exception:
                    dst0 = None
                    incoming0 = []
                if dst0 is None:
                    continue
                try:
                    pairs = [(int(b), int(v)) for (v, b) in incoming0]
                    result.setdefault(int(bid0), {})[dst0] = pairs
                    trace({
                        "phi": "analyze",
                        "block": int(bid0),
                        "dst": dst0,
                        "incoming": pairs,
                    })
                except Exception:
                    pass
    return result

